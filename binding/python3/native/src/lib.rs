#[macro_use] extern crate cpython;
extern crate za_prover;

use za_prover::groth16::helper;
use za_prover::groth16;

use cpython::{PyErr, PyString, PyResult, Python,exc};

py_module_initializer!(libza2py, initlibza2py, PyInit_libza2py, |py, m| {
    m.add(py, "__doc__", "za pyhon3 library")?;
    m.add(py, "verbose", py_fn!(py, verbose_py(on: bool)))?;
    m.add(py, "setup",   py_fn!(py, setup_py(circuit_path : &str, pk_path : &str, verifier_type : &str)))?;
    m.add(py, "prove",   py_fn!(py, prove_py(pk_path: &str, inputs: &str)))?;
    m.add(py, "verify",  py_fn!(py, verify_py(verifying_key : &str, proof_with_inputs : &str)))?;
    Ok(())
});

fn verbose_py(_: Python, on : bool) -> PyResult<bool>{
   if on {  
        za_prover::groth16::bellman_verbose(true);
        stderrlog::new()
            .verbosity(2)
            .timestamp(stderrlog::Timestamp::Off)
            .init()
            .unwrap();
   } else {
        za_prover::groth16::bellman_verbose(false);
        stderrlog::new()
            .quiet(true)
            .init()
            .unwrap();
   }
   Ok(on)
}

fn setup_py(py: Python, circuit_path : &str, pk_path : &str, verifier_type : &str) -> PyResult<String> {

    let verifier_type = match verifier_type {
        "json" => helper::VerifierType::JSON,
        "solidity" => helper::VerifierType::Solidity,
        _ => return Err(PyErr::new::<exc::TypeError, _>(py, PyString::new(py,"invalid verifier type")))
    };
    
    helper::setup(&circuit_path,&pk_path,verifier_type)
        .map_err(|err| PyErr::new::<exc::TypeError, _>(py, format!("{:?}",err)))
}

fn prove_py(py: Python, pk_path: &str, inputs: &str) -> PyResult<String> {
    groth16::flatten_json("main",&inputs)
        .and_then(|inputs| helper::prove(&pk_path,inputs))
        .map_err(|err| PyErr::new::<exc::TypeError, _>(py, format!("{:?}",err)))
}

fn verify_py(py: Python, verifying_key : &str, proof_with_inputs : &str) -> PyResult<bool> {
    helper::verify(&verifying_key,&proof_with_inputs)
        .map_err(|err| PyErr::new::<exc::TypeError, _>(py, format!("{:?}",err)))
}
