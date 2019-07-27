use super::algebra;
use super::algebra::{AlgZero, Value};
use super::error::*;
use crate::evaluator::Evaluator;
use crate::storage::{Constraints, Signals, Signal};
use std::rc::Rc;

pub fn check_constrains_eval_zero<C:Constraints,S:Signals>(constraints: &C, signals: &S) -> Result<()> { 
    let eval_lc = |lc: &algebra::LC| lc.0
        .iter()
        .fold(Ok(algebra::FS::zero()),|acc,(s,v)| {
            let s_val = if *s == 0 {
                algebra::FS::one()
            } else {
                let s_val = &*signals.get_by_id(*s).unwrap().unwrap();
                match &s_val.value {
                    Some(algebra::Value::FieldScalar(fs)) => fs.clone(),
                    _=> return Err(Error::CannotCheckConstrain(format!("signal bad value {:?}",s_val)))
                }
            };
            Ok(&acc? + &(v * &s_val))
        });

    for n in 0..constraints.len().unwrap() {
        let qeq = constraints.get(n).unwrap();
        let a = eval_lc(&qeq.a)?;
        let b = eval_lc(&qeq.b)?;
        let c = eval_lc(&qeq.c)?;

        let zero = &(&a * &b) + &c;

        if !zero.is_zero() {
            let nonzero_value = algebra::Value::QuadraticEquation(qeq);
            let debug = constraints.get_debug(n).unwrap_or("".to_string());
            let msg = format!("constrain '{}' ({}) evals to non-zero ({:?})",format_algebra(signals,&nonzero_value),debug,zero);
            return Err(Error::CannotCheckConstrain(msg));
        }
    }

    Ok(())
}

pub fn format_algebra<S:Signals>(signals: &S, a: &algebra::Value) -> String {
    let qname = |s: Option<Rc<Signal>>| {
        Ok(s.map_or("unknown".to_string(), |s| s.full_name.to_string()))
    };
    let sname = |id| signals.get_by_id(id).and_then(qname).unwrap();

    match a {
        algebra::Value::FieldScalar(fe) => fe.0.to_str_radix(10),
        algebra::Value::LinearCombination(lc) => lc.format(sname),
        algebra::Value::QuadraticEquation(qeq) => qeq.format(sname),
    }
}

pub fn print_info<S:Signals,C:Constraints>(eval : &Evaluator<S,C>, print_all: bool) {
    info!(
        "{} signals, {} constraints",
        eval.signals.len().unwrap(),
        eval.constraints.len().unwrap()
    );
    if print_all {
        info!("signals -------------------------");
        for n in 0..eval.signals.len().unwrap() {
            info!("{}: {:?}",n,eval.signals.get_by_id(n).unwrap());
        }
        info!("constrains ----------------------");
        for n in 0..eval.constraints.len().unwrap() {
            let constrain = Value::QuadraticEquation(eval.constraints.get(n).unwrap());
            info!("{}:  {}=0",n,format_algebra(&eval.signals,&constrain));
        }
    }
}


