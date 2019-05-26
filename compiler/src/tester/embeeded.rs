use super::error::Result;
use crate::evaluator::{Evaluator, Mode, ScopeValue};
use crate::storage::{Constraints, Signals, StorageFactory};

pub fn run_embeeded_tests<F, S, C>(
    path: &str,
    filename: &str,
    mut factory: F,
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
    let scan_scope = eval.eval_file(&path, &filename);

    let tests = match &scan_scope {
        Ok(scope) => scope
            .vars
            .borrow()
            .iter()
            .filter_map(|(k, v)| match v {
                ScopeValue::Template { attrs, .. } if attrs.has_tag_test() => Some(k),
                _ => None,
            })
            .map(|f| f.to_string())
            .collect::<Vec<_>>(),
        Err(err) => {
            return Ok(Some((eval, format!("{:?}", err))));
        }
    };

    let mut scan_scope = scan_scope.unwrap();

    for test_name in tests.iter() {
        println!("Generating witness for {}", test_name);
        let code = format!("component test_{}={}();", test_name, test_name);
        let mut eval = Evaluator::new(
            Mode::GenWitness,
            factory.new_signals()?,
            factory.new_constraints()?,
        );
        if let Err(err) = &eval.eval_inline(&mut scan_scope, &code) {
            return Ok(Some((eval, format!("{:?}", err))));
        }
        println!("Testing witness for {}", test_name);
    }

    Ok(None)
}
