extern crate stderrlog;
extern crate za_prover;

use za_prover::groth16;
use za_prover::groth16::helper;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn prove(
    pk_path:              *const c_char,
    inputs:               *const c_char,
) -> *mut c_char  {

    let pk_path = unsafe { CStr::from_ptr(pk_path) };
    let pk_path = pk_path.to_str().expect("parse pk_path");

    let inputs = unsafe { CStr::from_ptr(inputs) };
    let inputs = inputs.to_str().expect("parse inputs");

    match groth16::flatten_json("main",&inputs)
    .and_then(|inputs| helper::prove(&pk_path,inputs)) {   
        Ok(proof) => {
            CString::new(format!("1:{}",proof)).unwrap().into_raw()
        }
        Err(err) => { 
            CString::new(format!("0:{:?}",err)).unwrap().into_raw()
        }
    }
}


#[no_mangle]
pub extern fn rust_cstr_free(s: *mut c_char) {
    unsafe {
        if s.is_null() { return }
        CString::from_raw(s)
    };
}