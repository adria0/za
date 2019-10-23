extern crate stderrlog;
extern crate za_prover;
extern crate libc;

use za_prover::groth16;
use za_prover::groth16::helper;

use std::ffi::{CStr, CString};

const ERR_NONE                : libc::c_int = 0;
const ERR_BUFFER_TOO_SMALL    : libc::c_int = 1;
const ERR_VERIFICATION_FAILED : libc::c_int = 2;
const ERR_CUSTOM              : libc::c_int = 100;

fn cstr_to_string(cptr :*const libc::c_char) -> String {
    let s = unsafe { 
        CStr::from_ptr(cptr).to_bytes() 
    };
    String::from_utf8(s.to_vec()).unwrap()
}

fn return_string(s : &str, buffer : *mut libc::c_char, size : libc::size_t, ret: libc::c_int) -> libc::c_int {
    if s.len() >= size {
        ERR_BUFFER_TOO_SMALL
    } else {
        let s = CString::new(s).expect("CString::new failed");
        unsafe { libc::strcpy(buffer, s.as_ptr()); };
        ret
    }
}

#[no_mangle]
pub extern "C" fn verbose(on: libc::c_int) {
   if on != 0 {
        groth16::bellman_verbose(true);
        stderrlog::new()
            .verbosity(2)
            .timestamp(stderrlog::Timestamp::Off)
            .init()
            .unwrap();
   } else {
        groth16::bellman_verbose(false);
        stderrlog::new()
            .quiet(true)
            .init()
            .unwrap();
   }
}

#[no_mangle]
pub extern "C" fn setup(
    circuit_path:         *const libc::c_char,
    pk_path:              *const libc::c_char,
    verifier_type:        *const libc::c_char,
    verifier_buffer:      *mut   libc::c_char,
    verifier_buffer_size: libc::size_t, 
    error_buffer:         *mut   libc::c_char,
    error_buffer_size:    libc::size_t, 
) -> libc::c_int {

    let circuit_path = cstr_to_string(circuit_path);
    let pk_path = cstr_to_string(pk_path); 
    let verifier_type = cstr_to_string(verifier_type);

    let verifier_type = match verifier_type.as_ref() {
        "json" => helper::VerifierType::JSON,
        "solidity" => helper::VerifierType::Solidity,
        _ => return return_string("invalid validator type",error_buffer,error_buffer_size,ERR_CUSTOM)
    };

    match helper::setup(&circuit_path,&pk_path,verifier_type) {
        Ok(verifier) => {
            return_string(&verifier,verifier_buffer,verifier_buffer_size,ERR_NONE)
        }
        Err(err) => {
            return_string(&format!("{:?}",err),error_buffer,error_buffer_size,ERR_CUSTOM)
        },
    }
}

#[no_mangle]
pub extern "C" fn prove(
    pk_path:              *const libc::c_char,
    inputs:               *const libc::c_char,
    proof_buffer:         *mut   libc::c_char,
    proof_buffer_size:    libc::size_t, 
    error_buffer:         *mut   libc::c_char,
    error_buffer_size:    libc::size_t, 
) -> libc::c_int {

    let pk_path =  cstr_to_string(pk_path); 
    let inputs = cstr_to_string(inputs);

    match groth16::flatten_json("main",&inputs)
    .and_then(|inputs| helper::prove(&pk_path,inputs)) {   
        Ok(proof) => return_string(&proof,proof_buffer,proof_buffer_size,ERR_NONE),
        Err(err) => return_string(&format!("{:?}",err),error_buffer,error_buffer_size,ERR_CUSTOM)
    }
}

#[no_mangle]
pub extern "C" fn verify(
    verifying_key:        *const libc::c_char,
    proof_with_inputs:    *const libc::c_char,
    error_buffer:         *mut   libc::c_char,
    error_buffer_size:    libc::size_t, 
) -> libc::c_int {

    let verifying_key = cstr_to_string(verifying_key);
    let proof_with_inputs =  cstr_to_string(proof_with_inputs); 

    match helper::verify(&verifying_key,&proof_with_inputs) {
        Ok(true) => ERR_NONE,
        Ok(false) => ERR_VERIFICATION_FAILED,
        Err(err) => return_string(&format!("{:?}",err),error_buffer,error_buffer_size,ERR_CUSTOM),
    }
}
