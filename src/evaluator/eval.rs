use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use num_bigint::BigInt;
use circom2_parser;
use circom2_parser::ast::{
    SignalType, BodyElementP, ExpressionP, Opcode, StatementP, SelectorP, VariableP, VariableType, Meta
};

use blake2_rfc::blake2b::{Blake2b};
use hex;

use super::algebra;
use super::algebra::{SignalId,QEQ,AlgZero};
use super::error::*;
use super::signal::*;
use super::retval::*;
use super::scope::*;

pub struct Component {
    signal_ids : Vec<SignalId>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            signal_ids : Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ErrorContext {
    pub scope : String,
    pub meta : Meta,
    pub file : String,
    pub component : String,
    pub function : Option<String>  
}

#[derive(PartialEq,Debug)]
pub enum Mode {
    Collect,        // collect declarations
    GenConstraints, // generate R1CS
    Test,           // run tests
}

impl Mode {
    pub fn must_process_eval(&self, meta: &Meta) -> bool {
        !(self == &Mode::GenConstraints && meta.attrs.has_tag_w()) 
    }
    pub fn must_process_root_decrl(&self) -> bool {
        self != &Mode::Collect 
    }
}

pub struct Evaluator {

    // the current file, component and function being processed
    pub current_file : String,
    pub current_component : String,
    pub current_function  : Option<String>,
    pub debug_iterations  : usize,

    // collected signals, constrains and components
    pub signals    : Signals,
    pub constrains : Vec<QEQ>,
    pub components : HashMap<String,Option<Component>>,

    // processed includes
    pub processed_files : Vec<String>,

    // last got error
    pub last_error : Option<ErrorContext>,

    // evaluation mode
    pub mode : Mode,

}

impl Evaluator {

    pub fn new(mode : Mode) -> Self {
        Self {
            current_file : "".to_string(),
            current_component : "".to_string(),
            current_function : None,
            signals : Signals::default(),
            constrains : Vec::new(),
            components : HashMap::new(),
            debug_iterations : 0,
            processed_files : Vec::new(),
            last_error : None,
            mode
        }
    }

    // public interface ---------------------------------------------------------------------------

    pub fn eval_inline(&mut self, scope: &mut Scope, code : &str) -> Result<()> {
        match circom2_parser::parse(&code) {
            Ok(elements) =>
                self.eval_body_elements_p(&Meta::new(0,0,None), scope, &elements)?,

            Err(circom2_parser::Error::ParseError(err,meta)) =>
                return self.register_error(&meta,&scope,Err(Error::Parse(err)))                       
        }
        Ok(())        
    }

    pub fn eval_file(&mut self, path : &str) -> Result<Scope> {
        let mut scope = Scope::new(true, None,  path.to_string());
        self.eval_include(&Meta::new(0,0,None), &mut scope, path)?;
        Ok(scope)
    }

    pub fn format_algebra(&self, a : algebra::Value ) -> String {
        let sname = |id| self.signals.get_by_id(id).map_or("unknown".to_string(),|s| s.full_name.to_string());
        match a {
            algebra::Value::FieldScalar(fe) => format!("{:?}",fe),
            algebra::Value::LinearCombination(lc) => lc.format(sname),
            algebra::Value::QuadraticEquation(qeq) => qeq.format(sname),
        }
    }

    // evaluators -----------------------------------------------------------------------------------

    fn debug_trace(&mut self, _meta: &Meta) {
        if self.debug_iterations == 0 {
            self.debug_iterations = 1;
            //println!("debug: {}:{} {:?}",self.current_file,self.current_component,self.current_function);
        } else {
            self.debug_iterations -= 1;
        }
    }

    fn register_error<T>(&mut self,  meta: &Meta, scope: &Scope, res: Result<T>) -> Result<T> {        
        if res.is_err() && self.last_error.is_none() {
            self.last_error = Some(ErrorContext {
                scope : format!("{:?}",scope),
                meta : meta.clone(),
                file : self.current_file.clone(),
                component : self.current_component.clone(),
                function : self.current_function.clone()
            });
        }
        res
    } 

