#[cfg(test)]
mod test {
    use super::super::eval::Evaluator;
    use super::super::scope::Scope;
    use super::super::error::Result;

    fn constrain_eq(eval: &Evaluator, index: usize, value : &str) {
        let signals = &eval.signals;
        let formatted = eval.constrains[index].format(
            |id| format!("{:?}",signals.get_by_id(id).unwrap().full_name)
        );
        assert_eq!(formatted,value);
    }
    fn signal_eq(eval: &Evaluator, name: &str, value: &str) {
        let signals = &eval.signals;
        if let Some(signal) = signals.get_by_name(name) {
            assert_eq!(signals.to_string(signal.id),value);
        } else {
            assert_eq!("None",value);
        }
    }
    fn scope_eq(scope: &Scope, name: &str, value: &str) {
        assert_eq!(scope.get(name, |v| format!("{:?}",v)),value);
    }

    #[test]
    fn test_bodyp_vars() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("var i = 1;var j=5; var k=j;")?; 
        
        scope_eq(&scope,"i","Some(Algebra(1))");
        scope_eq(&scope,"j","Some(Algebra(5))");
        scope_eq(&scope,"k","Some(Algebra(5))");
        Ok(())
    }

    #[test]
    fn test_infix_vars() -> Result<()> {
        // + * -
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("var i = 1+2*3; var j=i-3;")?;
        scope_eq(&scope,"i","Some(Algebra(7))");
        scope_eq(&scope,"j","Some(Algebra(4))");

        // bool & fs == !=
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("var iyes=1==1;var ino=1!=1; var byes=iyes==iyes; var bno=iyes!=iyes;")?; 
         
        scope_eq(&scope,"iyes","Some(Bool(true))");
        scope_eq(&scope,"ino","Some(Bool(false))");
        scope_eq(&scope,"byes","Some(Bool(true))");
        scope_eq(&scope,"bno","Some(Bool(false))");

        // <= < > >= compare ops
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("var yes1=1<2;var no1=1>2; var yes2=1<=2; var no2=1>=2;")?; 
         
        scope_eq(&scope,"yes1","Some(Bool(true))");
        scope_eq(&scope,"no1","Some(Bool(false))");
        scope_eq(&scope,"yes2","Some(Bool(true))");
        scope_eq(&scope,"no2","Some(Bool(false))");

        Ok(())
    }

    #[test]
    fn test_prefix_vars() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("var i = -5; var j=-i;")?; 
         
        scope_eq(&scope,"j","Some(Algebra(5))");
        Ok(())
    }

    #[test]
    fn test_function() -> Result<()> {
        let mut eval = Evaluator::new();

        let scope = eval.eval_inline("function f(a) { return a; }\nvar k=f(1);")?; 
        scope_eq(&scope,"k","Some(Algebra(1))");

        let scope = eval.eval_inline("function f(a,b) { return a+b; }\nvar k=f(1,2);")?;  
        scope_eq(&scope,"k","Some(Algebra(3))");
        
        Ok(())
    }

    #[test]
    fn test_assig_vars()-> Result<()> {
        let mut eval = Evaluator::new();

        let scope = eval.eval_inline("function f(a) { var t=5; t+=a; t-=2; t*=2; return t; }\nvar k=f(2);")?; 
         
        scope_eq(&scope,"k","Some(Algebra(10))");
        Ok(())
    }

    #[test]
    fn test_for() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("function fact(N) { var f=1; for (var i=1;i<=N;i+=1) { f = f * i; } return f;}\nvar out=fact(10);")?; 
         
         
        scope_eq(&scope,"out","Some(Algebra(3628800))");
        Ok(())
    }

    #[test]
    fn test_for_inner_return() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("function fact(N) { var f=1; for (var i=1;i<=N;i+=1) { return N; f = f * i; } return f;}\nvar out=fact(10);")?; 
        scope_eq(&scope,"out","Some(Algebra(10))");
        Ok(())
    }

    #[test]
    fn test_while() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("function fact(N) { var f=1; var i=1; while (i<=N) { f = f * i; i+=1; } return f;}\nvar out=fact(10);")?; 
        scope_eq(&scope,"out","Some(Algebra(3628800))");
        Ok(())
    }

    #[test]
    fn test_while_inner_return() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("function fact(N) { var f=1; var i=1; while (i<=N) { return N; f = f * i; i+=1; } return f;}\nvar out=fact(10);")?; 
        scope_eq(&scope,"out","Some(Algebra(10))");
        Ok(())
    }

    #[test]
    fn test_if() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("function test(v) { if (v==1) { return 1; } return 2;}\nvar out1=test(1); var out2=test(2);")?; 
        scope_eq(&scope,"out1","Some(Algebra(1))");
        scope_eq(&scope,"out2","Some(Algebra(2))");
        Ok(())
    }

    #[test]
    fn test_if_else() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("function test(v) { if (v==1) { return 1; } else { return 2;}}\nvar out1=test(1); var out2=test(2);")?; 
        scope_eq(&scope,"out1","Some(Algebra(1))");
        scope_eq(&scope,"out2","Some(Algebra(2))");
        Ok(())
    }

    #[test]
    fn test_template_signal_base() -> Result<()> {
        let mut eval = Evaluator::new();

        eval.eval_inline("template t() { signal a; signal input b; signal private input c; signal output d; }\ncomponent main=t();")?; 
        signal_eq(&eval,"main.a","main.a:Internal:None");
        signal_eq(&eval,"main.b","main.b:PublicInput:None");
        signal_eq(&eval,"main.c","main.c:PrivateInput:None");
        signal_eq(&eval,"main.d","main.d:Output:None");
        signal_eq(&eval,"main.e","None");
        Ok(())
    }

    #[test]
    fn test_template_first_constrain() -> Result<()> {
        let mut eval = Evaluator::new();

        eval.eval_inline("template t() { signal input a; signal input b; signal private input c; c === 5 * a * b  + 5;}\ncomponent main=t();")?; 
         
        constrain_eq(&eval,0,"[5main.a]*[1main.b]+[5one-1main.c]");
        Ok(())
    }

   #[test]
    fn test_signal_fs_assign()-> Result<()> {
        let mut eval = Evaluator::new();
        eval.eval_inline("template t() { signal in; signal const; const <-- 2;  2 === 1 + in * const ;}\ncomponent main=t();")?; 
        println!("{:?}",eval.signals);

        signal_eq(&eval,"main.const","main.const:Internal:Some(2)");
        constrain_eq(&eval,0,"[ ]*[ ]+[2main.in-1one]");

        Ok(())
    }

   #[test]
    fn test_signal_equivalence_constrain() -> Result<()> {
        let mut eval = Evaluator::new();
        eval.eval_inline("template t() { signal in; signal out; out <== in; out === 1; }\ncomponent main=t();")?; 


        constrain_eq(&eval,0,"[ ]*[ ]+[1main.out-1main.in]");
        constrain_eq(&eval,1,"[ ]*[ ]+[1main.out-1one]");
        Ok(())
    }

   #[test]
    fn test_signal_fs_constrain() -> Result<()> {
        let mut eval = Evaluator::new();
        eval.eval_inline("template t() { signal in; signal const; const <== 2; 2 === 1 + in * const ; }\ncomponent main=t();")?; 
         
        constrain_eq(&eval,0,"[ ]*[ ]+[1main.const-2one]");
        constrain_eq(&eval,1,"[ ]*[ ]+[2main.in-1one]");
        Ok(())
    }
    
    #[test]
    fn test_signal_single_array_assig()  -> Result<()>{
        let mut eval = Evaluator::new();
        eval.eval_inline("template t() { signal in[2][2]; for (var i=0;i<2;i+=1) { in[i][0] <-- i+2 ; in[i][1] <--i+3 ; }}\ncomponent main=t();")?; 
         
        signal_eq(&eval,"main.in[0][0]","main.in[0][0]:Internal:Some(2)");
        signal_eq(&eval,"main.in[0][1]","main.in[0][1]:Internal:Some(3)");
        signal_eq(&eval,"main.in[1][0]","main.in[1][0]:Internal:Some(3)");
        signal_eq(&eval,"main.in[1][1]","main.in[1][1]:Internal:Some(4)");
        Ok(())
    }

    #[test]
    fn test_signal_single_array_constrain()-> Result<()> {
        let mut eval = Evaluator::new();
        eval.eval_inline("template t() { signal in[2][2]; signal s; in[1][0] + in[0][1] === 0 ; }\ncomponent main=t();")?; 
         
        constrain_eq(&eval,0,"[ ]*[ ]+[1main.in[1][0]+1main.in[0][1]+0one]");
        Ok(())
    }

    #[test]
    fn test_signal_single_array_assig_constrain()-> Result<()> {
        let mut eval = Evaluator::new();
         eval.eval_inline("template t() { signal in[2]; signal s; in[0] <== 1 ; in[0] === in[1]; }\ncomponent main=t();")?; 

        constrain_eq(&eval,0,"[ ]*[ ]+[1main.in[0]-1one]");
        constrain_eq(&eval,1,"[ ]*[ ]+[1main.in[1]-1one]");
        Ok(())
    }

    #[test]
    fn test_subcomponent() -> Result<()> {
        let mut eval = Evaluator::new();
        eval.eval_inline("template t0() { signal t0in; t0in === 5; } template t1() { signal t1in; component T0 = t0(); t1in <== T0.t0in; }\ncomponent main=t1();")?; 
         
        constrain_eq(&eval,0,"[ ]*[ ]+[1main.T0.t0in-5one]");
        Ok(())
    }

    #[test]
    fn test_component_array() -> Result<()> {
        let mut eval = Evaluator::new();
        eval.eval_inline("template t0() { signal t0in; t0in === 5; } template t1() { signal t1in; component T0[1]; for (var k=0;k<1;k +=1) { T0[k] = t0(); t1in <== T0[k].t0in; }}\ncomponent main=t1();")?; 

        constrain_eq(&eval,0,"[ ]*[ ]+[1main.T0[0].t0in-5one]");
        Ok(())
    }

    #[test]
    fn test_variable_array() -> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("function f() { var k[1]; k[0]=6; return k[0]; }\nvar out=f();")?; 
         
        scope_eq(&scope,"out","Some(Algebra(6))");
        Ok(())
    }

    #[test]
    fn test_variable_array_fe_init()-> Result<()> {
        let mut eval = Evaluator::new();
        let scope = eval.eval_inline("var P=[1,2,3,4,5];\nvar out=P[2];")?; 
         
        scope_eq(&scope,"out","Some(Algebra(3))");
        Ok(())
    }
}