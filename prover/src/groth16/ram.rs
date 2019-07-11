use circom2_compiler::{
    evaluator::print_info,
    evaluator::{Evaluator,Mode},
    algebra::{Value,FS}
};

use std::fs::File;
use super::error::{Error,Result};

use circom2_compiler::storage::{Constraints, Signals};
use circom2_compiler::storage::{Ram, StorageFactory};
use circom2_compiler::tester::dump_error;


pub fn setup_ram(circuit_path: &str, proving_key_path: &str, verificator_key_path: &str) -> Result<()> {

    let mut storage = Ram::default();

    let mut eval = Evaluator::new(
        Mode::GenConstraints,
        storage.new_signals()?,
        storage.new_constraints()?,
    );
    info!("Compiling circuit...");

    if let Err(err) = eval.eval_file(".", &circuit_path) {
        dump_error(&eval, &format!("{:?}", err));
        return Err(Error::from(err));
    }

    print_info(&eval,false);
    info!("Running setup");

    let (pk,vk) = (
        File::create(proving_key_path)?,
        File::create(verificator_key_path)?
    );

    super::setup(&eval, pk, vk)?;

    Ok(())
}

pub fn prove_ram(circuit_path: &str,proving_key_path: &str, inputs: Vec<(String,FS)>) -> Result<String> {

    info!("Generating witness...");

    let mut ram = Ram::default();
    let mut ev_witness = Evaluator::new(
        Mode::GenWitness,
        ram.new_signals()?,
        ram.new_constraints()?,
    );

    info!("Checking constraints...");

    if ev_witness.constraints.len()? > 0 {
        return Err(Error::Unexpected("Constrains generated in witnes".to_string()));
    }

    info!("Checking signals...");

    for n in 1..ev_witness.signals.len()? {
        let signal = &*ev_witness.signals.get_by_id(n).unwrap().unwrap();
        if signal.value.is_none() {
            return Err(Error::Unexpected(format!("signal '{}' value is not defined",signal.full_name.0)));
        }  
    }

    for (signal,value) in inputs {
        ev_witness.set_deferred_value(signal, Value::from(value));
    }

    ev_witness.eval_file(".", &circuit_path)?;

    // Create proof
    info!("Creating and self-verifying proof...");

    let pk = File::open(proving_key_path)?;

    let mut proof = Vec::new();

    let _ = super::generate_verified_proof(
        ev_witness.signals,
        pk,
        &mut proof
    )?;

    info!("Proof generated and self-verified");

    Ok(String::from_utf8_lossy(&proof).to_string())
}
