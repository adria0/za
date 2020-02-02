use std::collections::HashMap;

use crate::algebra::AlgZero;
use crate::algebra::SignalId;
use crate::algebra::{FS, LC, QEQ, SIGNAL_ONE};
use crate::types::Constraints;

#[derive(Clone)]
struct Change {
    replace_s : SignalId,
    replace_f : FS,
}

pub fn optimize(
    constraints: &Constraints,
    irreductible_signals: &[SignalId],
) -> (Constraints, Vec<SignalId>) {

    let (constraints, mut r1) = optimize_n(&constraints,irreductible_signals);
    println!("Optimize L1 {} {}",constraints.len(),r1.len());
    let (constraints, mut r2) = optimize_n(&constraints,irreductible_signals);
    println!("Optimize L2 {} {}",constraints.len(),r2.len());
    let (constraints, mut r3) = optimize_n(&constraints,irreductible_signals);
    println!("Optimize L3 {} {}",constraints.len(),r3.len());
    
    r1.append(&mut r2);
    r1.append(&mut r3);

    (constraints, r1)
}

pub fn optimize_n(
    constraints: &Constraints,
    irreductible_signals: &[SignalId],
) -> (Constraints, Vec<SignalId>) {
    let mut replaces = HashMap::<SignalId, Change>::new();
    let mut rmconstraints = Vec::new();

    let mut type1 = 0;
    let mut type2 = 0;
    let mut type3 = 0;

    // optimize constraints
    for n_c in 0..constraints.len() {
        let mut cnstr = constraints.get(n_c);

        // Rewrite to only-C if possible
        //   rewrite [a][c2SOne]+[c3s3] :> [][]+[ c2 a c3s3 ]
        //   rewrite [c1SOne][b]+[c3s3] :> [][]+[ c1 b c3s3 ]

        if cnstr.a.0.len() == 1 && cnstr.a.0[0].0 == SIGNAL_ONE {
            cnstr = QEQ::new(
                LC::zero(),
                LC::zero(),
                &cnstr.c + &(&cnstr.b  * &cnstr.a.0[0].1),
            );
            type1 += 1;        
        } else if cnstr.b.0.len() == 1 && cnstr.b.0[0].0 == SIGNAL_ONE {
            cnstr = QEQ::new(
                LC::zero(),
                LC::zero(),
                &cnstr.c + &(&cnstr.a  * &cnstr.b.0[0].1),
            );
            type1 += 1;        
        }

        // Remove constrain
        //   a) [][]+[c1S1+c2S2] :>  search: S1 replace: c2/c1 S1 iff S1 is not irreductuble
        //   b) [][]+[c1S1+c2S2] :>  search: S2 replace: c1/c2 S2 iff S2 is not irreductuble

        if cnstr.a.0.is_empty() && cnstr.b.0.is_empty() && cnstr.c.0.len() == 2 {
            
            let first = &cnstr.c.0[0];
            let second = &cnstr.c.0[1];

            let first_is_irreductible = irreductible_signals.iter().any(|&x| x == first.0);
            let second_is_irreductible = irreductible_signals.iter().any(|&x| x == second.0);

            let (search, replace) = match (first_is_irreductible,second_is_irreductible) {
                (false, true) => (first, second),
                (true, false) => (second, first),
                (false, false) => if first.0 > second.0 { (first,second) } else { (second,first) },
                _ => continue
            };

            let (search_s, mut replace_s, mut replace_f) = (
                search.0, replace.0,
                 -&(&replace.1 / &search.1).unwrap()
            );

            if replaces.get(&search_s).is_none() {
                
                while let Some(v) = replaces.get(&replace_s.clone()) {
                    replace_s = v.replace_s;
                    replace_f = &replace_f * &v.replace_f;
                    type3 +=1;
                }
                
                replaces.insert(
                    search_s,
                    Change {
                        replace_s,
                        replace_f,
                    },
                );

                rmconstraints.push(n_c);
            }

        }
    }


    // fix replaces
    //
    // if can happen this case:
    //    [9868] :> [287] (1)
    //    [287] :> [30]   (2)
    //
    // this means that signal 287 will be removed in (2) but is still used in (1)
    // even more, this should be optimized into
    //   
    //    [9868] :> [30]
    //
    // so, now let's reduce the graph
    // TODO: optimize

    let keys : Vec<SignalId> = replaces.keys().copied().collect();
    let mut any_processed = true;
    while any_processed  {
        any_processed = false;
        keys.iter().for_each(|s| {
            let mut remove = None;
            if let Some(r) = replaces.get(s) {
                if let Some(r2) = replaces.get(&r.replace_s) {
                    // now we are in the case [s] :> f1*[r], [r] :> f2*[r2]
                    // update [s] :> f1*[r] to [s] :> f1*f2*[r2]
                    remove = Some(r2.clone());
                } 
            }
            if let Some(remove_r) = remove {
                if let Some(r) = replaces.get_mut(s) {
                    type2 += 1;
                    *r = Change {
                        replace_s : remove_r.replace_s,
                        replace_f : &r.replace_f * &remove_r.replace_f,
                    };
                }
                any_processed = true;
            }
        });
    }

    // now update constraints
    let mut opt_cons = Constraints::default();
    let mut rm_index = 0;

    for n_c in 0..constraints.len() {
        if rm_index < rmconstraints.len() && rmconstraints[rm_index] == n_c {
            rm_index += 1;
            continue;
        }
        let mut con = constraints.get(n_c);
        for lcelem in con.a.0.iter_mut() {
            if let Some(v) = replaces.get(&lcelem.0) {
                *lcelem = (v.replace_s, &lcelem.1 * &v.replace_f);
            }
        }
        for lcelem in con.b.0.iter_mut() {
            if let Some(v) = replaces.get(&lcelem.0) {
                *lcelem = (v.replace_s, &lcelem.1 * &v.replace_f);
            }
        }
        for lcelem in con.c.0.iter_mut() {
            if let Some(v) = replaces.get(&lcelem.0) {
                *lcelem = (v.replace_s, &lcelem.1 * &v.replace_f);
            }
        }
        opt_cons.push(con, None);
    }

    let mut removed_signals = Vec::with_capacity(replaces.len());
    replaces
        .into_iter()
        .for_each(|(k, _)| removed_signals.push(k));

    removed_signals.sort();

    info!("type1={} type2={} type3={}",type1,type2,type3);

    (opt_cons, removed_signals)
}

