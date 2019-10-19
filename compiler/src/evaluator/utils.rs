use super::algebra;
use super::algebra::{AlgZero, Value, SignalId};
use super::error::*;

use crate::types::{Constraints, Signals};

pub fn check_constrains_eval_zero(
    constraints: &Constraints,
    signals: &Signals,
) -> Result<()> {
    let eval_lc = |lc: &algebra::LC| {
        lc.0.iter().fold(Ok(algebra::FS::zero()), |acc, (s, v)| {
            let s_val = if *s == 0 {
                algebra::FS::one()
            } else {
                let s_val = &*signals.get_by_id(*s).unwrap();
                match &s_val.value {
                    Some(algebra::Value::FieldScalar(fs)) => fs.clone(),
                    _ => {
                        return Err(Error::CannotCheckConstrain(format!(
                            "signal bad value {:?}",
                            s_val
                        )))
                    }
                }
            };
            Ok(&acc? + &(v * &s_val))
        })
    };

    for n in 0..constraints.len() {
        let qeq = constraints.get(n);
        let a = eval_lc(&qeq.a)?;
        let b = eval_lc(&qeq.b)?;
        let c = eval_lc(&qeq.c)?;

        let zero = &(&a * &b) + &c;

        if !zero.is_zero() {
            let nonzero_value = algebra::Value::QuadraticEquation(qeq);
            let debug = constraints.get_debug(n).unwrap_or("".to_string());
            let msg = format!(
                "constrain '{}' ({}) evals to non-zero ({:?})",
                format_algebra(signals, &nonzero_value),
                debug,
                zero
            );
            return Err(Error::CannotCheckConstrain(msg));
        }
    }

    Ok(())
}

pub fn format_algebra(signals: &Signals, a: &algebra::Value) -> String {

    let sname = |id| signals.get_by_id(id).map_or("unwnown".to_string(), |s| s.full_name.to_string());
    match a {
        algebra::Value::FieldScalar(fe) => fe.to_string(),
        algebra::Value::LinearCombination(lc) => lc.format(sname),
        algebra::Value::QuadraticEquation(qeq) => qeq.format(sname),
    }
}

pub fn print_info(title: &str, constraints: &Constraints, signals:&Signals, ignore_signals: &[SignalId], print_all: bool) {
    info!(
        "[{}] {} signals, {} constraints",
        title,
        signals.len() - ignore_signals.len(),
        constraints.len()
    );
    if print_all {
        info!("signals -------------------------");
        let mut ignore_it = ignore_signals.iter().peekable();
        for n in 0..signals.len() {
            if let Some(i) = ignore_it.peek() {
                if n == **i {
                    ignore_it.next();
                    continue;
                }
            }
            info!("{}: {:?}", n, signals.get_by_id(n).unwrap());
        }
        info!("constrains ----------------------");
        for n in 0..constraints.len() {
            let constrain = Value::QuadraticEquation(constraints.get(n));
            info!("{}:  {}=0", n, format_algebra(signals, &constrain));
        }
    }
}
