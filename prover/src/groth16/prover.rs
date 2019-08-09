use circom2_compiler::algebra::FS;
use circom2_compiler::evaluator::{Evaluator,check_constrains_eval_zero};
use circom2_compiler::storage;
use circom2_compiler::storage::{Constraints, Signals,public_inputs,is_public_input};

use std::time::SystemTime;
use std::io::{Read, Write};
use std::marker::PhantomData;

use pairing::bn256::{Bn256, Fr};
use pairing::Engine;

use bellman::{Circuit, ConstraintSystem, SynthesisError};
use bellman::groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof
};

use ff_ce::PrimeField;

use rand::thread_rng;

use super::error::Result;
use super::format::*;

const BELLMAN_VERBOSE : &str = "BELLMAN_VERBOSE";

pub fn bellman_verbose(verbose : bool) {
    if verbose {
            std::env::set_var(BELLMAN_VERBOSE, "1");
    } else {
            std::env::set_var(BELLMAN_VERBOSE, "0");
    }
}


pub struct CircomCircuit<'a, E: Engine> {
    constraints: &'a Constraints,
    signals: &'a Signals,
    phantom: PhantomData<E>,
}

impl<'a, E: Engine> CircomCircuit<'a, E> {}

fn map_storage_error<V>(
    e: std::result::Result<V, storage::Error>,
) -> std::result::Result<V, SynthesisError> {
    match e {
        Ok(v) => Ok(v),
        _ => Err(SynthesisError::Unsatisfiable),
    }
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
                            .map(|vv| value_to_bellman_fr::<E>(&vv))
                            .ok_or(SynthesisError::AssignmentMissing)
                    },
                )?
            } else {
                cs.alloc(
                    || (*s.full_name.0).clone(),
                    || {
                        s.value
                            .clone()
                            .map(|vv| value_to_bellman_fr::<E>(&vv))
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

pub fn setup<S: Signals, C: Constraints, W: Write>(
    eval: &Evaluator<S, C>,
    out_pk: W,
) -> Result<(bellman::groth16::VerifyingKey<Bn256>,Vec<String>)> {
    let rng = &mut thread_rng();
    let circuit = CircomCircuit::<Bn256> {
        signals: &eval.signals,
        constraints: &eval.constraints,
        phantom: PhantomData,
    };

    // perform setup
    let start = SystemTime::now();
    let params = generate_random_parameters(circuit, rng)?;
    info!("Setup time: {:?}",SystemTime::now().duration_since(start).unwrap());
    let start = SystemTime::now();
    write_pk(out_pk, &eval.constraints, &params)?;
    info!("Proving key write time: {:?}",SystemTime::now().duration_since(start).unwrap());
    
    let inputs = public_inputs(&eval.signals)?; 

    Ok((params.vk,inputs))
}

pub fn generate_verified_proof<S: Signals, R: Read, W: Write>(
    signals: S,
    in_pk: R,
    out_proof: &mut W
) -> Result<Vec<(String,FS)>> {

    let rng = &mut thread_rng();
    
    let start = SystemTime::now();
    let (constraints, params) = read_pk(in_pk)?;
    info!("Proving key read time: {:?}",SystemTime::now().duration_since(start).unwrap());

    let start = SystemTime::now();
    check_constrains_eval_zero(&constraints,&signals)
        .expect("check_constrains_eval_zero failed");
    info!("Constraint check time: {:?}",SystemTime::now().duration_since(start).unwrap());

    let circuit = CircomCircuit::<Bn256> {
        signals: &signals,
        constraints: &constraints,
        phantom: PhantomData,
    };

    // Create proof
    let start = SystemTime::now();
    let proof = create_random_proof(circuit, &params, rng).expect("cannot create proof");
    info!("Proof generation time: {:?}",SystemTime::now().duration_since(start).unwrap());

    // Self-verify and generate public inputs
    let start = SystemTime::now();
    let mut public_inputs = Vec::new();
    for i in 0..signals.len()? {
        let signal = signals.get_by_id(i)?.unwrap();
        if is_public_input(&signal) {
            let fs = (&*signal).clone().value.unwrap().try_into_fs().unwrap();
            let name = signal.full_name.0.to_string();
            public_inputs.push((name,fs));
        }
    }

    let vk = prepare_verifying_key(&params.vk);
    let verify_public_inputs = public_inputs
        .iter()
        .map(|(_,n)| Fr::from_str(&(n.to_string()))
            .expect(&format!("cannot parse fe {}",&n.to_string())))
        .collect::<Vec<_>>();

    verify_proof(&vk, &proof, &verify_public_inputs)?;
    JsonProofAndInput::from_bellman(proof, public_inputs.clone())?.write(out_proof)?;
    info!("Proof verification time: {:?}",SystemTime::now().duration_since(start).unwrap());

    Ok(public_inputs)
}

#[cfg(test)]
mod test {
    use super::*;

    use bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };
    use circom2_compiler::algebra::Value;
    use circom2_compiler::evaluator::{Evaluator, Mode, Scope};
    use circom2_compiler::storage::Ram;
    use circom2_compiler::storage::StorageFactory;
    use pairing::bn256::{Bn256, Fr};
    use rand::thread_rng;
    use std::fs::File;

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

        // check constraints
        check_constrains_eval_zero(&ev_r1cs.constraints,&ev_witness.signals)
            .expect("cannot check all constraints = 0");

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

        let pk = File::create("/tmp/pk").unwrap();
        let (_,_) = setup(&ev_r1cs, pk).expect("cannot setup");

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

        check_constrains_eval_zero(&ev_r1cs.constraints,&ev_witness.signals)
            .expect("cannot check all constraints = 0");

        // Create and verify proof
        let mut proof_out = Vec::new();
        let pk = File::open("/tmp/pk").unwrap();
        let public_input = generate_verified_proof(ev_witness.signals, pk, &mut proof_out).unwrap();
        assert_eq!("[(\"main.c\", 21)]", format!("{:?}", public_input));

    }

}
