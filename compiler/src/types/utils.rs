use crate::algebra::{Value, SignalId};
use crate::types::{Constraints, Signals};

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
            info!("{}:  {}=0", n, signals.format(&constrain));
        }
    }
}
