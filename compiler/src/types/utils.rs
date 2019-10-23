use crate::algebra::{Value, SignalId};
use crate::types::{Constraints, Signals};

pub fn print_info(title: &str, constraints: &Constraints, signals:&Signals, ignore_signals: &[SignalId], print_all: bool) {
    info!(
        "[{}] {} signals, {} constraints",
        title,
        signals.len() - ignore_signals.len(),
        constraints.len()
    );
/*
    let mut abc_count = std::collections::HashMap::new();
    for n in 0..constraints.len() {

        let cnstr = constraints.get(n);
        let k = 
            (cnstr.a.0.len() as u64) 
            + (cnstr.b.0.len() as u64) * 10_000
            + (cnstr.c.0.len() as u64) * 100_000_000;
        
        *(abc_count.entry(k).or_insert(0u64))+=1;
    }
    let split = |v:u64| (v%10_000,(v/10_000)%10_000,v/100_000_000);
    let mut keys : Vec<_> = abc_count.keys().collect();
    keys.sort_by(|a,b| { let (a,b)=(split(**a),split(**b)); (a.0+a.1+a.2).cmp(&(b.0+b.1+b.2)) }); 

    for k in keys.iter() {
        let (a,b,c) = split(**k);
        info!("{} {} {} -> {}",a,b,c,abc_count.get(&k).unwrap());
    }
*/
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