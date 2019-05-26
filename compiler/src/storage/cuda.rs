use super::Result;
use crate::algebra::{LC, QEQ};
use crate::storage::{Constraints, Signals};
use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use circom2_parser::ast::SignalType;

pub fn generate_cuda(path: &str, constraints: &Constraints, signals: &Signals) -> Result<()> {

    // find the number of public inputs, by now should be ordered 
    //   in the following way:
    //
    //   SignalType::Internal      (value=one)
    //   SignalType::PrivateInput  (value not set)
    //   SignalType::PublicInput   (value not set)
    //
    
    println!("Scanning constraints...");

    let mut public_signal_count = 0;
    let mut private_signal_count = 0;

    for n in 0..signals.len()? {
        let s = signals.get_by_id(n)?.unwrap();
        let component_len = s.full_name.0.chars().filter(|ch| *ch == '.').count();
        if component_len == 1 {
            match s.xtype {
                SignalType::Output | SignalType::PublicInput => public_signal_count += 1,
                SignalType::PrivateInput => private_signal_count += 1,
                _ => {}
            }
        }
    }

    println!("public_signal_count = {}",public_signal_count);
    println!("private_signal_count = {}",private_signal_count);

    let input_signals_count = public_signal_count + private_signal_count;

    println!("Writing cuda constraints file {}...",path);

    let mut file = File::create(&path)?;

    // nWords : File size in 32 bit workds --------------- 32 bits
    let offset_words = file.seek(SeekFrom::Current(0))?;
    file.write_u32::<LittleEndian>(0).unwrap();

    // nPubInputs : -------------------------------------- 32 bits
    file.write_u32::<LittleEndian>(input_signals_count).unwrap();

    // nOutputs   : -------------------------------------- 32 bits
    file.write_u32::<LittleEndian>(0).unwrap();

    // nVars      : -------------------------------------- 32 bits
    file.write_u32::<LittleEndian>(signals.len()? as u32)
        .unwrap();

    // nConstraints : Number of constraints--------------- 32 bits
    file.write_u32::<LittleEndian>(constraints.len()? as u32)
        .unwrap();

    // R1CSA_nWords : R1CSA size in 32 bit words --------- 32 bits
    let offset_r1cs_a = file.seek(SeekFrom::Current(0))?;
    file.write_u32::<LittleEndian>(0).unwrap();

    // R1CSB_nWords : R1CSB size in 32 bit words --------- 32 bits
    let offset_r1cs_b = file.seek(SeekFrom::Current(0))?;
    file.write_u32::<LittleEndian>(0).unwrap();

    // R1CSC_nWords : R1CSC size in 32 bit words --------- 32 bits
    let offset_r1cs_c = file.seek(SeekFrom::Current(0))?;
    file.write_u32::<LittleEndian>(0).unwrap();

    fn write_lc(file: &mut File, constraints: &Constraints, lc_of: &Fn(QEQ) -> LC) -> Result<()> {
        let zeroes = vec![0; 32];
        let constraints_len = constraints.len()?;

        // N constraints  -------------------------------- 32 bits
        file.write_u32::<LittleEndian>(constraints_len as u32)
            .unwrap();

        // cumsum(  -> cumulative
        //    N coeff constraints[0] ---------------------- 32 bits
        //    N coeff constraints[1] ---------------------- 32 bits : N constraints[0] + N constraints[1]
        //    ----
        //    N coeff constraints[N-1] -------------------- 32 bits : N contraints[0] + N constraints[1] +
        //                                                            N constraints[2] +...+ Nconstraints[N-1]
        // )
        let mut coeff_count = 0;
        for n in 0..constraints_len {
            let lc = lc_of(constraints.get(n)?).0;
            coeff_count += lc.len();
            file.write_u32::<LittleEndian>(coeff_count as u32).unwrap();
        }

        for n in 0..constraints_len {
            let lc = lc_of(constraints.get(n)?).0;
            for (signal_id, _) in lc.iter() {
                file.write_u32::<LittleEndian>(*signal_id as u32).unwrap();
            }
            for (_, mult) in lc.iter() {
                let le = mult.0.to_bytes_le();
                file.write_all(&le)?;
                if le.len() < 32 {
                    file.write_all(&zeroes[le.len()..32])?;
                }
            }
        }
        Ok(())
    }

    // Write R1CS.a
    let offset_start_a = file.seek(SeekFrom::Current(0))? as u32;
    write_lc(&mut file, constraints, &|qeq| qeq.a)?;

    // Write R1CS.b
    let offset_start_b = file.seek(SeekFrom::Current(0))? as u32;
    write_lc(&mut file, constraints, &|qeq| qeq.b)?;

    // Write -R1CS.c
    let offset_start_c = file.seek(SeekFrom::Current(0))? as u32;
    write_lc(&mut file, constraints, &|qeq| -&(qeq.c))?;

    let offset_end = file.seek(SeekFrom::End(0))? as u32;

    // Write R1CS.a len
    file.seek(SeekFrom::Start(offset_r1cs_a))?;
    file.write_u32::<LittleEndian>((offset_start_b - offset_start_a) / 4)
        .unwrap();

    // Write R1CS.b len
    file.seek(SeekFrom::Start(offset_r1cs_b))?;
    file.write_u32::<LittleEndian>((offset_start_c - offset_start_b) / 4)
        .unwrap();

    // Write R1CS.c len
    file.seek(SeekFrom::Start(offset_r1cs_c))?;
    file.write_u32::<LittleEndian>((offset_end - offset_start_c) / 4)
        .unwrap();

    // Write nWords
    file.seek(SeekFrom::Start(offset_words))?;
    file.write_u32::<LittleEndian>(offset_end / 4).unwrap();

    Ok(())
}

