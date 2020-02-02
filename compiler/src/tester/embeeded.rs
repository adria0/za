use std::fs::File;

use super::error::{Error, Result};
use super::report::dump_error;

use crate::algebra::FS;
use crate::evaluator::{Evaluator, Mode, ScopeValue};
use crate::types::{Constraints, Signals};

pub fn run_embeeded_tests(
    path: &str,
    filename: &str,
    debug: bool,
    skip_compile: bool,
    output_witness: bool,
    test_prefix: &str,
) -> Result<Option<(Evaluator, String)>> {
    let mut eval = Evaluator::new(Mode::Collect, Signals::default(), Constraints::default());

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
                .map(|template_name| template_name.to_string())
                .filter(|template_name| template_name.starts_with(test_prefix))
                .collect::<Vec<_>>();

            for test_name in tests.iter() {
                println!("📏 Testing {} ", test_name);

                // Generate witness
                println!("➡ Generating witness");
                let mut ev_witness =
                    Evaluator::new(Mode::GenWitness, Signals::default(), Constraints::default());
                ev_witness.debug = debug;
                if let Err(err) = ev_witness.eval_template(&mut scan_scope.clone(), &test_name) {
                    dump_error(&ev_witness, &format!("{:?}", &err));
                    return Err(Error::Evaluator(err));
                }

                if output_witness {
                    let mut witness_file = File::create(format!("./{}.binwitness", test_name))?;
                    let witness_len = ev_witness.signals.len();
                    FS::from(witness_len as u64).write_256_w32(&mut witness_file)?;
                    FS::from(1).write_256_w32(&mut witness_file)?;
                    for n in 1..witness_len {
                        let signal = &*ev_witness.signals.get_by_id(n).unwrap();
                        let value = signal.value.clone().unwrap().try_into_fs().unwrap();
                        value.write_256_w32(&mut witness_file)?;
                    }
                }

                if !skip_compile {
                    // Generate constraints
                    println!("  ➡ Generating constraints");
                    let mut ev_constraints = Evaluator::new(
                        Mode::GenConstraints,
                        Signals::default(),
                        Constraints::default(),
                    );
                    ev_constraints.debug = debug;
                    if let Err(err) =
                        ev_constraints.eval_template(&mut scan_scope.clone(), &test_name)
                    {
                        dump_error(&ev_constraints, &format!("{:?}", &err));
                        return Err(Error::Evaluator(err));
                    }

                    // Sanity check that the generated constrains are the same
                    let wi_count = ev_witness.signals.len();
                    let cn_count = ev_constraints.signals.len();
                    let ckeck_up_to = if wi_count < cn_count {
                        wi_count
                    } else {
                        cn_count
                    };

                    for n in 1..ckeck_up_to {
                        let wi_signal = &*ev_witness.signals.get_by_id(n).unwrap();
                        let cn_signal = &*ev_constraints.signals.get_by_id(n).unwrap();
                        if wi_signal.full_name.0 != cn_signal.full_name.0 {
                            panic!(
                                "constrain & witness signals differ #cn(len={})={},#wi(len={})={}",
                                cn_count, &cn_signal.full_name.0, wi_count, &wi_signal.full_name.0
                            );
                        }
                    }

                    if ev_constraints.signals.len() != ev_witness.signals.len() {
                        panic!(
                            "constrain & witness signals differ #cn(len={}),#wi(len={})",
                            cn_count, wi_count
                        )
                    }

                    // Test constraints
                    println!(
                        "➡  Testing {} constraints evals to zero",
                        ev_constraints.constraints.len()
                    );
                    ev_constraints
                        .constraints
                        .satisfies_with_signals(&ev_witness.signals)
                        .map_err(Error::Unexpected)?;
                }
            }
        }

        Err(err) => {
            dump_error(&eval, &format!("{:?}", &err));
        }
    }

    Ok(None)
}
