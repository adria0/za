mod cuda;

extern crate za_compiler;
extern crate za_parser;
extern crate za_prover;

extern crate codespan;
extern crate codespan_reporting;
extern crate stderrlog;
extern crate structopt;

#[macro_use]
extern crate log;

use za_compiler::tester::dump_error;
use za_compiler::types::{print_info, Constraints, Signals};
use za_compiler::{
    evaluator::{Evaluator, Mode},
    tester,
};
use za_prover::groth16;

use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;

const DEFAULT_CIRCUIT: &str = "circuit.circom";
const DEFAULT_PROVING_KEY: &str = "proving.key";
const DEFAULT_INPUT: &str = "input.json";
const DEFAULT_PROOF: &str = "proof.json";
const DEFAULT_VERIFIER_SOLIDITY: &str = "verifier.sol";
const DEFAULT_VERIFIER_JSON: &str = "verifier.json";
const VERIFIER_TYPE_SOLIDITY: &str = "solidity";
const VERIFIER_TYPE_JSON: &str = "json";
const DEFAULT_VERIFIER_TYPE: &str = VERIFIER_TYPE_SOLIDITY;

fn generate_cuda(constraints: &Constraints, signals: &Signals, cuda_file: Option<String>) {
    if let Some(cuda_file) = cuda_file {
        let start = SystemTime::now();
        cuda::export(&cuda_file, constraints, signals).unwrap();
        info!(
            "Cuda generation time: {:?}",
            SystemTime::now().duration_since(start).unwrap()
        );
    }
}

fn compile_ram(filename: &str, print_all: bool, cuda_file: Option<String>) {
    let mut start = SystemTime::now();
    let mut eval = Evaluator::new(
        Mode::GenConstraints,
        Signals::default(),
        Constraints::default(),
    );
    if let Err(err) = eval.eval_file(".", &filename) {
        dump_error(&eval, &format!("{:?}", err));
    } else {
        info!(
            "Compile time: {:?}",
            SystemTime::now().duration_since(start).unwrap()
        );
        start = SystemTime::now();

        let Evaluator {
            constraints,
            signals,
            ..
        } = eval;
        print_info("compile", &constraints, &signals, &[], print_all);

        let irreductible_signals = signals.main_input_ids();
        let (constraints, removed_signals) =
            za_compiler::optimizer::optimize(&constraints, &irreductible_signals);

        info!(
            "Optimization time: {:?}",
            SystemTime::now().duration_since(start).unwrap()
        );

        print_info(
            "optimized",
            &constraints,
            &signals,
            &removed_signals,
            print_all,
        );

        generate_cuda(&constraints, &signals, cuda_file);
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

#[derive(StructOpt, Debug)]
enum VerifierType {
    #[structopt(name = "json")]
    /// JSON with validation params
    JSON {},

    #[structopt(name = "solidity")]
    /// Solidity smartcontract
    Solidity {},
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "compile")]
    /// Only compile the circuit
    Compile {
        #[structopt(long = "circuit")]
        /// Input circuit, defaults to circuit.circom
        circuit: Option<String>,

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
        /// Input circuit, defaults to circuit.circom
        circuit: Option<String>,

        #[structopt(long = "pk")]
        /// Output proving key output file, defaults to prover.key
        pk: Option<String>,

        #[structopt(long = "verifier")]
        /// Output verifier file
        verifier_file: Option<String>,

        #[structopt(long = "verifiertype")]
        /// Verifier type, solidity (default) or json
        verifier_type: Option<String>,
    },

    #[structopt(name = "prove")]
    /// Generate a proof
    Prove {
        #[structopt(long = "pk")]
        /// Input proving key file, defaults to prover.key
        pk: Option<String>,

        #[structopt(long = "input")]
        /// Input inputs file, defaults to input.json
        input: Option<String>,

        #[structopt(long = "proof")]
        /// Ouput proof file, defaults to proof.json
        proof: Option<String>,
    },

    #[structopt(name = "test")]
    /// Run embeeded circuit tests
    Test {
        #[structopt(long = "circuit")]
        /// Input circuit, defaults to circuit.circom
        circuit: Option<String>,

        #[structopt(long = "debug")]
        /// Turn on debugging
        debug: bool,

        /// Genetate binary witness file
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

    za_prover::groth16::bellman_verbose(true);

    let cmd = Command::from_args();
    match cmd {
        Command::Compile {
            circuit,
            print,
            cuda,
        } => {
            let circuit = circuit.unwrap_or_else(|| DEFAULT_CIRCUIT.to_string());
            compile_ram(&circuit, print, cuda)
        }
        Command::Setup {
            circuit,
            pk,
            verifier_file,
            verifier_type,
        } => {
            let circuit = circuit.unwrap_or_else(|| DEFAULT_CIRCUIT.to_string());
            let pk = pk.unwrap_or_else(|| DEFAULT_PROVING_KEY.to_string());
            let verifier_type = match verifier_type
                .unwrap_or_else(|| DEFAULT_VERIFIER_TYPE.to_string())
                .as_ref()
            {
                VERIFIER_TYPE_JSON => groth16::helper::VerifierType::JSON,
                VERIFIER_TYPE_SOLIDITY => groth16::helper::VerifierType::Solidity,
                _ => panic!("unknown verifier type"),
            };
            let verifier_file = verifier_file.unwrap_or_else(|| {
                match verifier_type {
                    groth16::helper::VerifierType::Solidity => DEFAULT_VERIFIER_SOLIDITY,
                    groth16::helper::VerifierType::JSON => DEFAULT_VERIFIER_JSON,
                }
                .to_string()
            });
            let verifier = groth16::helper::setup(&circuit, &pk, verifier_type)
                .expect("unable to create proof");

            File::create(verifier_file)
                .expect("cannot create verifier file")
                .write_all(verifier.as_bytes())
                .expect("cannot write verifier file");
        }
        Command::Test {
            circuit,
            debug,
            outputwitness,
            skipcompile,
            prefix,
        } => {
            let circuit = circuit.unwrap_or_else(|| DEFAULT_CIRCUIT.to_string());
            let prefix = prefix.unwrap_or_else(|| "".to_string());
            match tester::run_embeeded_tests(
                ".",
                &circuit,
                debug,
                skipcompile,
                outputwitness,
                &prefix,
            ) {
                Ok(Some((eval, err))) => dump_error(&eval, &err),
                Err(err) => warn!("Error: {:?}", err),
                _ => {}
            }
        }
        Command::Prove { pk, input, proof } => {
            let pk_path = pk.unwrap_or_else(|| DEFAULT_PROVING_KEY.to_string());
            let input_path = input.unwrap_or_else(|| DEFAULT_INPUT.to_string());
            let proof_path = proof.unwrap_or_else(|| DEFAULT_PROOF.to_string());

            let mut inputs_json = String::new();
            File::open(input_path)
                .expect("cannot open inputs file")
                .read_to_string(&mut inputs_json)
                .expect("cannot read inputs file");

            let inputs = za_prover::groth16::flatten_json("main", &inputs_json)
                .expect("cannot parse inputs file");

            let proof = groth16::helper::prove(&pk_path, inputs).expect("cannot generate proof");

            File::create(proof_path)
                .expect("cannot create proof file")
                .write_all(proof.as_bytes())
                .expect("cannot write proof file");
        }
    }
}
