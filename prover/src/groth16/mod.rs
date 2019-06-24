use circom2_compiler::algebra::{Value, FS, LC, QEQ};
use circom2_compiler::storage;
use circom2_compiler::storage::Ram;
use circom2_compiler::storage::RamConstraints;
use circom2_compiler::storage::StorageFactory;
use circom2_compiler::storage::{Constraints, Signals};
use circom2_parser::ast::SignalType;

use std::io::{Read, Write};
use std::marker::PhantomData;

use pairing::Engine;

use bellman::{Circuit, ConstraintSystem, LinearCombination, SynthesisError};

use ff::PrimeField;

use circom2_compiler::evaluator::{Evaluator, Mode, Scope};
use pairing::bn256::{Bn256, Fr};
use rand::thread_rng;

use bellman::groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    Parameters, Proof
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use serde_cbor::{from_slice, to_vec};

mod parse;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Synthesis(bellman::SynthesisError),
    Storage(circom2_compiler::storage::Error),
    Cbor(serde_cbor::error::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<bellman::SynthesisError> for Error {
    fn from(err: bellman::SynthesisError) -> Self {
        Error::Synthesis(err)
    }
}

impl From<circom2_compiler::storage::Error> for Error {
    fn from(err: circom2_compiler::storage::Error) -> Self {
        Error::Storage(err)
    }
}

impl From<serde_cbor::error::Error> for Error {
    fn from(err: serde_cbor::error::Error) -> Self {
        Error::Cbor(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct CircomCircuit<'a, E: Engine> {
    constraints: &'a Constraints,
    signals: &'a Signals,
    phantom: PhantomData<E>,
}

impl<'a, E: Engine> CircomCircuit<'a, E> {}

fn value_to_bellman<E: Engine>(value: &Value) -> E::Fr {
    match value {
        Value::FieldScalar(fs) => fe_to_bellman::<E>(fs),
        _ => panic!("Invalid signal value"),
    }
}

fn fe_to_bellman<E: Engine>(fe: &FS) -> E::Fr {
    E::Fr::from_str(&fe.0.to_str_radix(10)).unwrap()
}

fn lc_to_bellman<E: Engine>(
    mut base: LinearCombination<E>,
    signals: &[bellman::Variable],
    lc: &LC,
) -> LinearCombination<E> {
    use std::ops::Add;
    for (s, v) in &lc.0 {
        base = base.add((fe_to_bellman::<E>(&v), signals[*s]));
    }
    base
}

fn map_storage_error<V>(
    e: std::result::Result<V, storage::Error>,
) -> std::result::Result<V, SynthesisError> {
    match e {
        Ok(v) => Ok(v),
        _ => Err(SynthesisError::Unsatisfiable),
    }
}

fn is_public_input(signal: &circom2_compiler::storage::Signal) -> bool {
    let component_len = signal.full_name.0.chars().filter(|ch| *ch == '.').count();
    component_len == 1
        && (signal.xtype == SignalType::Output || signal.xtype == SignalType::PublicInput)
}

impl<'a, E: Engine> Circuit<E> for CircomCircuit<'a, E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self,
        cs: &mut CS,
    ) -> std::result::Result<(), SynthesisError> {
        let mut signals = Vec::new();
        signals.push(CS::one());

        // register signals
        for n in 1..map_storage_error(self.signals.len())? {
            let s = map_storage_error(self.signals.get_by_id(n))?.unwrap();
            let signal = if is_public_input(&s) {
                cs.alloc_input(
                    || (*s.full_name.0).clone(),
                    || {
                        s.value
                            .clone()
                            .map(|vv| value_to_bellman::<E>(&vv))
                            .ok_or(SynthesisError::AssignmentMissing)
                    },
                )?
            } else {
                cs.alloc(
                    || (*s.full_name.0).clone(),
                    || {
                        s.value
                            .clone()
                            .map(|vv| value_to_bellman::<E>(&vv))
                            .ok_or(SynthesisError::AssignmentMissing)
                    },
                )?
            };
            signals.push(signal);
        }

        // register constrains
        for n in 0..map_storage_error(self.constraints.len())? {
            let constraint = map_storage_error(self.constraints.get(n))?;
            let name = format!("c{}", n);
            cs.enforce(
                || name,
                |lc| lc_to_bellman(lc, &signals, &constraint.a),
                |lc| lc_to_bellman(lc, &signals, &constraint.b),
                |lc| lc_to_bellman(lc, &signals, &-&constraint.c),
            );
        }
        Ok(())
    }
}

fn write_pk<W: Write, C: Constraints>(
    mut pk: W,
    constraints: &C,
    params: &Parameters<Bn256>,
) -> Result<()> {
    // write constratins & proving key
    pk.write_u32::<BigEndian>(constraints.len()? as u32)?;
    for i in 0..constraints.len()? {
        let qeq = to_vec(&constraints.get(i)?)?;
        pk.write_u32::<BigEndian>(qeq.len() as u32)?;
        pk.write(&qeq)?;
    }
    
    params.write(pk)?;
    Ok(())
}

fn read_pk<R: Read>(mut pk: R) -> Result<(RamConstraints, Parameters<Bn256>)> {
    let mut buffer = Vec::with_capacity(1024);
    let mut constraints = Ram::default().new_constraints()?;
    let count = pk.read_u32::<BigEndian>()?;
    dbg!(count);

    for _ in 0..count {
        let len = pk.read_u32::<BigEndian>()? as usize;
        if len > buffer.capacity() {
            buffer.reserve(buffer.capacity() - len);
        }
        buffer.resize(len, 0u8);
        pk.read_exact(&mut buffer)?;
        let qeq = from_slice::<QEQ>(&buffer)?;
        dbg!(&qeq);
        constraints.push(qeq)?;
    }

    let params: Parameters<Bn256> = Parameters::read(pk, true)?;

    Ok((constraints, params))
}

pub fn setup<S: Signals, C: Constraints, WP: Write, WV: Write>(
    eval: Evaluator<S, C>,
    mut out_pk: WP,
    mut out_vk: WV,
) -> Result<()> {
    let rng = &mut thread_rng();
    let circuit = CircomCircuit::<Bn256> {
        signals: &eval.signals,
        constraints: &eval.constraints,
        phantom: PhantomData,
    };

    // perform setup
    let params = generate_random_parameters(circuit, rng)?;

    let out_pk = write_pk(out_pk, &eval.constraints, &params)?;

    // write verification key
    let mut contract = String::from(CONTRACT_TEMPLATE);
    contract = contract.replace("<%vk_a%>", &parse::parse_g1_hex(&params.vk.alpha_g1));
    contract = contract.replace("<%vk_b%>", &parse::parse_g2_hex(&params.vk.beta_g2));
    contract = contract.replace("<%vk_gamma%>", &parse::parse_g2_hex(&params.vk.gamma_g2));
    contract = contract.replace("<%vk_delta%>", &parse::parse_g2_hex(&params.vk.delta_g2));
    contract = contract.replace(
        "<%vk_gammaABC_length%>",
        &format!("{}", &params.vk.ic.len()),
    );
    contract = contract.replace(
        "<%vk_gammaABC_pts%>",
        &params
            .vk
            .ic
            .iter()
            .enumerate()
            .map(|(i, x)| format!("vk.gammaABC[{}] = {}", i, parse::parse_g1_hex(x)))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    out_vk.write_all(contract.as_bytes())?;

    Ok(())
}

pub fn proof<S: Signals, R: Read, W: Write>(
    signals: S,
    in_pk: R,
    out_proof: W,
) -> Result<Vec<FS>> {
    let rng = &mut thread_rng();

    let (constraints, params) = read_pk(in_pk)?;

    let circuit = CircomCircuit::<Bn256> {
        signals: &signals,
        constraints: &constraints,
        phantom: PhantomData,
    };

    // Create proof
    let proof = create_random_proof(circuit, &params, rng).expect("cannot create proof");
    proof.write(out_proof)?;

    let mut public_inputs = Vec::new();
    for i in 0..signals.len()? {
        let signal = signals.get_by_id(i)?.unwrap();
        if is_public_input(&signal) {
            let fs = (&*signal).clone().value.unwrap().try_into_fs().unwrap();
            public_inputs.push(fs);
        }
    }

    Ok(public_inputs)
}

pub fn verify<RPK: Read, RPR: Read>(
    in_pk: RPK,
    in_proof: RPR,
    in_public_input: Vec<FS>,
) -> Result<bool> {
    let proof: Proof<Bn256> = Proof::read(in_proof)?;    

    let (_, params) = read_pk(in_pk)?;
    let vk = prepare_verifying_key(&params.vk);
    
    let in_public_input = in_public_input
        .into_iter()
        .map(|n| Fr::from_str(&n.0.to_string()).unwrap())
        .collect::<Vec<_>>();

    Ok(verify_proof(&vk, &proof, &in_public_input)?)
}

const CONTRACT_TEMPLATE: &str = r#"
contract Verifier {
    using Pairing for *;
    struct VerifyingKey {
        Pairing.G1Point a;
        Pairing.G2Point b;
        Pairing.G2Point gamma;
        Pairing.G2Point delta;
        Pairing.G1Point[] gammaABC;
    }
    struct Proof {
        Pairing.G1Point A;
        Pairing.G2Point B;
        Pairing.G1Point C;
    }
    function verifyingKey() pure internal returns (VerifyingKey memory vk) {
        vk.a = Pairing.G1Point(<%vk_a%>);
        vk.b = Pairing.G2Point(<%vk_b%>);
        vk.gamma = Pairing.G2Point(<%vk_gamma%>);
        vk.delta = Pairing.G2Point(<%vk_delta%>);
        vk.gammaABC = new Pairing.G1Point[](<%vk_gammaABC_length%>);
        <%vk_gammaABC_pts%>
    }
    function verify(uint[] memory input, Proof memory proof) internal returns (uint) {
        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.gammaABC.length);
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint i = 0; i < input.length; i++)
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.gammaABC[i + 1], input[i]));
        vk_x = Pairing.addition(vk_x, vk.gammaABC[0]);
        if(!Pairing.pairingProd4(
             proof.A, proof.B,
             Pairing.negate(vk_x), vk.gamma,
             Pairing.negate(proof.C), vk.delta,
             Pairing.negate(vk.a), vk.b)) return 1;
        return 0;
    }
    event Verified(string s);
    function verifyTx(
            uint[2] memory a,
            uint[2][2] memory b,
            uint[2] memory c,
            uint[<%vk_input_length%>] memory input
        ) public returns (bool r) {
        Proof memory proof;
        proof.A = Pairing.G1Point(a[0], a[1]);
        proof.B = Pairing.G2Point([b[0][0], b[0][1]], [b[1][0], b[1][1]]);
        proof.C = Pairing.G1Point(c[0], c[1]);
        uint[] memory inputValues = new uint[](input.length);
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i];
        }
        if (verify(inputValues, proof) == 0) {
            emit Verified("Transaction successfully verified.");
            return true;
        } else {
            return false;
        }
    }
}
"#;

