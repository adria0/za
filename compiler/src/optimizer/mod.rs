use crate::algebra::AlgZero;
use crate::algebra::SignalId;
use crate::algebra::{FS, LC, QEQ, SIGNAL_ONE};
use crate::storage::RamConstraints;
use crate::storage::{Constraints, Signals};
use std::collections::HashMap;

#[derive(Clone)]
struct Change {
    replace_s : SignalId,
    replace_f : FS,
}

pub fn optimize(
    constraints: &RamConstraints,
    irreductible_signals: &[usize],
) -> (RamConstraints, Vec<SignalId>) {
    let mut replaces = HashMap::<SignalId, Change>::new();
    let mut rmconstraints = Vec::new();

    // optimize constraints
    for n_c in 0..constraints.len().unwrap() {
        let mut cnstr = constraints.get(n_c).unwrap();

        // Rewrite to only-C if possible
        //   rewrite [c1S1][c2SOne]+[c3s3] :> [][]+[c1s2S1+c3s3]
        //   rewrite [c1SOne][c2S2]+[c3s3] :> [][]+[c1s2S1+c3s3]

        if cnstr.a.0.len() == 1 && cnstr.b.0.len() == 1 && cnstr.c.0.len() == 1 {
            if cnstr.a.0[0].0 == SIGNAL_ONE {
                cnstr = QEQ::new(
                    LC::zero(),
                    LC::zero(),
                    &cnstr.c + &LC::from_signal(cnstr.b.0[0].0, &cnstr.a.0[0].1 * &cnstr.b.0[0].1),
                );
            } else if cnstr.b.0[0].0 == SIGNAL_ONE {
                cnstr = QEQ::new(
                    LC::zero(),
                    LC::zero(),
                    &cnstr.c + &LC::from_signal(cnstr.a.0[0].0, &cnstr.a.0[0].1 * &cnstr.b.0[0].1),
                );
            }
        }

        // Remove constrain
        //   a) []][]+[c1S1+c2S2] :>  search: S1 replace: c2/c1 S1 iff S1 is not irreductuble
        //   b) []][]+[c1S1+c2S2] :>  search: S2 replace: c1/c2 S2 iff S2 is not irreductuble

        if cnstr.a.0.len() == 0 && cnstr.b.0.len() == 0 && cnstr.c.0.len() == 2 {
            
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

    let mut keys : Vec<SignalId> = replaces.keys().copied().collect();
    let mut any_processed = true;
    let mut round = 0;
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
                    *r = Change {
                        replace_s : remove_r.replace_s,
                        replace_f : &r.replace_f * &remove_r.replace_f,
                    };
                }
                any_processed = true;
            }
        });
        round+=1;
    }

    let mut opt_cons = crate::storage::RamConstraints::default();
    let mut rm_index = 0;

    for n_c in 0..constraints.len().unwrap() {
        if rm_index < rmconstraints.len() && rmconstraints[rm_index] == n_c {
            rm_index += 1;
            continue;
        }
        let mut con = constraints.get(n_c).unwrap();
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

    (opt_cons, removed_signals)
}

#[test]
fn test_optimize_eq() {
    let mut cons = crate::storage::RamConstraints::default();

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

    cons.push(qeq1, None).unwrap();
    cons.push(qeq2, None).unwrap();
    cons.push(qeq3, None).unwrap();
    let (opt_cons, mut removed_signals) = optimize(&cons, &[sin, sout]);

    let qeq_optimized = QEQ::new(
        LC::zero(),
        LC::zero(),
        &LC::from_signal(sout, FS::one()) + &LC::from_signal(sin, -&FS::from(4)),
    );

    assert_eq!([st, sk].to_vec(), removed_signals);
    assert_eq!(1, opt_cons.len().unwrap());
    assert_eq!(
        format!("{:?}", qeq_optimized),
        format!("{:?}", opt_cons.get(0).unwrap())
    );
}
