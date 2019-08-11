#[macro_use]
extern crate neon;
extern crate stderrlog;
extern crate circom2_prover;
extern crate circom2_compiler;

use neon::prelude::*;

fn verbose(mut cx: FunctionContext) -> JsResult<JsUndefined> {
   let on = cx.argument::<JsBoolean>(0)?.value();
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
   Ok(cx.undefined())
}

fn setup_sync(mut cx: FunctionContext) -> JsResult<JsString> {
    let circuit_path = cx.argument::<JsString>(0)?.value();
    let pk_path = cx.argument::<JsString>(1)?.value();
    
    let verifier_type = match cx.argument::<JsString>(2)?.value().as_ref() {
        "json" => circom2_prover::groth16::VerifierType::JSON,
        "solidity" => circom2_prover::groth16::VerifierType::Solidity,
        _ => return  cx.throw_error(format!("invalid verifier")),
    };

    match circom2_prover::groth16::setup_ram(&circuit_path,&pk_path,verifier_type) {
        Ok(verifier) => Ok(cx.string(verifier)),
        Err(err) => cx.throw_error(format!("{:?}",err)),
    }
}

fn prove_sync(mut cx: FunctionContext) -> JsResult<JsString> {
    let circuit_path = cx.argument::<JsString>(0)?.value();
    let pk_path = cx.argument::<JsString>(1)?.value();
    let inputs = cx.argument::<JsString>(2)?.value();
    match circom2_prover::groth16::flatten_json("main",&inputs) {
        Ok(inputs) => {
            match circom2_prover::groth16::prove_ram(&circuit_path,&pk_path,inputs) {
                Ok(proof) => {
                    Ok(cx.string(proof))
                }
                Err(err) => {
                    cx.throw_error(format!("{:?}",err))
                } 
            }
        } 
        Err(err) => {
            cx.throw_error(format!("{:?}",err))
        }
    }
}

fn verify_sync(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let verifying_key = cx.argument::<JsString>(0)?.value();
    let proof_with_inputs = cx.argument::<JsString>(1)?.value();
    match circom2_prover::groth16::verify_ram(&verifying_key,&proof_with_inputs) {
        Ok(ok) => {
            Ok(cx.boolean(ok))
        }
        Err(err) => {
            cx.throw_error(format!("{:?}",err))
        } 
    }
}

register_module!(mut cx, {
    cx.export_function("proveSync", prove_sync)?;
    cx.export_function("setupSync", setup_sync)?;
    cx.export_function("verifySync", verify_sync)?;
    cx.export_function("verbose", verbose)?;
    Ok(())
});
