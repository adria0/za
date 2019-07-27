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
    tester
};

use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::prelude::*;

use circom2_compiler::storage::{Constraints, Signals};
use circom2_compiler::storage::{Ram, StorageFactory};
use circom2_compiler::tester::dump_error;
use circom2_compiler::evaluator::{print_info};

use circom2_bigsnark::Rocks;

const DEFAULT_CIRCUIT : &str = "circuit.circom";
const DEFAULT_PROVING_KEY : &str = "proving.key";
const DEFAULT_INPUT : &str = "input.json";
const DEFAULT_PROOF : &str = "proof.json";
const DEFAULT_SOLIDITY_VERIFIER : &str = "verifier.sol";

fn generate_cuda<S:Signals,C:Constraints>(eval : &Evaluator<S,C>, cuda_file : Option<String>) {
    if let Some(cuda_file) = cuda_file {
        let start = SystemTime::now();
        circom2_prover::cuda::export_r1cs(&cuda_file, &eval.constraints, &eval.signals).unwrap();
        info!("Cuda generation time: {:?}",SystemTime::now().duration_since(start).unwrap());
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
        info!("Compile time: {:?}",SystemTime::now().duration_since(start).unwrap());
        generate_cuda(&eval,cuda_file);
        print_info(&eval, print_all);
    }
}

fn compile_ram(filename: &str, print_all: bool, cuda_file: Option<String>) {
    let mut storage = Ram::default();

    let start = SystemTime::now();
    let mut eval = Evaluator::new(
        Mode::GenConstraints,
        storage.new_signals().unwrap(),
        storage.new_constraints().unwrap(),
    );
    if let Err(err) = eval.eval_file(".", &filename) {
        dump_error(&eval, &format!("{:?}", err));
    } else {
        info!("Compile time: {:?}",SystemTime::now().duration_since(start).unwrap());
        generate_cuda(&eval,cuda_file);
        print_info(&eval, print_all);
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
    /// Only compile the circuit
    Compile {
        #[structopt(long = "circuit")]
        /// Circuit, defaults to circuit.circom
        circuit: Option<String>,

        #[structopt(long = "disk")]
        /// Use RAM (default) or local storage
        disk: bool,

        #[structopt(long = "print")]
        /// Print constaints and signals
        print: bool,

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
        debug: bool,

        /// Dump witness
        #[structopt(long = "outputwitness")]
        outputwitness: bool,

        /// Skip circuit compilation
        #[structopt(long = "skipcompile")]
        skipcompile: bool,

        /// Prefix of the tests to execute
        #[structopt(long = "prefix")]
        prefix: Option<String>,

    },
}

fn main() {
    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    circom2_prover::groth16::bellman_verbose(true);

    let cmd = Command::from_args();
    match cmd {
        Command::Compile { circuit, disk, print, cuda } => {
            let circuit = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            if disk {
                compile_rocks(&circuit,print, cuda)
            } else {
                compile_ram(&circuit,print,cuda)
            }
        }
        Command::Setup { circuit, pk, verifier } => {
            let circuit = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            let pk = pk.unwrap_or(DEFAULT_PROVING_KEY.to_string());
            let verifier = verifier.unwrap_or(DEFAULT_SOLIDITY_VERIFIER.to_string());
            circom2_prover::groth16::setup_ram(&circuit,&pk,&verifier)
                .expect("unable to create proof");
        }
        Command::Test { circuit, debug, outputwitness, skipcompile, prefix } => {
            let circuit = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            let prefix = prefix.unwrap_or("".to_string());
            let ram = Ram::default();
            match tester::run_embeeded_tests(".", &circuit, ram, debug, skipcompile, outputwitness, &prefix) {
                Ok(Some((eval, err))) => dump_error(&eval, &err),
                Err(err) => warn!("Error: {:?}", err),
                _ => {}
            }
        }
        Command::Prove { circuit, pk, input, proof } => {
            let circuit_path = circuit.unwrap_or(DEFAULT_CIRCUIT.to_string());
            let pk_path = pk.unwrap_or(DEFAULT_PROVING_KEY.to_string());
            let input_path = input.unwrap_or(DEFAULT_INPUT.to_string());
            let proof_path = proof.unwrap_or(DEFAULT_PROOF.to_string());

            let mut inputs_json = String::new();
            File::open(input_path)
                .expect("cannot open inputs file")
                .read_to_string(&mut inputs_json)
                .expect("cannot read inputs file");
            
            let inputs = circom2_prover::groth16::flatten_json("main", &inputs_json)
                .expect("cannot parse inputs file");

            let proof = circom2_prover::groth16::prove_ram(&circuit_path,&pk_path,inputs)
                .expect("cannot generate proof");

            File::create(proof_path)
                .expect("cannot create proof file")
                .write_all(proof.as_bytes())
                .expect("cannot write proof file");
        }
    }
}
