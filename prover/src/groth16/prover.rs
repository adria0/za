use za_parser::ast::BodyElementP;

use za_compiler::algebra::{FS,SignalId};
use za_compiler::types::{Constraints, Signals};

use std::io::Write;
use std::marker::PhantomData;
use std::time::SystemTime;

use pairing::bn256::{Bn256, Fr};
use pairing::Engine;

use bellman::groth16::{
    Parameters,
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
};
use bellman::{Circuit, ConstraintSystem, SynthesisError};

use ff_ce::PrimeField;

use rand::thread_rng;

use super::error::Result;
use super::format::*;

const BELLMAN_VERBOSE: &str = "BELLMAN_VERBOSE";

pub fn bellman_verbose(verbose: bool) {
    if verbose {
        std::env::set_var(BELLMAN_VERBOSE, "1");
    } else {
        std::env::set_var(BELLMAN_VERBOSE, "0");
    }
}

pub struct CircomCircuit<'a, E: Engine> {
    constraints: &'a Constraints,
    signals: &'a Signals,
    ignore_signals: &'a Vec<SignalId>,
    phantom: PhantomData<E>,
}

impl<'a,E: Engine> CircomCircuit<'a, E> {}

impl<'a,E: Engine> Circuit<E> for CircomCircuit<'a, E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self,
        cs: &mut CS,
    ) -> std::result::Result<(), SynthesisError> {
        let mut signals = Vec::new();
        signals.push(Some(CS::one()));

        let mut ignore_it = self.ignore_signals.iter().peekable();
        // register signals
        for n in 1..self.signals.len() {

            let s = self.signals.get_by_id(n).unwrap();

            if let Some(ignore) = ignore_it.peek() {
                if **ignore == n {
                    signals.push(None);
                    ignore_it.next();
                    continue;
                }
            }
            
            let signal = if s.is_main_public_input() {
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
            signals.push(Some(signal));
        }

        // register constrains
        for n in 0..self.constraints.len() {
            let constraint = self.constraints.get(n);
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

pub fn setup<W: Write>(
    asts : &Vec<BodyElementP>,
    signals: &Signals,
    constraints: &Constraints,
    ignore_signals : &Vec<SignalId>,
    out_pk: W,
) -> Result<(bellman::groth16::VerifyingKey<Bn256>, Vec<String>)> {

    let rng = &mut thread_rng();
    let circuit = CircomCircuit::<Bn256> {
        signals,
        ignore_signals,
        constraints,
        phantom: PhantomData,
    };

    // perform setup
    let start = SystemTime::now();
    let params = generate_random_parameters(circuit, rng)?;
    info!(
        "Setup time: {:?}",
        SystemTime::now().duration_since(start).unwrap()
    );
    let start = SystemTime::now();
    write_pk(out_pk, &asts, &constraints, &ignore_signals, &params)?;
    info!(
        "Proving key write time: {:?}",
        SystemTime::now().duration_since(start).unwrap()
    ); 

    let inputs = signals.main_public_input_names();

    Ok((params.vk, inputs))
}

pub fn generate_verified_proof<W: Write>(
    signals: &Signals,
    ignore_signals : &Vec<SignalId>,
    constraints : &Constraints,
    params : &Parameters<Bn256>,
    out_proof: &mut W,
) -> Result<Vec<(String, FS)>> {
    let rng = &mut thread_rng();

    let start = SystemTime::now();
    info!(
        "Proving key read time: {:?}",
        SystemTime::now().duration_since(start).unwrap()
    );

    let start = SystemTime::now();
    constraints.satisfies_with_signals(&signals).expect("check_constrains_eval_zero failed");
    info!(
        "Constraint check time: {:?} for {} constraint",
        SystemTime::now().duration_since(start).unwrap(),
        constraints.len()
    );

    let circuit = CircomCircuit::<Bn256> {
        signals,
        ignore_signals,
        constraints,
        phantom: PhantomData,
    };

    // Create proof
    let start = SystemTime::now();
    let proof = create_random_proof(circuit, params, rng).expect("cannot create proof");
    info!(
        "Proof generation time: {:?}",
        SystemTime::now().duration_since(start).unwrap()
    );

    // Self-verify and generate public inputs
    let start = SystemTime::now();
    let mut public_inputs = Vec::new();
    for i in 0..signals.len() {
        let signal = signals.get_by_id(i).unwrap();
        if signal.is_main_public_input() {
            let fs = (&*signal).clone().value.unwrap().try_into_fs().unwrap();
            let name = signal.full_name.0.to_string();
            public_inputs.push((name, fs));
        }
    }

    let vk = prepare_verifying_key(&params.vk);
    let verify_public_inputs = public_inputs
        .iter()
        .map(|(_, n)| {
            Fr::from_str(&(n.to_string())).expect(&format!("cannot parse fe {}", &n.to_string()))
        })
        .collect::<Vec<_>>();

    verify_proof(&vk, &proof, &verify_public_inputs)?;
    JsonProofAndInput::from_bellman(proof, public_inputs.clone())?.write(out_proof)?;
    info!(
        "Proof verification time: {:?}",
        SystemTime::now().duration_since(start).unwrap()
    );

    Ok(public_inputs)
}

#[cfg(test)]
mod test {
    use super::*;

    use bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };
    use za_compiler::algebra::Value;
    use za_compiler::evaluator::{Evaluator, Mode, Scope};
    use pairing::bn256::{Bn256, Fr};
    use rand::thread_rng;
    use std::fs::File;
    use std::marker::PhantomData;
    use super::super::format::read_pk;

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

        let mut ev_r1cs = Evaluator::new(
            Mode::GenConstraints,
            Signals::default(),
            Constraints::default(),
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
                ignore_signals: &Vec::new(),
                constraints: &ev_r1cs.constraints,
                phantom: PhantomData,
            };

            generate_random_parameters(circuit, rng).unwrap()
        };

        // Prepare the verification key (for proof verification)
        let pvk = prepare_verifying_key(&params.vk);

        // Compute witness
        let mut ev_witness = Evaluator::new(
            Mode::GenWitness,
            Signals::default(),
            Constraints::default(),
        );

        ev_witness.set_deferred_value("main.a".to_string(), Value::from(7));
        ev_witness.set_deferred_value("main.b".to_string(), Value::from(3));
        ev_witness
            .eval_inline(&mut Scope::new(true, None, "root".to_string()), circuit)
            .unwrap();

        // check constraints
        &ev_r1cs.constraints.satisfies_with_signals(&ev_witness.signals)
            .expect("cannot check all constraints = 0");

        println!("Creating proofs --------------------------------- ");
        let circuit = CircomCircuit::<Bn256> {
            signals: &ev_witness.signals,
            ignore_signals: &Vec::new(),
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
    fn test_setup_and_prove_pk() {

        let circuit = "
            template t() {
                signal private input a;
                signal private input b;  
                signal output c;

                c <== a * b;
            }
            component main = t();
        ";

        let mut ev_r1cs = Evaluator::new(
            Mode::GenConstraints,
            Signals::default(),
            Constraints::default(),
        );
        ev_r1cs
            .eval_inline(&mut Scope::new(true, None, "root".to_string()), circuit)
            .unwrap();

        // setup -----------------------------------------------------

        let pk = File::create("/tmp/pk").unwrap();
        let (_, _) = setup(
            &ev_r1cs.collected_asts,
            &ev_r1cs.signals,
            &ev_r1cs.constraints,
            &Vec::new(),
            pk
        ).expect("cannot setup");

        // Compute witness -------------------------------------------
        let pk = File::open("/tmp/pk").unwrap();
        let (pk_asts,pk_constraints, pk_ignore_signals, pk_params) = read_pk(pk).unwrap();
        let mut ev_witness = Evaluator::new(
            Mode::GenWitness,
            Signals::default(),
            Constraints::default(),
        );

        ev_witness.set_deferred_value("main.a".to_string(), Value::from(7));
        ev_witness.set_deferred_value("main.b".to_string(), Value::from(3));
        ev_witness
            .eval_asts(&pk_asts)
            .unwrap();

        ev_r1cs.constraints.satisfies_with_signals(&ev_witness.signals)
            .expect("cannot check internal evaluator constraints = 0");
        pk_constraints.satisfies_with_signals(&ev_witness.signals)
            .expect("cannot check optimized constraints = 0");

        // Create and verify proof
        let mut proof = Vec::new();
        let public_input = generate_verified_proof(
            &ev_witness.signals,
            &pk_ignore_signals,
            &pk_constraints,
            &pk_params,
            &mut proof).unwrap();

        assert_eq!("[(\"main.c\", 21)]", format!("{:?}", public_input));
    }


}
