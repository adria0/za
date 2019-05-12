use crate::evaluator::{Evaluator,Mode,ScopeValue,Error};

pub fn run_embeeded_test(path: &str, filename : &str) -> Result<(),(Evaluator,Error)> {   
    let mut eval = Evaluator::new(Mode::Collect);
    let scan_scope = eval.eval_file(&path, &filename);
    
    let tests = match &scan_scope {
        Ok(scope) => {
            let vars = scope.vars.borrow();
            let tests = vars.iter()
                .filter_map( |(k,v)|
                    match v  {
                        ScopeValue::Template(attrs,_,_,_) if attrs.has_tag_test() => Some(k),
                         _  => None
                    }
                )
                .map ( |f| f.to_string() )
                .collect::<Vec<_>>();
            tests
        },
        Err(err) => {
            return Err((eval,err.clone()));
        }
    };

    let mut scan_scope = scan_scope.unwrap();

    for test_name in tests.iter() {
        println!("Testing {}",test_name);
        let code = format!("component test_{}={}();",test_name,test_name);        
        let mut eval = Evaluator::new(Mode::GenWitness);
        if let Err(err) = &eval.eval_inline(&mut scan_scope, &code) {
            return Err((eval,err.clone()));
        }
    }
    Ok(())    
}