    fn alg_eval_prefix(&mut self, meta: &Meta, scope: &Scope,  op: circom2_parser::ast::Opcode, rhv: &algebra::Value) -> Result<algebra::Value>  {
        match algebra::eval_prefix(op,rhv) {
            Err(err) => self.register_error(meta,scope,Err(Error::Algebra(err))),
            Ok(v) => Ok(v)
        }
    }
    fn alg_eval_infix(&mut self, meta: &Meta, scope: &Scope, lhv: &algebra::Value, op: circom2_parser::ast::Opcode, rhv: &algebra::Value) -> Result<algebra::Value>  {
        match algebra::eval_infix(lhv,op,rhv) {
            Err(err) => self.register_error(meta,scope,Err(Error::Algebra(err))),
            Ok(v) => Ok(v)
        }
    }

    fn eval_expression_p(
        &mut self,
        scope: &Scope,
        v: &ExpressionP
    ) -> Result<ReturnValue> {
        use circom2_parser::ast::ExpressionP::*;
        match v {
            FunctionCall{meta,name,args} => self.eval_function_call(meta,scope, name, args),
            Variable{meta,name} => self.eval_variable(meta,scope, name),
            Number{meta,value} => self.eval_number(meta,scope, value),
            PrefixOp{meta,op,rhe} => self.eval_prefix_op(meta,scope, *op, rhe),
            InfixOp{meta,lhe,op,rhe} => self.eval_infix_op(meta,scope, lhe, *op, rhe),
            Array{meta, values} => self.eval_array(meta,scope,values),
        }
    }

    fn eval_statement_p(
        &mut self,
        scope: &mut Scope,
        v: &StatementP
    ) -> Result<()> {
        use circom2_parser::ast::StatementP::*;
        match v {
            IfThenElse{meta, xif, xthen, xelse} => self.eval_if_then_else(meta,scope,xif, xthen, xelse),
            For{meta,init, cond, step, stmt} => self.eval_for(meta,scope,init, cond, step, stmt),
            While{meta, cond, stmt} => self.eval_while(meta,scope, cond, stmt),
            Return{meta , value} => self.eval_return(meta,scope, value),
            Declaration{meta, xtype, name, init} => self.eval_declaration(meta,scope, *xtype, name, init),
            Substitution{meta, name, op, value} => self.eval_substitution(meta,scope,name, *op, value),
            Block{meta , stmts} => self.eval_block(meta,scope, stmts),
            SignalLeft{meta, name, op, value} => self.eval_signal_left(meta,scope,name, *op, value),
            SignalRight{meta, value, op, name} => self.eval_signal_right(meta,scope, value, *op, name),
            SignalEq{meta , lhe , rhe ,..} => self.eval_signal_eq(meta,scope, lhe, rhe),
            InternalCall{meta,name,args} => self.eval_internal_call(meta,scope, name, args),
        }
    }

    fn eval_body_element_p(
        &mut self,
        scope: &mut Scope,
        v: &BodyElementP
    ) -> Result<()> {
        use circom2_parser::ast::BodyElementP::*;
        match v {
            Include{meta,path} => self.eval_include(meta,scope, path),
            FunctionDef{meta, name, args, stmt} => self.eval_function_def(meta,scope, name, args, stmt),
            TemplateDef{meta, name, args, stmt} => self.eval_template_def(meta,scope, name, args, stmt),
            Declaration{decl ,..} => self.eval_statement_p(scope, decl),
        }
    }

