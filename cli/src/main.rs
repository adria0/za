extern crate circom2_compiler;
extern crate circom2_parser;
extern crate circom2_prover;

extern crate codespan;
extern crate codespan_reporting;
extern crate stderrlog;
extern crate structopt;

#[macro_use]
extern crate log;

use circom2_compiler::{
    evaluator::{Evaluator,Mode},
    tester,
    algebra::Value
};

use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::prelude::*;

use circom2_compiler::storage::{Constraints, Signals};
use circom2_compiler::storage::{Ram, StorageFactory};
use circom2_compiler::tester::dump_error;
use circom2_compiler::evaluator::format_algebra;

use circom2_bigsnark::Rocks;

const DEFAULT_CIRCUIT : &str = "circuit.circom";
const DEFAULT_PROVING_KEY : &str = "proving.key";
const DEFAULT_INPUT : &str = "input.json";
const DEFAULT_PROOF : &str = "proof.json";
const DEFAULT_SOLIDITY_VERIFIER : &str = "verifier.sol";


fn print_info<S:Signals,C:Constraints>(eval : &Evaluator<S,C>, print_all: bool) {
    info!(
        "{} signals, {} constraints",
        eval.signals.len().unwrap(),
        eval.constraints.len().unwrap()
    );
    if print_all {
        println!("signals -------------------------");
        for n in 0..eval.signals.len().unwrap() {
            println!("{}: {:?}",n,eval.signals.get_by_id(n).unwrap());
        }
        println!("constrains ----------------------");
        for n in 0..eval.constraints.len().unwrap() {
            let constrain = Value::QuadraticEquation(eval.constraints.get(n).unwrap());
            println!("{}:  {}=0",n,format_algebra(&eval.signals,&constrain));
        }
    }
}

fn generate_cuda<S:Signals,C:Constraints>(eval : &Evaluator<S,C>, cuda_file : Option<String>) {
    if let Some(cuda_file) = cuda_file {
        circom2_prover::cuda::export_r1cs(&cuda_file, &eval.constraints, &eval.signals).unwrap();
    }
}

fn compile_rocks(filename: &str, print_all: bool, cuda_file: Option<String>) {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut storage = Rocks::new(format!("db_{}_{}", filename, since_the_epoch));

    let mut eval = Evaluator::new(
        Mode::GenConstraints,
        storage.new_signals().unwrap(),
        storage.new_constraints().unwrap(),
    );
    if let Err(err) = eval.eval_file(".", &filename) {
        dump_error(&eval, &format!("{:?}", err));
    } else {
        generate_cuda(&eval,cuda_file);
        print_info(&eval, print_all);
    }
}

fn compile_ram(filename: &str, print_all: bool, cuda_file: Option<String>) {
    let mut storage = Ram::default();

    let mut eval = Evaluator::new(
        Mode::GenConstraints,
        storage.new_signals().unwrap(),
        storage.new_constraints().unwrap(),
    );
    if let Err(err) = eval.eval_file(".", &filename) {
        dump_error(&eval, &format!("{:?}", err));
    } else {
        generate_cuda(&eval,cuda_file);
        print_info(&eval, print_all);
    }
}

fn setup_ram(circuit_path: &str, proving_key_path: &str, verificator_key_path: &str) {
    let mut storage = Ram::default();

    let mut eval = Evaluator::new(
        Mode::GenConstraints,
        storage.new_signals().unwrap(),
        storage.new_constraints().unwrap(),
    );
    info!("Compiling circuit...");
    if let Err(err) = eval.eval_file(".", &circuit_path) {
        dump_error(&eval, &format!("{:?}", err));
        return;
    } 
    print_info(&eval,false);
    info!("Running setup");
    let (pk,vk) = (
        File::create(proving_key_path).unwrap(),
        File::create(verificator_key_path).unwrap()
    );

    circom2_prover::groth16::setup(&eval, pk, vk).expect("cannot generate setup");
}

