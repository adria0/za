extern crate circom2_parser;
extern crate codespan;
extern crate codespan_reporting;
#[macro_use]
extern crate lazy_static;
extern crate blake2_rfc;
extern crate hex;

pub mod evaluator;
pub mod algebra;
pub mod optimizer;

use std::env;

use evaluator::Mode;
use codespan_reporting::termcolor::{StandardStream, ColorChoice};
use codespan::{CodeMap, Span, ByteSpan};
use codespan_reporting::{emit, Diagnostic, Label, Severity};

fn dump_error(eval : &evaluator::Evaluator, err : &evaluator::Error) {

    let msg = format!("{:?}",err);

    println!("ERROR: {}",msg);

    if let Some(ctx) = &eval.last_error {

        let span : ByteSpan= Span::from_offset(
            (1+ctx.meta.start as u32).into(),
            (1+(ctx.meta.end - ctx.meta.start) as i64).into()
        );

        //println!("{}",ctx.scope);

        if ctx.file != "" {    
            let mut code_map = CodeMap::new();
            code_map.add_filemap_from_disk(&ctx.file)
                .expect(&format!("cannot read source file '{}'",&ctx.file));
                
            let error = Diagnostic::new(Severity::Error, "Failed to execute")
                .with_label(
                    Label::new_primary(span).with_message(msg.clone())
                );
            
            let writer = StandardStream::stderr(ColorChoice::Always);
            emit(&mut writer.lock(), &code_map, &error).unwrap();
        }  
    } 

}

fn generate_constrains(filename : &str) {
    let mut eval = evaluator::Evaluator::new(evaluator::Mode::GenConstraints);
    if let Err(err) = eval.eval_file(&filename) {
        dump_error(&eval, &err);
    } else {
        println!(
            "{} signals, {} constraints",
            eval.signals.len(),eval.constrains.len()
        );     
        // print constraints
        //println!("{:?}",eval.signals);
        //println!("constrains ----------------------");
        //for constrain in eval.constrains {
        //    println!("  {:?}=0",constrain);
        //}
    }
}

fn run_tests(filename : &str) {   
    let mut eval = evaluator::Evaluator::new(Mode::Collect);
    let scan_scope = eval.eval_file(&filename);
    
    let tests = match &scan_scope {
        Ok(scope) => {
            let vars = scope.vars.borrow();
            let tests = vars.iter()
                .filter_map( |(k,v)|
                    match v  {
                        evaluator::ScopeValue::Template(attrs,_,_,_) if attrs.has_tag_test() => Some(k),
                         _  => None
                    }
                )
                .map ( |f| f.to_string() )
                .collect::<Vec<_>>();
            tests
        },
        Err(err) => {
            dump_error(&eval,err);
            return;
        }
    };

    let mut scan_scope = scan_scope.unwrap();

    for test_name in tests.iter() {
        let code = format!("component test_{}={}();",test_name,test_name);        
        let mut eval = evaluator::Evaluator::new(Mode::GenWitness);
        if let Err(err) = &eval.eval_inline(&mut scan_scope, &code) {
            println!("FAILED {}",&test_name);
            //println!("{:?}",eval.signals);
            dump_error(&eval,&err);
        } else {
            println!("SUCCESS {}",&test_name);
        }
    }    
}

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() == 3 {
        if args[1] ==  "c" {
            generate_constrains(&args[2]);
        } else if args[1] == "t" {
            run_tests(&args[2]);
        } else {
            panic!("Invald parameter");
        }
    } else {
        println!("Usage: {} <file>",args[0]);
    }
}