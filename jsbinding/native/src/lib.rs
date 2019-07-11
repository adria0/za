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

fn setup_sync(mut cx: FunctionContext) -> JsResult<JsUndefined> {

    let circuit_path = cx.argument::<JsString>(0)?.value();
    let pk_path = cx.argument::<JsString>(1)?.value();
    let sol_path = cx.argument::<JsString>(2)?.value();

    if let Err(err) = circom2_prover::groth16::setup_ram(&circuit_path,&pk_path,&sol_path) {
        cx.throw_error(format!("{:?}",err))
    } else {
        Ok(cx.undefined())
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

register_module!(mut cx, {
    cx.export_function("proveSync", prove_sync)?;
    cx.export_function("setupSync", setup_sync)?;
    cx.export_function("verbose", verbose)?;
    Ok(())
});
