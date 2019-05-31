use circom2_parser::ast::SignalType;
use circom2_compiler::algebra::{LC, QEQ};
use circom2_compiler::storage::{Constraints, Signals,Result};

use circom2_compiler::storage::{
    Constraints,
    Signals,
    Result
};

// For randomness (during paramgen and proof generation)
use self::rand::{thread_rng};

// Bring in some tools for using pairing-friendly curves
use self::pairing::{
    Engine,
    Field,
    PrimeField
};

// We're going to use the BLS12-381 pairing-friendly elliptic curve.
use self::pairing::bls12_381::{
    Bls12,
    Fr,
};

// We'll use these interfaces to construct our circuit.
use self::bellman::{
    Circuit,
    ConstraintSystem,
    SynthesisError
};

// We're going to use the Groth16 proving system.
use self::bellman::groth16::{
    Proof,
    generate_random_parameters,
    prepare_verifying_key,
    create_random_proof,
    verify_proof,
};

pub<'a> struct CircomCircuit<E: Engine> {
    constraints: &'a Constraints,
    signals: &'a Signals,
}

fn<E:Engine> lc_circom_to_bellman<E>(lc :&LC,&[]) -> E::Fr {

}

impl <E: Engine> Circuit<E> for CircomCircuit<E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self, 
        cs: &mut CS
    ) -> Result<(), SynthesisError>
    {
        let mut signals = Vec::new();

        for n in 0..self.signals.len()? {
            let s = self.signals.get_by_id(n)?.unwrap();
            let component_len = s.full_name.0.chars().filter(|ch| *ch == '.').count();
            let signal = match (component_len,s.xtype) {
                (1, SignalType::Output)| (1,SignalType::PublicInput) => {
                    cs.alloc_input(|| s.full_name, || SynthesisError::AssignmentMissing )?
                },
                _ => {
                    cs.alloc(|| s.full_name, || SynthesisError::AssignmentMissing )?                        
                },
            };
            signals.push(signal);
        }





        // a * b = c?
        cs.enforce(
            || "mult",
            |lc| lc + a,
            |lc| lc + b,
            |lc| lc + c
        );
        
        Ok(())
    }
}