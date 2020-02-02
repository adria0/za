use crate::algebra::{FS,LC,QEQ,Value,AlgZero};
use super::signal::Signals;

pub struct Constraints(Vec<(QEQ, Option<String>)>);

impl Default for Constraints {
    fn default() -> Self {
        Constraints(Vec::new())
    }
}

impl Constraints  {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, i: usize) -> QEQ {
        self.0[i].0.clone()
    }
    pub fn get_debug(&self, i: usize) -> Option<String> {
        self.0[i].1.clone()
    }
    pub fn push(&mut self, qeq: QEQ, debug: Option<String>) -> usize {
        self.0.push((qeq, debug));
        self.0.len() - 1
    }
    pub fn satisfies_with_signals(
        &self,
        signals: &Signals,
    ) -> Result<(),String> {
        let eval_lc = |lc: &LC| {
            lc.0.iter().fold(Ok(FS::zero()), |acc, (s, v)| {
                let s_val = if *s == 0 {
                    FS::one()
                } else {
                    let s_val = &*signals.get_by_id(*s).unwrap();
                    match &s_val.value {
                        Some(Value::FieldScalar(fs)) => fs.clone(),
                        _ => {
                            return Err(format!(
                                "signal bad value {:?}",
                                s_val
                            ))
                        }
                    }
                };
                Ok(&acc? + &(v * &s_val))
            })
        };

        for n in 0..self.len() {
            let qeq = self.get(n);
            let a = eval_lc(&qeq.a)?;
            let b = eval_lc(&qeq.b)?;
            let c = eval_lc(&qeq.c)?;

            let zero = &(&a * &b) + &c;

            if !zero.is_zero() {
                let nonzero_value = Value::QuadraticEquation(qeq);
                let debug = self.get_debug(n).unwrap_or_else(|| "".to_string());
                let msg = format!(
                    "constrain '{}' ({}) evals to non-zero ({:?})",
                    signals.format(&nonzero_value),
                    debug,
                    zero
                );
                return Err(msg);
            }
        }

        Ok(())
    }
}