    fn eval_internal_call(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        name: &str,
        params: &[Box<ExpressionP>],
    ) -> Result<()> {
        self.debug_trace(meta);

        let mut internal = || {        
            if name == "dbg" {
                print!("DBG ");            
                for param in params {                
                    let value = self.eval_expression_p(scope, param)?;
                    print!("{:?}",value);
                }
                println!();
                return Ok(());
            }
            Err(Error::NotFound(format!("internal funcion {}!",name)))
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_function_call(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        name: &str,
        params: &[Box<ExpressionP>],
    ) -> Result<ReturnValue> {
        self.debug_trace(meta);
        
        let mut internal = || {        
            scope.root().get(name, |v| match v {
                Some(ScopeValue::Function(args, stmt, function_path)) => {
                    if args.len() != params.len() {
                        return Err(Error::InvalidParameter(name.to_string()));
                    }

                    let mut func_scope = Scope::new(
                        true, Some(scope),
                        format!("{}:{}",self.current_file,meta.start)
                    );

                    for n in 0..args.len() {
                        let value = self.eval_expression_p(scope, &*params[n])?;
                        func_scope.insert(args[n].clone(), ScopeValue::from(value));
                    }

                    let mut new_current_function = Some(name.to_string()); 
                    let mut new_current_file = function_path.to_string(); 

                    std::mem::swap(&mut new_current_function, &mut self.current_function);
                    std::mem::swap(&mut new_current_file, &mut self.current_file);

                    self.eval_statement_p(&mut func_scope, stmt)?;

                    std::mem::swap(&mut self.current_function, &mut new_current_function);
                    std::mem::swap(&mut self.current_file, &mut new_current_file );
                    
                    func_scope.take_return().ok_or_else(||Error::BadFunctionReturn(name.to_string()))
                }
                _ => Err( Error::NotFound(format!("function {}",name))),
            })
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_component_decl(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        name: &VariableP,
     ) -> Result<()> {
        self.debug_trace(meta);
        for selector_name in self.generate_selectors(scope, &name)? {
            self.components.insert(self.expand_full_name(&selector_name), None);
        }
        Ok(())
    }

    fn eval_component_instantiation(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        component_name: &str,
        init: &ExpressionP,
    ) -> Result<()> {
        self.debug_trace(meta);

        let mut internal = || {
            if let ExpressionP::FunctionCall{name: template_name, args: params,..} = init {
                scope.root().get(template_name, |v| match v {
                    Some(ScopeValue::Template(_, args, stmt, template_path)) => {
                        if args.len() != params.len() {
                            Err(Error::InvalidParameter(format!("Invalid parameter count when instantiating {}",template_name)))
                        } else  {
                            let mut template_scope = Scope::new(
                                true, Some(scope),
                                format!("{}:{}",self.current_file,meta.start)
                            );
                        
                            for n in 0..args.len() {
                                let value = self.eval_expression_p(scope, &*params[n])?;
                                template_scope.insert(args[n].clone(), ScopeValue::from(value));
                            }

                            let mut new_current_component = self.expand_full_name(component_name);
                            let mut new_current_file = template_path.to_string();

                            if let Some(component) = self.components.get_mut(&new_current_component) {
                                *component = Some(Component::default());

                                std::mem::swap(&mut new_current_file, &mut self.current_file);
                                std::mem::swap(&mut new_current_component, &mut self.current_component);

                                self.eval_statement_p(&mut template_scope, stmt)?;

                                std::mem::swap(&mut self.current_file, &mut new_current_file);
                                std::mem::swap(&mut self.current_component, &mut new_current_component);

                                Ok(())
                            } else {
                                Err(Error::NotFound(format!("component {}",&new_current_component)))
                            }
                        }
                    }
                    _ => Err(Error::NotFound(format!("template {}",template_name))),
                })
            } else {
                Err(Error::InvalidType(format!("component {} only can be initialized with template",&component_name)))
            }
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_variable(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        var: &VariableP
    ) -> Result<ReturnValue> {
        self.debug_trace(meta);

        let mut internal = || {
            scope.get(&var.name, |v| match v {
                Some(ScopeValue::Algebra(a)) => Ok(ReturnValue::Algebra(a.clone())),

                Some(ScopeValue::Bool(a)) => Ok(ReturnValue::Bool(*a)),
                
                Some(ScopeValue::Array(a)) => match var.sels.len() {
                    0 => Ok(ReturnValue::Array(a.clone())),
                    1 => if let SelectorP::Index{pos,..} = &*var.sels[0] {
                            let pos = self.eval_expression_p(scope, &pos)?.into_u64()? as usize;
                            if pos  < a.len() {
                                Ok(ReturnValue::Algebra(a[pos].clone()))
                            } else {
                                Err(Error::InvalidSelector("index overflow".to_string()))
                            }
                        } else {
                            Err(Error::InvalidSelector("needs index".to_string()))
                        },
                    _ => Err(Error::InvalidSelector("array needs only one index".to_string()))
                },
                
                None => {
                    let name = self.expand_selectors(scope,var)?;
                    if let Some(signal) = self.signals.get_by_name(&self.expand_full_name(&name)) {
                        if let Some(algebra::Value::FieldScalar(value)) = &signal.value {
                            Ok(ReturnValue::Algebra(algebra::Value::FieldScalar(value.clone())))
                        } else {
                            ReturnValue::from_signal_id(signal.id)
                        }
                    } else {
                        Err(Error::InvalidType(format!("Variable '{}' not found",&name)))
                    }
                }
                _ => Err(Error::InvalidType(format!("Variable '{:?}' cannot be used",&var.name)))
            })
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_number(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        n: &BigInt
    ) -> Result<ReturnValue> {
        self.debug_trace(meta);

        let internal = || {
            Ok(ReturnValue::Algebra(algebra::Value::from(n)))
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_prefix_op(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        op: Opcode,
        rhe: &ExpressionP
    ) -> Result<ReturnValue> {
        self.debug_trace(meta);

        let mut internal = || {
            let right = self.eval_expression_p(&scope, &rhe)?.into_algebra()?;
            Ok(ReturnValue::Algebra(self.alg_eval_prefix(meta,scope,op, &right)?))
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_infix_op(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        lhe: &ExpressionP,
        op: Opcode, 
        rhe: &ExpressionP,
    ) -> Result<ReturnValue> {
        self.debug_trace(meta);

        let mut internal = || {
            let left = self.eval_expression_p(&scope, &lhe)?;
            let right = self.eval_expression_p(&scope, &rhe)?;

            use Opcode::*;
            use ReturnValue::*;
            use algebra::Value::*;

            match op {
                Add | Sub | Mul | Div | IntDiv | Mod | ShiftL | ShiftR | BitAnd | BitOr | BitXor | Pow => {
                    let left = left.into_algebra()?;
                    let right = right.into_algebra()?;
                    Ok(ReturnValue::Algebra(self.alg_eval_infix(meta,scope,&left, op, &right)?))
                }
                BoolAnd => {
                    Ok(Bool(left.into_bool()? && right.into_bool()?))
                }
                BoolOr => {
                    Ok(Bool(left.into_bool()? || right.into_bool()?))
                }
                Greater => {
                    Ok(Bool(left.into_fs()?.0 > right.into_fs()?.0))
                }
                GreaterEq => {
                    Ok(Bool(left.into_fs()?.0 >= right.into_fs()?.0))
                }
                Lesser => {
                    Ok(Bool(left.into_fs()?.0 < right.into_fs()?.0))
                }
                LesserEq => {
                    Ok(Bool(left.into_fs()?.0 <= right.into_fs()?.0))
                }            
                Eq => match (&left,&right) {
                    (Bool(l),Bool(r)) => Ok(Bool(l == r)), 
                    (Algebra(FieldScalar(l)),Algebra(FieldScalar(r))) =>  Ok(Bool(l == r)),
                    _ => Err(Error::InvalidType(format!("Cannot compare {:?}=={:?}",left,right))),
                }
                NotEq => match (&left,&right) {
                    (Bool(l),Bool(r)) => Ok(Bool(l != r)), 
                    (Algebra(FieldScalar(l)),Algebra(FieldScalar(r))) =>  Ok(Bool(l != r)),
                    _ => Err(Error::InvalidType(format!("Cannot compare {:?}=={:?}",left,right))),
                }
                _ => Err(Error::NotYetImplemented(format!("eval_infix_op '{:?}'",op)))
            }
        };

        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_array(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        exprs: &[Box<ExpressionP>]
    ) -> Result<ReturnValue> {
        self.debug_trace(meta);

        let mut internal = || {
            let mut out : Vec<algebra::Value> = Vec::new();
            for expr in exprs.iter() {
                out.push(self.eval_expression_p(scope, expr)?.into_algebra()?);
            }
            Ok(ReturnValue::Array(out))
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_if_then_else(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        xif: &ExpressionP,
        xthen: &StatementP,
        xelse: &Option<Box<StatementP>>,
    ) -> Result<()> {
        self.debug_trace(meta);
        if !self.mode.must_process_eval(&meta) {
            return Ok(())
        }

        let mut internal = || { 
            use ReturnValue::*;
            match (self.eval_expression_p(scope, xif)?,xelse) {
                (Bool(true),_)  => self.eval_statement_p(scope, xthen), 
                (Bool(false),Some(xelse)) => self.eval_statement_p(scope, xelse),
                (Bool(false),None) => Ok(()),
                _ => Err(Error::InvalidType("if condition is not boolean".to_string()))
            }
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_for(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        init: &StatementP,
        cond: &ExpressionP,
        step: &StatementP,
        stmt: &StatementP,
    ) -> Result<()> {
        self.debug_trace(meta);
        if !self.mode.must_process_eval(&meta) {
            return Ok(())
        }

        let mut scope = Scope::new(
            false, Some(scope),
            format!("{}:{}",self.current_file,meta.start)
        );

        let mut internal = || {
            self.eval_statement_p(&mut scope,init)?;
            loop {
                use ReturnValue::*;
                match self.eval_expression_p(&scope, cond)? {
                    Bool(true)  => {}, 
                    Bool(false) => break,
                    _ => {
                        return Err(Error::InvalidType("for loop condition is not boolean".to_string()));
                    } 
                }
                self.eval_statement_p(&mut scope, stmt)?;
                if scope.has_return() {
                    break;
                }
                self.eval_statement_p(&mut scope, step)?;
            }
            Ok(())
        };
        let res = internal();
        self.register_error(meta,&scope,res)
    }

    fn eval_while(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        cond: &ExpressionP,
        stmt: &StatementP
    ) -> Result<()> {

        self.debug_trace(meta);
        if !self.mode.must_process_eval(&meta) {
            return Ok(())
        }
        
        let mut scope = Scope::new(
            false, Some(scope),
            format!("{}:{}",self.current_file,meta.start)
        );

        let mut internal = || {
            loop {
                use ReturnValue::*;
                match self.eval_expression_p(&scope, cond)? {
                    Bool(true)  => {}, 
                    Bool(false) => break,
                    _ => {
                        return Err(Error::InvalidType("while loop condition is not boolean".to_string()));
                    }
                }
                self.eval_statement_p(&mut scope, stmt)?;
                if scope.has_return() {
                    break;
                }
            }
            Ok(())
        };
        
        let res = internal();
        self.register_error(meta,&scope,res)
    }

    fn eval_return(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        expr: &ExpressionP
    ) -> Result<()> {

        self.debug_trace(meta);
        if !self.mode.must_process_eval(&meta) {
            return Ok(())
        }

        let mut internal = || {
            scope.set_return(self.eval_expression_p(scope, expr)?);
            Ok(())
        };

        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_declaration(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        xtype: VariableType,
        var: &VariableP,
        init: &Option<(Opcode, Box<ExpressionP>)>,
    ) -> Result<()> {
        self.debug_trace(meta);

        if !self.mode.must_process_eval(&meta) {
            return Ok(())
        }

        if self.current_component.is_empty() && !self.mode.must_process_root_decrl() {
            return Ok(());
        }

        let mut internal = || {
            if scope.contains_key(&var.name) {
                return Err(Error::AlreadyExists(var.name.clone()));
            }

            match (xtype, init) {
                
                (VariableType::Var, None) => {
                    match var.sels.len() {
                        0 => {
                            scope.insert(var.name.clone(),ScopeValue::Undefined);
                            Ok(())
                        }
                        1 => if let SelectorP::Index{pos,..} = &*var.sels[0] {
                                let size = self.eval_expression_p(scope, &pos)?.into_u64()? as usize;
                                let mut array = Vec::new();
                                (0..size).for_each(|_| array.push(algebra::Value::default()));
                                scope.insert(var.name.clone(),ScopeValue::Array(array));
                                Ok(())
                            } else {
                                Err(Error::InvalidSelector("needs [size]".to_string()))
                            },
                        _ => Err(Error::InvalidSelector("array needs only one index".to_string()))
                    }
                }
                
                (VariableType::Var, Some(init)) => {
                    let value = self.eval_expression_p(&scope, &*init.1)?;
                    match (init.0, value) {
                        (Opcode::Assig, ReturnValue::Algebra(n)) => {
                            scope.insert(var.name.clone(),ScopeValue::Algebra(n));
                            Ok(())
                        }
                        (Opcode::Assig, ReturnValue::Bool(b)) => {
                            scope.insert(var.name.clone(),ScopeValue::Bool(b));
                            Ok(())
                        }
                        (Opcode::Assig, ReturnValue::Array(a)) => {
                            scope.insert(var.name.clone(),ScopeValue::Array(a));
                            Ok(())
                        }
                        _ => Err(Error::InvalidType(format!("Unsupported type for var '{}' declaration",&var.name))),
                    }
                }

                (VariableType::Component, Some(init)) => {
                    self.eval_component_decl(meta, &scope, &var)?;
                    let var_w_selectors = self.expand_selectors(scope,var)?;
                    self.eval_component_instantiation(meta, &scope, &var_w_selectors, &*init.1)?;
                    Ok(())
                }

                (VariableType::Component, None) => {
                    self.eval_component_decl(meta, &scope, &var)?;
                    Ok(())
                }

                (VariableType::Signal(SignalType::Test), Some((_,expr))) => {
                    let value = self.eval_expression_p(scope, expr)?.into_fs()?;
                    self.signals.insert(format!("test.{:?}",var), SignalType::Test,Some(algebra::Value::from(value)));
                    Ok(())
                }

                (VariableType::Signal(xtype), None) => {

                    for signal_name in self.generate_selectors(scope, &var)? {

                        let full_name = self.expand_full_name(&signal_name);

                        if let Some(s) = self.signals.get_by_name(&full_name) {
                            if s.xtype == SignalType::Test {
                                continue;
                            } else {
                                return Err(Error::AlreadyExists(format!("signal {}",full_name)));
                            }     
                        }    

                        let signal_id = self.signals.insert(full_name,xtype,None);

                        if let Some(Some(component)) = self.components.get_mut(&self.current_component) {
                            component.signal_ids.push(signal_id);
                        } else {
                            panic!(format!("'{}' not initialized",&self.current_component));
                        }
                    }
                    Ok(())
                }
                _ =>  Err(Error::NotYetImplemented("eval_declaration_b".to_string())),
            }
        };

        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_substitution(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        var: &VariableP,
        op: Opcode,
        expr: &ExpressionP,
    ) -> Result<()> {
        self.debug_trace(meta);
        if !self.mode.must_process_eval(&meta) {
            return Ok(())
        }

        let mut internal = || {

            // check if is a component
            let var_sel = self.expand_selectors(scope,var)?;
            let var_full = self.expand_full_name(&var_sel);
            if self.components.contains_key(&var_full) {
                self.eval_component_instantiation(meta, scope,&var_sel,expr)?;
                return Ok(())
            }

            // check for variables
            let right = self.eval_expression_p(&scope, &expr)?.into_algebra()?;
            let value = if op == Opcode::Assig {
                right
            } else {
                let left = self.eval_variable(meta, scope, var)?.into_algebra()?;
                use Opcode::*;
                match op {
                    Assig        => right,
                    AssigAdd     => self.alg_eval_infix(meta, scope, &left, Add, &right)?,
                    AssigSub     => self.alg_eval_infix(meta, scope,&left, Sub, &right)?,
                    AssigMul     => self.alg_eval_infix(meta, scope,&left, Mul, &right)?,
                    AssigDiv     => self.alg_eval_infix(meta, scope,&left, Div, &right)?,
                    AssigMod     => self.alg_eval_infix(meta, scope,&left, Mod, &right)?,
                    AssigShiftL  => self.alg_eval_infix(meta, scope,&left, ShiftL, &right)?,
                    AssigShiftR  => self.alg_eval_infix(meta, scope,&left, ShiftR, &right)?,
                    AssigBitAnd  => self.alg_eval_infix(meta, scope,&left, BitAnd, &right)?,
                    AssigBitOr   => self.alg_eval_infix(meta, scope,&left, BitOr, &right)?,
                    AssigBitXor  => self.alg_eval_infix(meta, scope,&left, BitXor, &right)?,
                    _ => unreachable!(),
                }
            };

            if var.sels.is_empty() {
                scope.update(&var.name,ScopeValue::Algebra(value))?;
            } else if let SelectorP::Index{pos,..} = &*var.sels[0] {
                let pos = self.eval_expression_p(scope, &pos)?.into_u64()? as usize;
                scope.get_mut(&var.name, |v| {
                    if let Some(ScopeValue::Array(a)) = v {
                        a[pos] = value;
                    }
                });
            }
            Ok(())
        };
        
        let res = internal();
        self.register_error(meta,scope,res)

    }

    fn eval_block(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        stmts: &[Box<StatementP>]
    ) -> Result<()> {
        self.debug_trace(meta);
        if !self.mode.must_process_eval(&meta) {
            return Ok(())
        }

        let mut internal = || {
            let mut scope = Scope::new(
                false, Some(scope),
                format!("{}:{}",self.current_file,meta.start)
            );

            for stmt in stmts {
                self.eval_statement_p(&mut scope, &stmt)?;
                if scope.has_return() {
                    break;
                }
            }
            Ok(())
        };

        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_signal_left(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        signal: &VariableP,
        op: Opcode, 
        expr: &ExpressionP,
    ) -> Result<()> {
        self.debug_trace(meta);
        
        // inv : op == Opcode::SignalContrainLeft || op == Opcode::SignalWireLeft 

        let mut internal = || {

            // We have different evaluation tactics depending if we are generating
            //   constrains or if we are running the tests.
            //
            //    S <== 1
            //
            // When generating constrains, first we generate the constrain, and then
            //  the variable is assigned
            //
            //    S === 1   // constrain generation
            //    S <-- 1
            //
            // When running tests, first we assign the value, and then run the constrain
            //  verification 
            //
            //    S <-- 1
            //    S === 1  // constrain verification
            //    

            if self.mode == Mode::GenConstraints {     
                if op == Opcode::SignalContrainLeft {
                    self.eval_signal_eq(meta, scope,&ExpressionP::Variable{meta:meta.clone(), name:Box::new(signal.clone())},expr)?;
                }
            }

            // both for <-- and <==, eval content
            let signal_sel = self.expand_selectors(scope, signal)?;
            let signal_full = self.expand_full_name(&signal_sel);
            if let Some(signal_id) = self.signals.get_by_name(&signal_full).map(|s| s.id) {
                if let Ok(v) = self.eval_expression_p(scope, expr) {
                    if let Some(signal) = self.signals.get_by_id_mut(signal_id) {
                        if let ReturnValue::Algebra(a) = v {
                            signal.value = Some(a);
                        }  else {
                            return Err(Error::InvalidType(format!("Cannot assign {:?} to signal",v)));
                        } 
                    }
                }
            } else {
                return Err(Error::NotFound(format!("Signal {}",signal_full)));
            }

            if self.mode == Mode::Test {     
                if op == Opcode::SignalContrainLeft {
                    self.eval_signal_eq(meta, scope,&ExpressionP::Variable{meta:meta.clone(), name:Box::new(signal.clone())},expr)?;
                }
            }
              
            Ok(())
        };

        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_signal_right(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        expr: &ExpressionP,
        op: Opcode,
        signal: &VariableP,
    ) -> Result<()> {
        self.debug_trace(meta);

        let mut internal = || {
            use Opcode::*;
            match op {
                SignalContrainRight => self.eval_signal_left(meta, scope,signal,SignalContrainLeft,expr),
                SignalWireRight => self.eval_signal_left(meta, scope,signal,SignalWireLeft,expr),
                _ => unreachable!()
            }
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }
    // when generating constrains eval_signal_eq adds a new constrain to the system
    // when verifying  cirtcuits  eval_signal_eq checks that the resulting QEQ is zero
    fn eval_signal_eq(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        lhe: &ExpressionP,
        rhe: &ExpressionP,
    ) -> Result<()> {
        self.debug_trace(meta);

        let mut internal = || {
            let left = self.eval_expression_p(&scope, &lhe)?.into_algebra()?;
            let right = self.eval_expression_p(&scope, &rhe)?.into_algebra()?;
            let constrain = self.alg_eval_infix(meta,scope,&left, Opcode::Sub, &right)?;

            if self.mode == Mode::Test {

                match constrain {
                    algebra::Value::FieldScalar(ref fs) if fs.is_zero() => { },
                    _ => return Err(Error::CannotTestConstrain(self.format_algebra(constrain)))
                }

            } else {

                let qeq = match constrain {
                    algebra::Value::FieldScalar(_) => return Err(Error::CannotGenerateConstrain(
                            format!("{}==={}",
                            self.format_algebra(left),
                            self.format_algebra(right))
                        )),
                    _ => constrain.into_qeq()
                };
                self.constrains.push(qeq);

            }

            Ok(())
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_include(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        path: &str
    ) -> Result<()> {
        self.debug_trace(meta);

        let mut internal = || {
        
            let mut code = String::new();
            if let Err(ioerr) = File::open(path).and_then(|ref mut file| file.read_to_string(&mut code)) {
                return Err(Error::Io(path.to_string(),ioerr));        
            }

            let mut hasher = Blake2b::new(64);
            hasher.update(&code.as_bytes());

            let hash = hasher.finalize();
            let hash_hex = hex::encode(hash.as_bytes());
            if !self.processed_files.iter().any(|h| h == &hash_hex) {

                self.processed_files.push(hash_hex);

                let mut new_current_file = path.to_string();
                std::mem::swap(&mut new_current_file, &mut self.current_file);
    
                match circom2_parser::parse(&code) {
                    Ok(elements) => self.eval_body_elements_p(&Meta::new(0,0,None), scope, &elements)?,
                    Err(circom2_parser::Error::ParseError(err,meta)) => {
                        let err : Result<()> = Err(Error::Parse(err));
                        return self.register_error(&meta,scope,err);
                    }
                }

                std::mem::swap(&mut self.current_file, &mut new_current_file);
            }

            Ok(())
        };

        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_function_def(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        name: &str,
        args: &[String],
        stmt: &StatementP,
    ) -> Result<()> {
        self.debug_trace(meta);

        let internal = || {
            scope.insert(
                name.to_string(),
                ScopeValue::Function(
                    args.to_vec(),
                    Box::new(stmt.clone()),
                    self.current_file.to_string()
                )
            );
            Ok(())
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_template_def(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        name: &str,
        args: &[String],
        stmt: &StatementP,
    ) -> Result<()> {
        self.debug_trace(meta);

        let internal = || {
            scope.insert(
                name.to_string(),
                ScopeValue::Template(
                    meta.attrs.clone(),
                    args.to_vec(),
                    Box::new(stmt.clone()),
                    self.current_file.clone()
                )
            );
            Ok(())
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    fn eval_body_elements_p(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        bes : &[BodyElementP]
    ) -> Result<()> {
        let mut internal = || {
            for be in bes {
                self.eval_body_element_p(scope, &be)?;
            }
            Ok(())
        };
        let res = internal();
        self.register_error(meta,scope,res)
    }

    // helpers  -------------------------------------------------------------------------------

    fn generate_selectors(&mut self, scope: &Scope, var: &VariableP) -> Result<Vec<String>> {
        // TODO: convert into iterable collection?
        fn generate_selectors_1(base_name: &str, sizes : &[u64], stack: &mut Vec<u64>, out : &mut Vec<String> ) {
            if !sizes.is_empty() {
                for i in 0..sizes[0] {
                    stack.push(i);
                    generate_selectors_1(base_name, &sizes[1..], stack, out);
                    stack.pop();
                }
            } else {
                let accessors = stack.iter().map(|i| format!("[{}]",i)).collect::<Vec<_>>().join("");
                out.push(format!("{}{}",base_name,accessors));
            }
        }

        let mut sizes : Vec<u64> = Vec::new(); 
        for selector in var.sels.iter() {
            if let SelectorP::Index{pos,..} = &**selector {
                sizes.push(self.eval_expression_p(scope, &*pos)?.into_u64()?);
            } else {
                return Err(Error::InvalidType(format!("selectors for {}",&var.name)));
            }
        }

        let mut stack : Vec<u64> = Vec::new();
        let mut out : Vec<String> = Vec::new(); 
        generate_selectors_1(&var.name,&sizes,&mut stack,&mut out);
        
        Ok(out)
    }

    fn expand_selectors(&mut self, scope: &Scope, v: &VariableP) -> Result<String> {
        let mut v_sel = v.name.clone();
        for selector in v.sels.iter() {
            match &**selector {
                SelectorP::Index{pos,..} => {
                    let index = self.eval_expression_p(scope, &*pos)?.into_u64()?;
                    v_sel.push_str(&format!("[{}]",index));
                }
                SelectorP::Pin{name,..} => {
                    v_sel.push_str(&format!(".{}",name));
                }
            }
        }
        Ok(v_sel)
    }

    fn expand_full_name(&self, s : &str) -> String {
        if self.current_component.is_empty() {
            s.to_string()
        } else {
            format!("{}.{}",self.current_component,s)
        }
    }
}
