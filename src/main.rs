extern crate circom2_parser;
extern crate circom2_compiler; 
extern crate codespan;
extern crate codespan_reporting;
extern crate structopt;
extern crate stderrlog;

#[macro_use]
extern crate log;

use circom2_compiler::{evaluator,tester};
use codespan_reporting::termcolor::{StandardStream, ColorChoice};
use codespan::{CodeMap, Span, ByteSpan};
use codespan_reporting::{emit, Diagnostic, Label, Severity};
use std::time::{SystemTime, UNIX_EPOCH};

use circom2_compiler::storage::{Signals,Constraints};
use circom2_compiler::storage::{Ram,Rocks,StorageFactory};

fn dump_error<S:Signals,C:Constraints>(eval : &evaluator::Evaluator<S,C>, err : &str) {

    let msg = format!("{:?}",err);

    if let Some(ctx) = &eval.last_error {

        let span : ByteSpan= Span::from_offset(
            (1+ctx.meta.start as u32).into(),
            (1+(ctx.meta.end - ctx.meta.start) as i64).into()
        );

        println!("{}",ctx.scope);

        if ctx.file != "" {    
            let mut code_map = CodeMap::new();
            code_map.add_filemap_from_disk(&ctx.file)
                .unwrap_or_else(|_| panic!("cannot read source file '{}'",&ctx.file));
                
            let error = Diagnostic::new(Severity::Error, "Failed to execute")
                .with_label(
                    Label::new_primary(span).with_message(msg.clone())
                );
            
            let writer = StandardStream::stderr(ColorChoice::Always);
            emit(&mut writer.lock(), &code_map, &error).unwrap();
        }  
    } 

}

fn generate_constrains_rocks(filename : &str) {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut storage = Rocks::new(format!("db_{}_{}",filename,since_the_epoch));

    let mut eval = evaluator::Evaluator::new(
        evaluator::Mode::GenConstraints,
        storage.new_signals().unwrap(),
        storage.new_constraints().unwrap()
    );
    if let Err(err) = eval.eval_file(".",&filename) {
        dump_error(&eval, &format!("{:?}",err));
    } else {
        info!(
            "{} signals, {} constraints",
            eval.signals.len().unwrap(),eval.constraints.len().unwrap()
        );     
        // print constraints
        //println!("{:?}",eval.signals);
        //println!("constrains ----------------------");
        //for constrain in eval.constrains {
        //    println!("  {:?}=0",constrain);
        //}
    }
}

fn generate_constrains_ram(filename : &str) {
    let mut storage = Ram::default();

    let mut eval = evaluator::Evaluator::new(
        evaluator::Mode::GenConstraints,
        storage.new_signals().unwrap(),
        storage.new_constraints().unwrap()
    );
    if let Err(err) = eval.eval_file(".",&filename) {
        dump_error(&eval, &format!("{:?}",err));
    } else {
        info!(
            "{} signals, {} constraints",
            eval.signals.len().unwrap(),eval.constraints.len().unwrap()
        );     
        // print constraints
        //println!("{:?}",eval.signals);
        //println!("constrains ----------------------");
        //for constrain in eval.constrains {
        //    println!("  {:?}=0",constrain);
        //}
    }
}


use structopt::StructOpt;

/// A StructOpt example
#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "t", long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,

    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "cfg", long = "cfg")]
    cfg: String,

}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "compile")]
    /// Compile the circuit
    Compile {
        file: String,
        
        #[structopt(long = "ram")]
        /// Use RAM (default) or local storage 
        use_ram: Option<bool>,
    },
    #[structopt(name = "test")]
    /// Run embeeded circuit tests
    Test {
        file : String,
    },
}

fn main() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Off)
        .init()
    .unwrap();

    let cmd = Command::from_args();
    match cmd {
        Command::Compile{file,use_ram} => {
            let use_ram = use_ram.unwrap_or(true);
            if use_ram {
                generate_constrains_ram(&file)
            } else {
                generate_constrains_rocks(&file)               
            }
        }
        Command::Test{file} => {
            let ram = Ram::default();
            match tester::run_embeeded_tests(".",&file,ram) {
                Ok(Some((eval,err))) => dump_error(&eval,&err),
                Err(err) => warn!("Error: {:?}",err),
                _ => {}
            }
        }
    }
}