fn prove_ram(circuit_path: &str,proving_key_path: &str, input_path: &str, proof_path: &str) {

    let mut inputs = String::new();
    File::open(input_path)
        .expect("cannot open inputs file")
        .read_to_string(&mut inputs)
        .expect("cannot read inputs file");

    info!("Parsing inputs...");
    let inputs = circom2_prover::groth16::flatten_json("main",&inputs)
        .expect("cannot parse input");
    
    info!("Generating witness...");
    let mut ram = Ram::default();
    let mut ev_witness = Evaluator::new(
        Mode::GenWitness,
        ram.new_signals().unwrap(),
        ram.new_constraints().unwrap(),
    );

    info!("Checking constraints...");
    if ev_witness.constraints.len().unwrap() > 0 {
        panic!("constrains generated in witness");
    }

    info!("Checking signals...");
    for n in 1..ev_witness.signals.len().unwrap() {
        let signal = &*ev_witness.signals.get_by_id(n).unwrap().unwrap();
        if signal.value.is_none() {
            panic!("signal '{}' value is not defined",signal.full_name.0);
        }  
    }

    for (signal,value) in inputs {
        ev_witness.set_deferred_value(signal, Value::from(value));
    }

    ev_witness.eval_file(".", &circuit_path)
        .expect("cannot evaluate circuit");

    // Create proof
    info!("Creating and self-verifying proof...");
    let pk = File::open(proving_key_path)
        .expect("cannot read proving key");

    let mut proof = File::create(proof_path)
        .expect("cannot create proof file");

    let _ = circom2_prover::groth16::generate_verified_proof(
        ev_witness.signals,
        pk,
        &mut proof
    ).expect("cannot generate and self-verify proof");

    info!("Done.");
        
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
    /// Only compile the circuit
    Compile {
        #[structopt(long = "circuit")]
        /// Circuit, defaults to circuit.circom
        circuit: Option<String>,

        #[structopt(long = "ram")]
        /// Use RAM (default) or local storage
        use_ram: Option<bool>,

        #[structopt(long = "print")]
        /// Print constaints and signals
        print: Option<bool>,

        #[structopt(long = "cuda")]
        /// Export cuda format
        cuda: Option<String>,
    },
    #[structopt(name = "setup")]
    /// Compile & generate trusted setup
    Setup {
        #[structopt(long = "circuit")]
        /// Circuit, defaults to circuit.circom
        circuit: Option<String>,

        #[structopt(long = "pk")]
        /// Proving key file, defaults to prover.key
        pk: Option<String>,

        #[structopt(long = "verifier")]
        /// Solidity verifier
        verifier: Option<String>,
    },
    #[structopt(name = "prove")]
    /// Compile & generate trusted setup
    Prove {
        #[structopt(long = "circuit")]
        /// Circuit, defaults to circuit.circom
        circuit: Option<String>,

        #[structopt(long = "pk")]
        /// Proving key file, defaults to prover.key
        pk: Option<String>,

        #[structopt(long = "input")]
        /// Public inputs file, defaults to input.json
        input: Option<String>,

        #[structopt(long = "proof")]
        /// Proof file, defaults to proof.json
        proof: Option<String>,
    },
    #[structopt(name = "test")]
    /// Run embeeded circuit tests
    Test {
        #[structopt(long = "circuit")]
        /// Circuit, defaults to circuit.circom
        circuit: Option<String>,

        #[structopt(long = "debug")]
        /// Turn on debugging
        debug: Option<bool>,
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
        Command::Compile { circuit, use_ram, print, cuda } => {
            let circuit = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            let use_ram = use_ram.unwrap_or(true);
            let print_all = print.unwrap_or(false);
            if use_ram {
                compile_ram(&circuit,print_all,cuda)
            } else {
                compile_rocks(&circuit,print_all, cuda)
            }
        }
        Command::Setup { circuit, pk, verifier } => {
            let circuit = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            let pk = pk.unwrap_or(DEFAULT_PROVING_KEY.to_string());
            let verifier = verifier.unwrap_or(DEFAULT_SOLIDITY_VERIFIER.to_string());
            setup_ram(&circuit,&pk,&verifier);
        }
        Command::Test { circuit, debug } => {
            let circuit = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            let debug = debug.unwrap_or(false);
            let ram = Ram::default();
            match tester::run_embeeded_tests(".", &circuit, ram, debug) {
                Ok(Some((eval, err))) => dump_error(&eval, &err),
                Err(err) => warn!("Error: {:?}", err),
                _ => {}
            }
        }
        Command::Prove { circuit, pk, input, proof } => {
            let circuit = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            let pk = pk.unwrap_or(DEFAULT_PROVING_KEY.to_string());
            let input = input.unwrap_or(DEFAULT_INPUT.to_string());
            let proof = proof.unwrap_or(DEFAULT_PROOF.to_string());
            prove_ram(&circuit,&pk,&input,&proof)
        }
    }
}