#[cfg(test)]
mod test {
    use super::*;

    use circom2_compiler::evaluator::{Evaluator, Mode, Scope};
    use circom2_compiler::storage::Ram;
    use circom2_compiler::storage::StorageFactory;
    use pairing::bn256::{Bn256, Fr};
    use rand::thread_rng;
    use std::fs::File;
    use bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };

    use std::marker::PhantomData;

    #[test]
    fn test_generate_internal() {
        let circuit = "
            template t() {
                signal private input a;
                signal private input b;  
                signal output c;

                c <== a * b;  
            }
            component main = t();
        ";

        let mut ram = Ram::default();
        let mut ev_r1cs = Evaluator::new(
            Mode::GenConstraints,
            ram.new_signals().unwrap(),
            ram.new_constraints().unwrap(),
        );

        ev_r1cs
            .eval_inline(&mut Scope::new(true, None, "root".to_string()), circuit)
            .unwrap();

        let rng = &mut thread_rng();

        // Create parameters for our circuit
        println!("Run setup ---------------------------------");
        let params = {
            let circuit = CircomCircuit::<Bn256> {
                signals: &ev_r1cs.signals,
                constraints: &ev_r1cs.constraints,
                phantom: PhantomData,
            };

            generate_random_parameters(circuit, rng).unwrap()
        };

        // Prepare the verification key (for proof verification)
        let pvk = prepare_verifying_key(&params.vk);

        // Compute witness
        let mut ram = Ram::default();
        let mut ev_witness = Evaluator::new(
            Mode::GenWitness,
            ram.new_signals().unwrap(),
            ram.new_constraints().unwrap(),
        );

        ev_witness.set_deferred_value("main.a".to_string(), Value::from(7));
        ev_witness.set_deferred_value("main.b".to_string(), Value::from(3));
        ev_witness
            .eval_inline(&mut Scope::new(true, None, "root".to_string()), circuit)
            .unwrap();

        println!("Creating proofs --------------------------------- ");
        let circuit = CircomCircuit::<Bn256> {
            signals: &ev_witness.signals,
            constraints: &ev_r1cs.constraints,
            phantom: PhantomData,
        };

        // Create proof
        let proof = create_random_proof(circuit, &params, rng).expect("cannot create proof");

        // Verify with valid public input
        let public_input = Fr::from_str("21");
        let success =
            verify_proof(&pvk, &proof, &[public_input.unwrap()]).expect("cannot verify proof");

        assert!(success);

        // Verify with invalid public input
        let public_input = Fr::from_str("22");
        let success =
            verify_proof(&pvk, &proof, &[public_input.unwrap()]).expect("cannot verify proof");

        assert!(!success);
    }

    #[test]
    fn test_generate_helper() {
        let circuit = "
            template t() {
                signal private input a;
                signal private input b;  
                signal output c;

                c <== a * b;  
            }
            component main = t();
        ";

        let mut ram = Ram::default();
        let mut ev_r1cs = Evaluator::new(
            Mode::GenConstraints,
            ram.new_signals().unwrap(),
            ram.new_constraints().unwrap(),
        );
        ev_r1cs
            .eval_inline(&mut Scope::new(true, None, "root".to_string()), circuit)
            .unwrap();

        // setup -----------------------------------------------------

        let (pk,vk) = (
            File::create("/tmp/pk").unwrap(),
            File::create("/tmp/ver.sol").unwrap()
        );
        setup(ev_r1cs, pk, vk).expect("cannot setup");

        // Compute witness -------------------------------------------
        let mut ram = Ram::default();
        let mut ev_witness = Evaluator::new(
            Mode::GenWitness,
            ram.new_signals().unwrap(),
            ram.new_constraints().unwrap(),
        );

        ev_witness.set_deferred_value("main.a".to_string(), Value::from(7));
        ev_witness.set_deferred_value("main.b".to_string(), Value::from(3));
        ev_witness
            .eval_inline(&mut Scope::new(true, None, "root".to_string()), circuit)
            .unwrap();

        // Create proof
        let mut proof_out = Vec::new();
        let pk = File::open("/tmp/pk").unwrap();
        let public_input = proof(ev_witness.signals, pk, &mut proof_out).unwrap();
        assert_eq!("[21]",format!("{:?}",public_input));

        // Verify with valid public input
        let pk = File::open("/tmp/pk").unwrap();
        assert_eq!(verify(pk,proof_out.as_slice(),public_input).unwrap(),true);

        // Verify with invalid public input
        let pk = File::open("/tmp/pk").unwrap();
        assert_eq!(verify(pk,proof_out.as_slice(),vec![FS::from(22)]).unwrap(),false);

    }

}