#[test]
fn test_optimize_eq() {
    let mut cons = Constraints::default();

    let sin = 1 as SignalId;
    let st = 2 as SignalId;
    let sk = 3 as SignalId;
    let sout = 4 as SignalId;

    // t <== in * 2
    let qeq1 = QEQ::new(
        LC::zero(),
        LC::zero(),
        &LC::from_signal(st, FS::one()) + &LC::from_signal(sin, -&FS::from(2)),
    );

    // k * 2 <== t * 4
    let qeq2 = QEQ::new(
        LC::from_signal(SIGNAL_ONE, FS::from(2)),
        LC::from_signal(sk, FS::one()),
        LC::from_signal(st, -&FS::from(4)),
    );

    // out === k
    let qeq3 = QEQ::new(
        LC::zero(),
        LC::zero(),
        &LC::from_signal(sout, FS::one()) + &LC::from_signal(sk, -&FS::one()),
    );

    cons.push(qeq1, None);
    cons.push(qeq2, None);
    cons.push(qeq3, None);
    let (opt_cons, removed_signals) = optimize_n(&cons, &[sin, sout]);

    let qeq_optimized = QEQ::new(
        LC::zero(),
        LC::zero(),
        &LC::from_signal(sout, FS::one()) + &LC::from_signal(sin, -&FS::from(4)),
    );

    assert_eq!([st, sk].to_vec(), removed_signals);
    assert_eq!(1, opt_cons.len());
    assert_eq!(
        format!("{:?}", qeq_optimized),
        format!("{:?}", opt_cons.get(0))
    );
}
