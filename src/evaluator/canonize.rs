// use super::signal::*;
// use super::algebra::{LC,QEQ,SIGNAL_ONE,AlgZero};

// if the deepest equivalence of this signal has an assigned value
//   remove this signal, and add the result to the ONE signal
//   
//       s0 = { equivalence : none , value : 5}
//       s1 = { equivalence : s0   , value : none}
//       LR = 2s1
//
//    converts to
//
//       LR = 10one
//
// else if its non-trivial deepest equivalence has no value 
//   remove this signal, and add to the result of the deepest equivalence
//
//       s0 = { equivalence : none , value : none}
//       s1 = { equivalence : s0   , value : none}
//       LR = 2s1
//
//    converts to
//
//       LR = 2s0
//
/*
fn canonize_lc(lc : &LC, signals : &Signals ) -> LC {
    let mut res = lc.clone();

    for (signal, value) in &lc.0 {

        if signal == SIGNAL_ONE {
            continue;
        }

        // find signal equivalence
        let  alias = signals.equivalent(signal);

        // canonize
        match &signals.get(alias).unwrap().value {
            Some(alias_value) => {
                let mul = alias_value * value;
                res.set(SIGNAL_ONE, |v| {
                    if let Some(v) = v {
                        v + &mul
                    } else {
                        mul
                    }
                });
                res.rm(&signal);
            }
            _ if signal != alias => {
                res.set(&alias, |v| {
                    if let Some(v) = v {
                        v + value
                    } else {
                        value.clone()
                    }
                });
                res.rm(&signal);
            }
            _ => {}
        }
    }
    res.0.retain(|(_, v)| !v.is_zero());
    res
}

pub fn canonize_qeq(qeq : &QEQ,  signals : &Signals) -> QEQ {
    QEQ {
        a: canonize_lc(&qeq.a, signals),
        b: canonize_lc(&qeq.b, signals),
        c: canonize_lc(&qeq.c, signals),
    }
}

#[cfg(test)]
mod test {
    use num_bigint::BigInt;
    use num_traits::cast::FromPrimitive;
    use circom2_parser::ast::SignalType;

    use super::{canonize_lc,canonize_qeq};
    use super::super::signal::*;
    use super::super::algebra::{FS,LC,Value};

    fn i642fs(i : i64) -> FS {
        FS::from(BigInt::from_i64(i).unwrap())
    }

    #[test]
    fn test_canonize_lc_const() {
        let mut s0  = Signal::new(SignalType::Internal,"s0".to_string());
        let mut s1  = Signal::new(SignalType::Internal,"s1".to_string());
        s0.value = Some(Value::from(i642fs(5)));
        s1.equivalence = Some(s0.full_name.clone());
        let mut signals = Signals::new();
        signals.insert(s0);
        signals.insert(s1);

        let lc = LC::from_signal("s1",i642fs(2));
        let lc_c = canonize_lc(&lc, &signals);
        assert_eq!("10one",format!("{:?}",lc_c));
    }

    #[test]
    fn test_canonize_lc_signal() {
        let s0  = Signal::new(SignalType::Internal,"s0".to_string());
        let mut s1  = Signal::new(SignalType::Internal,"s1".to_string());
        s1.equivalence = Some(s0.full_name.clone());
        let mut signals = Signals::new();
        signals.insert(s0);
        signals.insert(s1);

        let lc = LC::from_signal("s1",i642fs(2));
        let lc_c = canonize_lc(&lc, &signals);
        assert_eq!("2s0",format!("{:?}",lc_c));
    }

    #[test]
    fn test_canonize_qeq() {
        let s0  = Signal::new(SignalType::Internal,"s0".to_string());
        let mut s1  = Signal::new(SignalType::Internal,"s1".to_string());
        s1.equivalence = Some(s0.full_name.clone());
        let mut signals = Signals::new();
        signals.insert(s0);
        signals.insert(s1);

        let lc_a = LC::from_signal("s1",i642fs(2));
        let lc_b = LC::from_signal("s1",i642fs(3));
        let lc_c = LC::from_signal("s1",i642fs(4));

        let qeq = &(&lc_a * &lc_b) + &lc_c;
        assert_eq!("[2s1]*[3s1]+[4s1]",format!("{:?}",qeq));

        let qeq_c = canonize_qeq(&qeq, &signals);
        assert_eq!("[2s0]*[3s0]+[4s0]",format!("{:?}",qeq_c));
    }

}
*/
