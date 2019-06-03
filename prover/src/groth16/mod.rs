use circom2_compiler::algebra::{Value, FS, LC};
use circom2_compiler::storage;
use circom2_compiler::storage::{Constraints, Signals};
use circom2_parser::ast::SignalType;
use std::marker::PhantomData;

use pairing::Engine;

use bellman::{Circuit, ConstraintSystem, LinearCombination, SynthesisError};

use ff::PrimeField;

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
        Err(storage::Error::Io(io)) => Err(SynthesisError::IoError(io)),
        _ => Err(SynthesisError::Unsatisfiable), // ???
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
            let component_len = s.full_name.0.chars().filter(|ch| *ch == '.').count();
            let signal = match (component_len, &s.xtype, &s.value) {
                // public signals
                (1, SignalType::Output, v) | (1, SignalType::PublicInput, v) => cs.alloc_input(
                    || (*s.full_name.0).clone(),
                    || {
                        v.clone()
                            .map(|vv| value_to_bellman::<E>(&vv))
                            .ok_or(SynthesisError::AssignmentMissing)
                    },
                )?,
                // private signals
                (_, _, v) => cs.alloc(
                    || (*s.full_name.0).clone(),
                    || {
                        v.clone()
                            .map(|vv| value_to_bellman::<E>(&vv))
                            .ok_or(SynthesisError::AssignmentMissing)
                    },
                )?,
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

#[cfg(test)]
mod test {
    use super::*;

    use circom2_compiler::evaluator::{Evaluator, Mode, Scope};
    use circom2_compiler::storage::Ram;
    use circom2_compiler::storage::StorageFactory;
    use pairing::bn256::{Bn256, Fr};
    use rand::thread_rng;

    use bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };

    use std::marker::PhantomData;

    #[test]
    fn test_simple() {
        let circuit = "
            template t() {
                signal private input a;
                signal private input b;  
                signal input c;

                a * b === c;  
            }
            component main = t();
        ";

        let mut ram = Ram::default();
        let mut evaluator = Evaluator::new(
            Mode::GenConstraints,
            ram.new_signals().unwrap(),
            ram.new_constraints().unwrap(),
        );

        let mut scope = Scope::new(true, None, "root".to_string());
        evaluator.eval_inline(&mut scope, circuit).unwrap();

        let rng = &mut thread_rng();

        // Create parameters for our circuit
        println!("Run setup...");
        let params = {
            let circuit = CircomCircuit::<Bn256> {
                signals: &evaluator.signals,
                constraints: &evaluator.constraints,
                phantom: PhantomData,
            };

            generate_random_parameters(circuit, rng).unwrap()
        };

        // Prepare the verification key (for proof verification)
        let pvk = prepare_verifying_key(&params.vk);

        println!("Creating proofs...");
        let mut set_witness = |name, value: u64| {
            let id = evaluator.signals.get_by_name(name).unwrap().unwrap().id;
            evaluator
                .signals
                .update(id, Value::FieldScalar(FS::from(value)))
                .unwrap();
        };

        // Set witness
        set_witness("main.a", 7);
        set_witness("main.b", 3);
        set_witness("main.c", 21);

        let circuit = CircomCircuit::<Bn256> {
            signals: &evaluator.signals,
            constraints: &evaluator.constraints,
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
}
