#[cfg(test)]
mod test {
    use super::super::error::Result;
    use super::super::scope::Scope;
    use crate::algebra;
    use crate::evaluator::eval::{Evaluator, Mode};
    use crate::storage::{Constraints, Signals};
    use crate::storage::{Ram, RamConstraints, RamSignals, StorageFactory};
    use crate::evaluator::check_constrains_eval_zero;

    fn constrain_eq<'a, S: Signals, C: Constraints>(
        eval: &Evaluator<S, C>,
        index: usize,
        value: &str,
    ) {
        let name_of = |id| {
            (*(eval.signals.get_by_id(id).unwrap().unwrap()))
                .full_name
                .clone()
        };

        let formatted = eval
            .constraints
            .get(index)
            .unwrap()
            .format(|id| format!("{:?}", name_of(id)));

        assert_eq!(formatted, value);
    }
    fn signal_eq<'a, S: Signals, C: Constraints>(eval: &Evaluator<S, C>, name: &str, value: &str) {
        if let Some(signal) = eval.signals.get_by_name(name).unwrap() {
            assert_eq!(eval.signals.to_string(signal.id).unwrap(), value);
        } else {
            assert_eq!("None", value);
        }
    }
    fn scope_eq(scope: &Scope, name: &str, value: &str) {
        assert_eq!(scope.get(name, |v| format!("{:?}", v)), value);
    }

    fn eval_generic<F, S, C>(
        mode: Mode,
        s: &str,
        deferred_values: Vec<(String, u64)>,
        mut factory: F,
    ) -> Result<(Evaluator<S, C>, Scope)>
    where
        F: StorageFactory<S, C>,
        S: Signals,
        C: Constraints,
    {
        let mut evaluator =
            Evaluator::new(mode, factory.new_signals()?, factory.new_constraints()?);
        deferred_values
            .into_iter()
            .for_each(|(s, v)| evaluator.set_deferred_value(s, algebra::Value::from(v)));

        let mut scope = Scope::new(true, None, "root".to_string());
        evaluator.eval_inline(&mut scope, s)?;
        Ok((evaluator, scope))
    }

    fn eval_constraint(s: &str) -> Result<(Evaluator<RamSignals, RamConstraints>, Scope)> {
        let (eval, scope) = eval_generic(Mode::GenConstraints, s, vec![], Ram::default())?;
        Ok((eval, scope))
    }

    fn eval_witness(s: &str) -> Result<(Evaluator<RamSignals, RamConstraints>, Scope)> {
        let (eval_witness, scope_witness) = eval_generic(Mode::GenWitness, s, vec![], Ram::default())?;
        assert_eq!(eval_witness.constraints.len()?, 0);
            
        Ok((eval_witness, scope_witness))
    }

    fn eval_witness_with_defer(
        s: &str,
        deferred_values: Vec<(String, u64)>,
    ) -> Result<(Evaluator<RamSignals, RamConstraints>, Scope)> {
        let (eval, scope) = eval_generic(Mode::GenWitness, s, deferred_values, Ram::default())?;
        assert_eq!(eval.constraints.len()?, 0);

        let (eval_constraint, _) = eval_generic(Mode::GenConstraints, s, vec![], Ram::default())?;
        
        check_constrains_eval_zero(&eval_constraint.constraints,&eval.signals)?;

        Ok((eval, scope))
    }

    #[test]
    fn test_bodyp_vars() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            var i = 1;
            var j = 5;
            var k = j;",
        )?;

        scope_eq(&scope, "i", "Some(Algebra(1))");
        scope_eq(&scope, "j", "Some(Algebra(5))");
        scope_eq(&scope, "k", "Some(Algebra(5))");
        Ok(())
    }

    #[test]
    fn test_infix_vars() -> Result<()> {
        // + * -
        let (_, scope) = eval_constraint(
            "
            var i = 1+2*3;
            var j = i-3;
        ",
        )?;
        scope_eq(&scope, "i", "Some(Algebra(7))");
        scope_eq(&scope, "j", "Some(Algebra(4))");

        // bool & fs == !=
        let (_, scope) = eval_constraint(
            "
            var iyes = 1==1;
            var ino = 1!=1;
            var byes = iyes==iyes;
            var bno = iyes!=iyes;
        ",
        )?;
        scope_eq(&scope, "iyes", "Some(Bool(true))");
        scope_eq(&scope, "ino", "Some(Bool(false))");
        scope_eq(&scope, "byes", "Some(Bool(true))");
        scope_eq(&scope, "bno", "Some(Bool(false))");

        // <= < > >= compare ops

        let (_, scope) = eval_constraint(
            "
            var yes1 = 1<2;
            var no1 = 1 >2;
            var yes2 = 1<=2;
            var no2 = 1>=2;
        ",
        )?;
        scope_eq(&scope, "yes1", "Some(Bool(true))");
        scope_eq(&scope, "no1", "Some(Bool(false))");
        scope_eq(&scope, "yes2", "Some(Bool(true))");
        scope_eq(&scope, "no2", "Some(Bool(false))");

        Ok(())
    }

    #[test]
    fn test_prefix_vars() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            var i = -5;
            var j=-i;
        ",
        )?;

        scope_eq(&scope, "j", "Some(Algebra(5))");
        Ok(())
    }

    #[test]
    fn test_function() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function f(a) {
                return a;
            }
            var k=f(1);",
        )?;
        scope_eq(&scope, "k", "Some(Algebra(1))");

        let (_, scope) = eval_constraint(
            "
            function f(a,b) {
                return a+b; }\nvar k=f(1,2);",
        )?;
        scope_eq(&scope, "k", "Some(Algebra(3))");

        Ok(())
    }

    #[test]
    fn test_assig_vars() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function f(a) {
                var t=5;
                t+=a;
                t-=2;
                t*=2;
                return t;
            }
            var k=f(2);
        ",
        )?;

        scope_eq(&scope, "k", "Some(Algebra(10))");
        Ok(())
    }

    #[test]
    fn test_for() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function fact(N) {
                var f=1;
                for (var i=1;i<=N;i+=1) {
                    f = f * i;
                } return f;
            }
            var out=fact(10);
        ",
        )?;

        scope_eq(&scope, "out", "Some(Algebra(3628800))");
        Ok(())
    }

    #[test]
    fn test_for_inner_return() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function fact(N) {
                var f=1;
                for (var i=1;i<=N;i+=1) {
                    return N; f = f * i;
                }
                return f;
            }
            var out=fact(10);
        ",
        )?;

        scope_eq(&scope, "out", "Some(Algebra(10))");
        Ok(())
    }

    #[test]
    fn test_while() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function fact(N) {
                var f=1;
                var i=1;
                while (i<=N) {
                    f = f * i;
                    i+=1;
                }
                return f;
            }
            var out=fact(10);
        ",
        )?;

        scope_eq(&scope, "out", "Some(Algebra(3628800))");
        Ok(())
    }

    #[test]
    fn test_while_inner_return() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function fact(N) {
                var f=1;
                var i=1;
                while (i<=N) { 
                    return N;
                    f = f * i;
                    i+=1;
                }
                return f;
            }
            var out=fact(10);
        ",
        )?;

        scope_eq(&scope, "out", "Some(Algebra(10))");
        Ok(())
    }

    #[test]
    fn test_if() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function test(v) {
                if (v==1) {
                    return 1;
                }
                return 2;
            }
            var out1=test(1);
            var out2=test(2);
        ",
        )?;

        scope_eq(&scope, "out1", "Some(Algebra(1))");
        scope_eq(&scope, "out2", "Some(Algebra(2))");
        Ok(())
    }

    #[test]
    fn test_if_else() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function test(v){
                if (v==1) {
                    return 1;
                } else {
                    return 2;
                }
            }
            var out1=test(1);
            var out2=test(2);
        ",
        )?;

        scope_eq(&scope, "out1", "Some(Algebra(1))");
        scope_eq(&scope, "out2", "Some(Algebra(2))");
        Ok(())
    }

    #[test]
    fn test_matrix_get() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function test(){
                var M = [[1,2,3],[4,5,6],[7,8,9]];
                return M[1][1];
            }
            var out=test();
        ",
        )?;

        scope_eq(&scope, "out", "Some(Algebra(5))");
        Ok(())
    }

    #[test]
    fn test_matrix_set() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function test(){
                var M[5][5];
                M[3][1] = 5;
                M[1][2] = 7;
                return M[3][1] + M[1][2];
            }
            var out=test();
        ",
        )?;

        scope_eq(&scope, "out", "Some(Algebra(12))");
        Ok(())
    }

    #[test]
    fn test_template_signal_base() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal a;
                signal input b;
                signal private input c;
                signal output d;
            }
            component main=t();
        ",
        )?;

        signal_eq(&eval, "main.a", "main.a:Internal:None");
        signal_eq(&eval, "main.b", "main.b:PublicInput:None");
        signal_eq(&eval, "main.c", "main.c:PrivateInput:None");
        signal_eq(&eval, "main.d", "main.d:Output:None");
        signal_eq(&eval, "main.e", "None");
        Ok(())
    }

    #[test]
    fn test_template_first_constrain() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal input a;
                signal input b;
                signal private input c;
                c === 5 * a * b  + 5;
            }
            component main=t();
        ",
        )?;

        constrain_eq(&eval, 0, "[-5main.a]*[1main.b]+[-5one+1main.c]");
        Ok(())
    }
    #[test]
    fn test_onlywitness() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal a;
                var i = 1;
                #[w] i=2;
                a === i;
            }
            component main=t();
        ",
        )?;

        constrain_eq(&eval, 0, "[ ]*[ ]+[1main.a-1one]");
        Ok(())
    }

    #[test]
    fn test_signal_fs_assign() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal in;
                signal const;
                const <-- 2;
                2 === 1 + in * const ;
            }
            component main=t();
        ",
        )?;

        signal_eq(&eval, "main.const", "main.const:Internal:Some(2)");
        constrain_eq(&eval, 0, "[ ]*[ ]+[-2main.in+1one]");

        Ok(())
    }

    #[test]
    fn test_signal_equivalence_constrain() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal in;
                signal out;
                out <== in;
                out === 1;
            }
            component main=t();
        ",
        )?;
        constrain_eq(&eval, 0, "[ ]*[ ]+[1main.out-1main.in]");
        constrain_eq(&eval, 1, "[ ]*[ ]+[1main.out-1one]");
        Ok(())
    }

    #[test]
    fn test_signal_fs_constrain() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal in;
                signal const;
                const <== 2;
                2 === 1 + in * const ;
            }
            component main=t();
        ",
        )?;
        constrain_eq(&eval, 0, "[ ]*[ ]+[1main.const-2one]");
        constrain_eq(&eval, 1, "[ ]*[ ]+[-2main.in+1one]");
        Ok(())
    }

    #[test]
    fn test_signal_single_array_assig() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal in[2][2];
                for (var i=0;i<2;i+=1) {
                    in[i][0] <-- i+2 ;
                    in[i][1] <--i+3 ; 
                }
            }
            component main=t();
        ",
        )?;
        signal_eq(&eval, "main.in[0][0]", "main.in[0][0]:Internal:Some(2)");
        signal_eq(&eval, "main.in[0][1]", "main.in[0][1]:Internal:Some(3)");
        signal_eq(&eval, "main.in[1][0]", "main.in[1][0]:Internal:Some(3)");
        signal_eq(&eval, "main.in[1][1]", "main.in[1][1]:Internal:Some(4)");
        Ok(())
    }

    #[test]
    fn test_signal_single_array_constrain() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal in[2][2];
                signal s;
                in[1][0] + in[0][1] === 0 ;
            }
            component main=t();
        ",
        )?;
        constrain_eq(&eval, 0, "[ ]*[ ]+[1main.in[1][0]+1main.in[0][1]]");
        Ok(())
    }

    #[test]
    fn test_signal_single_array_assig_constrain() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal in[2];
                signal s;
                in[0] <== 1 ;
                in[0] === in[1];
            }
            component main=t();
        ",
        )?;
        constrain_eq(&eval, 0, "[ ]*[ ]+[1main.in[0]-1one]");
        constrain_eq(&eval, 1, "[ ]*[ ]+[-1main.in[1]+1one]");
        Ok(())
    }

    #[test]
    fn test_subcomponent() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t0() {
                signal t0in;
                t0in === 5;
            }
            template t1() {
                signal t1in;
                component T0 = t0();
                t1in <== T0.t0in;
            }
            component main=t1();
        ",
        )?;
        constrain_eq(&eval, 0, "[ ]*[ ]+[1main.T0.t0in-5one]");
        Ok(())
    }

    #[test]
    fn test_component_array() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t0() {
                signal t0in;
                t0in === 5;
            }
            template t1() {
                signal t1in;
                component T0[1];
                for (var k=0;k<1;k +=1) {
                    T0[k] = t0();
                    t1in <== T0[k].t0in;
                }
            }
            component main=t1();
        ",
        )?;
        constrain_eq(&eval, 0, "[ ]*[ ]+[1main.T0[0].t0in-5one]");
        Ok(())
    }

    #[test]
    fn test_variable_array() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            function f() {
                var k[1];
                k[0]=6;
                return k[0];
            }
            var out=f();
        ",
        )?;
        scope_eq(&scope, "out", "Some(Algebra(6))");
        Ok(())
    }

    #[test]
    fn test_variable_array_fe_init() -> Result<()> {
        let (_, scope) = eval_constraint(
            "
            var P=[1,2,3,4,5];
            var out=P[2];
        ",
        )?;
        scope_eq(&scope, "out", "Some(Algebra(3))");
        Ok(())
    }

    #[test]
    fn test_witness_simple_check() -> Result<()> {
        let (_, _) = eval_witness(
            "
            template t0() {
                signal t0in;
                t0in <-- 5;
                t0in === 5;
            }
            component main = t0();
        ",
        )?;
        Ok(())
    }

    #[test]
    fn test_witness_simple_fail_unknown_value() -> Result<()> {
        eval_witness(
            "
            template t0() {
                signal t0in;
                t0in === 5;
            }
            component main = t0();
        ",
        )
        .is_err();
        Ok(())
    }

    #[test]
    fn test_witness_simple_fail_bad_value() -> Result<()> {
        eval_witness(
            "
            template t0() {
                signal t0in;
                t0in <-- 2;
                t0in === 5;
            }
            component main = t0();
        ",
        )
        .is_err();
        Ok(())
    }

    #[test]
    fn test_witness_pass_simple_lazy_init() -> Result<()> {
        eval_witness(
            "
            template t1() {
                signal input a;
                a === 2;
            }
            template t0() {
                component c1 = t1();
                c1.a <-- 2;
            }
            component main = t0();
        ",
        )?;
        Ok(())
    }

    #[test]
    fn test_witness_fail_simple_lazy_init() -> Result<()> {
        eval_witness(
            "
            template t1() {
                signal input a;
                a === 3;
            }
            template t0() {
                component c1 = t1();
                c1.a <-- 2;
            }
            component main = t0();
        ",
        )
        .is_err();
        Ok(())
    }

    #[test]
    fn test_witness_pass_simple_lazy_array() -> Result<()> {
        eval_witness(
            "
            template t2() {
                signal input in[1];
                signal output out;  
                out <== in[0] * 3;
            }
            template t1() {
                signal input in[1];
                signal output out; 
                component c2 = t2();
                c2.in[0] <==  in[0]; 
                out <== c2.out * 7;
            }
            template t0() {
                component c1[1];
                c1[0] = t1();
                c1[0].in[0] <== 2;
                c1[0].out === 2*3*7;
            }
            component main = t0();
        ",
        )?;
        Ok(())
    }

    #[test]
    fn test_deferred_evaluation() -> Result<()> {
        eval_witness_with_defer(
            "
            template t() {
                signal input a;
                signal input b;
                a === 2 * b;
            }
            component main = t();
        ",
            vec![("main.a".to_string(), 4), ("main.b".to_string(), 2)],
        )?;
        Ok(())
    }

    #[test]
    fn test_p_1() -> Result<()> {
        eval_witness_with_defer(
            "
            template t() {
                signal input p;
                signal output out;
                out <== 1-p;
            }
            component main = t();
        ",
            vec![("main.p".to_string(), 2)],
        )?;
        Ok(())
    }

    #[test]
    fn test_signal_ordering() -> Result<()> {
        let (eval, _) = eval_constraint(
            "
            template t() {
                signal input pub1;
                signal private input priv1;
                signal int1; 
                signal output out;
                signal private input priv2;
                signal int2; 
                signal input pub2;
                out <== pub1 + pub2 + int1 + int2 + priv1 + priv2;
            }
            component main = t();
        ",
        )?;
        vec![
            "main.out",
            "main.pub1",
            "main.pub2",
            "main.priv1",
            "main.priv2",
            "main.int1",
            "main.int2",
        ]
        .iter()
        .enumerate()
        .for_each(|(n, s)| assert_eq!(1 + n, eval.signals.get_by_name(s).unwrap().unwrap().id));
        Ok(())
    }

}
