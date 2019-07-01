use super::error::{Error,Result};
use super::report::dump_error;

use crate::evaluator::{Evaluator, Mode, ScopeValue};
use crate::storage::{Constraints, Signals, StorageFactory};
use crate::evaluator::{check_constrains_eval_zero};

pub fn run_embeeded_tests<F, S, C>(
    path: &str,
    filename: &str,
    mut factory: F,
    debug : bool,
) -> Result<Option<(Evaluator<S, C>, String)>>
where
    S: Signals,
    C: Constraints,
    F: StorageFactory<S, C>,
{
    let mut eval = Evaluator::new(
        Mode::Collect,
        factory.new_signals()?,
        factory.new_constraints()?,
    );

    match eval.eval_file(&path, &filename) {
        Ok(scan_scope) => {

        let tests = scan_scope
            .vars
            .borrow()
            .iter()
            .filter_map(|(k, v)| match v {
                ScopeValue::Template { attrs, .. } if attrs.has_tag_test() => Some(k),
                _ => None,
            })
            .map(|f| f.to_string())
            .collect::<Vec<_>>();

            for test_name in tests.iter() {

                println!("📏 Testing {} ",test_name);

                // Generate constraints
                println!("  ➡ Generating constraints");
                let mut ev_constraints = Evaluator::new(
                    Mode::GenConstraints,
                    factory.new_signals()?,
                    factory.new_constraints()?,
                );
                ev_constraints.debug = debug;
                if let Err(err) = ev_constraints.eval_template(&mut scan_scope.deep_clone(), &test_name) {
                    dump_error(&ev_constraints, &format!("{:?}",&err));
                    return Err(Error::Evaluator(err)); 
                }
                // Generate witness
                println!("➡ Generating witness");
                let mut ev_witness = Evaluator::new(
                    Mode::GenWitness,
                    factory.new_signals()?,
                    factory.new_constraints()?,
                );
                ev_witness.debug = debug;
                if let Err(err) = ev_witness.eval_template(&mut scan_scope.deep_clone(), &test_name) {
                    dump_error(&ev_constraints, &format!("{:?}",&err));
                    return Err(Error::Evaluator(err)); 
                }

                // Sanity check that the generated constrains are the same
                let wi_count = ev_witness.signals.len()?; 
                let cn_count = ev_constraints.signals.len()?;
                let ckeck_up_to = if wi_count < cn_count {
                    wi_count
                } else {
                    cn_count
                };
                
                for n in 1..ckeck_up_to {
                    let wi_signal = &*ev_witness.signals.get_by_id(n).unwrap().unwrap();
                    let cn_signal = &*ev_constraints.signals.get_by_id(n).unwrap().unwrap();
                    if wi_signal.full_name.0 != cn_signal.full_name.0 {
                        panic!(
                            "constrain & witness signals differ #cn(len={})={},#wi(len={})={}",
                            cn_count,
                            &cn_signal.full_name.0,
                            wi_count,
                            &wi_signal.full_name.0
                        );
                    }
                }

                if ev_constraints.signals.len()? != ev_witness.signals.len()? {
                        panic!(
                            "constrain & witness signals differ #cn(len={}),#wi(len={})",
                            cn_count,
                            wi_count
                        )
                }

                // Test constraints
                println!("➡  Testing {} constraints evals to zero", ev_constraints.constraints.len()?);
                check_constrains_eval_zero(&ev_constraints.constraints,&ev_witness.signals)?;   
            }
        }

        Err(err) => {
            dump_error(&eval, &format!("{:?}",&err));
        }
    }

    Ok(None)
}
