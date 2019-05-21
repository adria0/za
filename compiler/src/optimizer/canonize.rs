use super::super::algebra::{SignalId, QEQ};
use std::collections::HashMap;

pub fn qeq_type(constrain: &QEQ) -> usize {
    let base = 1000;

    if constrain.a.0.len() >= base || constrain.b.0.len() >= base || constrain.c.0.len() >= base {
        panic!("Base too low");
    }
    (constrain.a.0.len() * base * base) + (constrain.b.0.len() * base) + (constrain.c.0.len())
}

pub fn print_summary(qeqs: Vec<QEQ>) {
    let mut counts = HashMap::new();
    let mut constants = 0;
    let one = 0 as SignalId;

    for constrain in qeqs {
        let id = qeq_type(&constrain);
        *counts.entry(id).or_insert(0) += 1;

        if constrain.a.0.is_empty()
            && constrain.b.0.is_empty()
            && constrain.c.0.len() == 2
            && (constrain.c.0[0].0 == one || constrain.c.0[1].0 == one)
        {
            constants += 1;
        }
    }

    let mut keys = counts.keys().collect::<Vec<_>>();
    keys.sort();

    println!("constants => {}", constants);
    for key in keys {
        println!("{:09} => {}", key, counts[key]);
    }
}

pub fn reduce_constrains(qeqs: Vec<QEQ>) {
    let base = 1000;
    let mut counts = HashMap::new();
    let mut constants = 0;
    let one = 0 as SignalId;

    for constrain in qeqs {
        if constrain.a.0.len() >= base || constrain.b.0.len() >= base || constrain.c.0.len() >= base
        {
            panic!("Base too low");
        }
        let id = (constrain.a.0.len() * base * base)
            + (constrain.b.0.len() * base)
            + (constrain.c.0.len());

        *counts.entry(id).or_insert(0) += 1;

        if constrain.a.0.is_empty()
            && constrain.b.0.is_empty()
            && constrain.c.0.len() == 2
            && (constrain.c.0[0].0 == one || constrain.c.0[1].0 == one)
        {
            constants += 1;
        }
    }

    let mut keys = counts.keys().collect::<Vec<_>>();
    keys.sort();

    println!("constants => {}", constants);
    for key in keys {
        println!("{:09} => {}", key, counts[&key]);
    }
}

pub fn print_dot_type2(qeqs: Vec<QEQ>) {
    let one = 0 as SignalId;
    println!("graph G {{");
    for constrain in qeqs {
        if qeq_type(&constrain) == 2 && constrain.c.0[0].0 != one && constrain.c.0[1].0 != one {
            println!("s{} -- s{};", constrain.c.0[0].0, constrain.c.0[1].0);
        }
    }
    println!("}}");
}

pub fn print_dot(qeqs: Vec<QEQ>) {
    let mut added: HashMap<usize, ()> = HashMap::new();
    println!("graph G {{");
    for constrain in qeqs {
        let mut signals = constrain
            .a
            .0
            .iter()
            .chain(constrain.b.0.iter())
            .chain(constrain.c.0.iter())
            .map(|(s, _)| s)
            .collect::<Vec<_>>();
        signals.sort();
        signals.dedup();
        for i in 0..signals.len() {
            for j in 0..i {
                let one = 0 as SignalId;
                if i != j && *signals[i] != one && *signals[j] != one {
                    let id1 = signals[i] * 100_000_000 + signals[j];
                    let id2 = signals[i] + signals[j] * 100_000_000;
                    if !added.contains_key(&id1) && !added.contains_key(&id2) {
                        println!("s{} -- s{};", signals[i], signals[j]);
                        added.insert(id1, ());
                    }
                }
            }
        }
    }
    println!("}}");
}

/*
fn optimize_constants(qeqs: Vec<QEQ>) {


     step1 -> find & replace
     find (S1,S2) in the QEQ with the form [ ] * [ ] + [aS1 + bS2]
        count all ocurrences of the signals in all QEQ

     find (S1,S2) in the QEQ with the form [ ] * [ ] + [aS1 + bS2]
        if count(S1)>count(S2)
            replace S1 :> b/a S2 in QEQs
        else
            replace S2 :> a/b S1 in QEQs



}

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

fn substitute_lc(lc : &LC, search: SignalId, replace: &LC) -> LC {
    let mut res = lc.clone();
    if let Some(coef) = lc.get(search) {
        for (repl_signal, repl_value) in &replace.0 {
            if search != *repl_signal {
                let mut v = coef * repl_value;
                if let Some(res_value) = res.get(*repl_signal) {
                    v = &v * res_value;
                }
                if v.is_zero() {
                    res.rm(*repl_signal);
                } else {
                    res.set(*repl_signal, |_| v);
                }
            }
        }
        res.rm(search);
    }
    res
}

fn substitute_qeq(qeq: &QEQ, search: SignalId, replace: &LC) -> QEQ {
    QEQ {
        a: substitute_lc(&qeq.a, search, replace),
        b: substitute_lc(&qeq.b, search, replace),
        c: substitute_lc(&qeq.c, search, replace),
    }
}

#[cfg(test)]
mod test {
    use num_bigint::BigInt;
    use num_traits::cast::FromPrimitive;
    use super::super::super::algebra::FS;
    use super::*;

    fn i642fs(i : i64) -> FS {
        FS::from(BigInt::from_i64(i).unwrap())
    }
/*
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
*/
#[test]
fn test_le_substitute() {

let s1 = 1 as SignalId;
let s2 = 2 as SignalId;
let s3 = 3 as SignalId;

// subst( 1s1 + 2s2 , s2 :> 3s3) = 1s1 + 6s3

let lc_1s1_2s2 = &LC::from_signal(s1, i642fs(1)) + &LC::from_signal(s2, i642fs(2));
let lc_3s3 = LC::from_signal(s3, i642fs(3));
let lc_1s1_6s3 = substitute_lc(&lc_1s1_2s2, s2, &lc_3s3);

assert_eq!("1s1+6s3", format!("{:?}", lc_1s1_6s3));
}

#[test]
fn test_le_substitute_inversion() {

let s1 = 1 as SignalId;
let s2 = 2 as SignalId;

// subst( 2s1 + s2 , s2 :> -2s1) = 0

let lc_2s1_1s2 = &LC::from_signal(s1, i642fs(2)) + &LC::from_signal(s2, i642fs(1));
let lc_inv2s1 = LC::from_signal(s1, -&i642fs(2));
let lc_zero = substitute_lc(&lc_2s1_1s2, s2, &lc_inv2s1);

assert_eq!("0", format!("{:?}", lc_zero));
}

#[test]
fn test_qeq_substitute() {
let s2 = 2 as SignalId;
let s3 = 3 as SignalId;

let lc_2s2 = LC::from_signal(s2, i642fs(2));
let lc_2s3 = LC::from_signal(s2, i642fs(3));
let lc_2s4 = LC::from_signal(s2, i642fs(4));
let qeq = &(&lc_2s2 * &lc_2s3) + &lc_2s4;
let lc_3s3 = LC::from_signal(s3, i642fs(3));
let qeq_subst = substitute_qeq(&qeq, s2, &lc_3s3);

assert_eq!("[6s3]*[9s3]+[12s3]", format!("{:?}", qeq_subst));
}

}

*/
