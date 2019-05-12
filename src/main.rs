extern crate circom2_parser;
extern crate circom2_compiler;
extern crate codespan;
extern crate codespan_reporting;

use std::env;

use circom2_compiler::{evaluator,tester};
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
    if let Err(err) = eval.eval_file(".",&filename) {
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

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() == 3 {
        if args[1] ==  "c" {
            generate_constrains(&args[2]);
        } else if args[1] == "t" {
            if let Err((eval,err)) = tester::run_embeeded_test(".",&args[2]) {
                dump_error(&eval,&err);
            }
        } else {
            panic!("Invald parameter");
        }
    } else {
        println!("Usage: {} [c|t] <file>",args[0]);
    }
}