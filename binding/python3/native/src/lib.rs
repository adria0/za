#[macro_use] extern crate cpython;
extern crate circom2_prover;

use cpython::{PyErr, PyString, PyResult, Python,exc};

py_module_initializer!(libcircom2py, initlibcircom2py, PyInit_libcircom2py, |py, m| {
    m.add(py, "__doc__", "rust-circom pyhon3 library")?;
    m.add(py, "verbose", py_fn!(py, verbose_py(on: bool)))?;
    m.add(py, "setup",   py_fn!(py, setup_py(circuit_path : &str, pk_path : &str, verifier_type : &str)))?;
    m.add(py, "prove",   py_fn!(py, prove_py(circuit_path : &str, pk_path: &str, inputs: &str)))?;
    m.add(py, "verify",  py_fn!(py, verify_py(verifying_key : &str, proof_with_inputs : &str)))?;
    Ok(())
});

fn verbose_py(_: Python, on : bool) -> PyResult<bool>{
   if on {  
        circom2_prover::groth16::bellman_verbose(true);
        stderrlog::new()
            .verbosity(2)
            .timestamp(stderrlog::Timestamp::Off)
            .init()
            .unwrap();
   } else {
        circom2_prover::groth16::bellman_verbose(false);
        stderrlog::new()
            .quiet(true)
            .init()
            .unwrap();
   }
   Ok(on)
}

fn setup_py(py: Python, circuit_path : &str, pk_path : &str, verifier_type : &str) -> PyResult<String> {

    let verifier_type = match verifier_type {
        "json" => circom2_prover::groth16::VerifierType::JSON,
        "solidity" => circom2_prover::groth16::VerifierType::Solidity,
        _ => return Err(PyErr::new::<exc::TypeError, _>(py, PyString::new(py,"invalid verifier type")))
    };
    
    circom2_prover::groth16::setup_ram(&circuit_path,&pk_path,verifier_type)
        .map_err(|err| PyErr::new::<exc::TypeError, _>(py, format!("{:?}",err)))
}

fn prove_py(py: Python, circuit_path : &str, pk_path: &str, inputs: &str) -> PyResult<String> {
    circom2_prover::groth16::flatten_json("main",&inputs)
        .and_then(|inputs| circom2_prover::groth16::prove_ram(&circuit_path,&pk_path,inputs))
        .map_err(|err| PyErr::new::<exc::TypeError, _>(py, format!("{:?}",err)))
}

fn verify_py(py: Python, verifying_key : &str, proof_with_inputs : &str) -> PyResult<bool> {
    circom2_prover::groth16::verify_ram(&verifying_key,&proof_with_inputs)
        .map_err(|err| PyErr::new::<exc::TypeError, _>(py, format!("{:?}",err)))
}
