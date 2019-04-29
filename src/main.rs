extern crate circom2_parser;
extern crate codespan;
extern crate codespan_reporting;
#[macro_use]
extern crate lazy_static;
extern crate blake2_rfc;
extern crate hex;

pub mod evaluator;
pub mod algebra;

use std::env;

use codespan_reporting::termcolor::{StandardStream, ColorChoice};
use codespan::{CodeMap, Span, ByteSpan};
use codespan_reporting::{emit, Diagnostic, Label, Severity};

fn main() -> evaluator::Result<()> {
    let args : Vec<String> = env::args().collect();
    if args.len() == 2 {
        let mut eval = evaluator::Evaluator::default();
        if let Err(err) = eval.eval_file(&args[1]) {

            let span : ByteSpan= Span::from_offset(
                (eval.error_meta.start as u32).into(),
                ((eval.error_meta.end - eval.error_meta.start) as i64).into()
            );
            let msg = format!("{:?}",err);
            
            let mut code_map = CodeMap::new();
            code_map.add_filemap_from_disk(eval.error_file)
                .expect("cannot read source file");
                
            let error = Diagnostic::new(Severity::Error, "Failed to execute")
                .with_label(
                    Label::new_primary(span).with_message(msg)
                );
            
            let writer = StandardStream::stderr(ColorChoice::Always);
            emit(&mut writer.lock(), &code_map, &error).unwrap();

            println!("{}",eval.error_scope);

        } else {

           println!(
               "{} signals, {} constraints",
                eval.signals.len(),eval.constrains.len()
           );     

            //println!("{:?}",eval.signals);
            //println!("constrains ----------------------");
            //for constrain in eval.constrains {
            //    println!("  {:?}=0",constrain);
            //}
        }
    } else {
        println!("Usage: {} <file>",args[0]);
    }
    Ok(())
}
