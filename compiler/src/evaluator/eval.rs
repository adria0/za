use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use za_parser;
use za_parser::ast::{
    BodyElementP, ExpressionP, Meta, Opcode, SelectorP, SignalType, StatementP, VariableP,
    VariableType,
};

use blake2_rfc::blake2b::Blake2b;
use hex;
use itertools::Itertools;
use num_bigint::BigInt;

use crate::algebra;
use crate::algebra::{AlgZero, SignalId};
use crate::types::{Signals,Constraints};

use super::error::*;
use super::types::{List,ReturnValue};
use super::scope::*;

#[derive(Debug)]
pub struct ErrorContext {
    pub scope: String,
    pub meta: Meta,
    pub file: String,
    pub component: String,
    pub function: Option<String>,
}

#[derive(PartialEq, Debug)]
pub enum Mode {
    Collect,        // collect declarations
    GenConstraints, // generate R1CS
    GenWitness,     // generate witness
}

impl Mode {
    pub fn skip_eval(&self, meta: &Meta) -> bool {
        self == &Mode::GenConstraints && meta.attrs.has_tag_w()
    }
    pub fn must_process_root_decrl(&self) -> bool {
        self != &Mode::Collect
    }
}

pub struct Evaluator {
    // the current file, component and function being processed
    pub current_file: String,
    pub current_component: String,
    pub current_function: Option<String>,
    pub debug_last_constraint: std::time::Instant,

    // collected signals, constraints and components
    pub signals: Signals,
    pub constraints: Constraints,

    // processed includes
    pub processed_files: Vec<String>,
    pub collected_asts: Vec<BodyElementP>,

    // last got error
    pub last_error: Option<ErrorContext>,

    // evaluation mode
    pub mode: Mode,

    // default path
    pub path: PathBuf,

    // deferred signal values
    pub deferred_signal_values: HashMap<String, algebra::Value>,

    // turn on debugging
    pub debug: bool,
}

impl Evaluator {
    pub fn new(mode: Mode, signals: Signals, constraints: Constraints) -> Self {
        Self {
            signals,
            constraints,
            mode,
            current_file: "".to_string(),
            current_component: "".to_string(),
            current_function: None,
            debug_last_constraint: std::time::Instant::now(),
            processed_files: Vec::new(),
            collected_asts: Vec::new(),
            last_error: None,
            path: PathBuf::from("."),
            deferred_signal_values: HashMap::new(),
            debug: false,
        }
    }

    // public interface ---------------------------------------------------------------------------

    pub fn eval_inline(&mut self, scope: &mut Scope, code: &str) -> Result<()> {
        match za_parser::parse(&code) {
            Ok(mut elements) => {
                self.eval_body_elements_p(&Meta::new(0, 0, None), scope, &elements)?;
                self.collected_asts.append(&mut elements);
            }
            Err(za_parser::Error::ParseError(err, meta)) => {
                return self.register_error(&meta, &scope, Err(Error::Parse(err)));
            }
        }
        Ok(())
    }

    pub fn eval_template(&mut self, scope: &mut Scope, template_name: &str) -> Result<()> {
        let err_not_found = || Error::NotFound(format!("template {}", template_name));

        match &*scope.get(&template_name).ok_or_else(err_not_found)? {
            ScopeValue::Template { stmt, path, .. } => {
                let mut scope = Scope::new(true, Some(scope), path.to_string());
                Ok(self.eval_statement_p(&mut scope, &stmt)?)
            }
            _ => Err(err_not_found())
        }
    }

    pub fn eval_file(&mut self, path: &str, filename: &str) -> Result<Scope> {
        self.path = PathBuf::from(path);
        let mut scope = Scope::new(true, None, filename.to_string());
        self.eval_include(&Meta::new(0, 0, None), &mut scope, filename)?;
        Ok(scope)
    }

    pub fn eval_asts(&mut self, asts: &[BodyElementP]) -> Result<Scope> {
        let mut scope = Scope::new(true, None, "".to_string());

        use BodyElementP::*;
        for body_element in asts.iter() {
            match body_element {
                FunctionDef { meta, name, args, stmt }
                  => self.eval_function_def(meta, &mut scope, name, args, stmt)?,
                TemplateDef { meta, name, args, stmt }
                  => self.eval_template_def(meta, &mut scope, name, args, stmt)?,
                _ => (),
            }
        }
        for body_element in asts.iter() {
            if let Declaration { decl, .. } =  body_element {
                self.eval_statement_p(&mut scope, decl)?;
            }
        }
        Ok(scope)
    }

    pub fn set_deferred_value(&mut self, signal_full_name: String, value: algebra::Value) {
        self.deferred_signal_values.insert(signal_full_name, value);
    }

    // evaluators -----------------------------------------------------------------------------------

    fn register_error<T>(&mut self, meta: &Meta, scope: &Scope, res: Result<T>) -> Result<T> {
        if res.is_err() && self.last_error.is_none() {
            self.last_error = Some(ErrorContext {
                scope: format!("{:?}", scope),
                meta: meta.clone(),
                file: self.current_file.clone(),
                component: self.current_component.clone(),
                function: self.current_function.clone(),
            });
        }
        res
    }

    fn alg_eval_prefix(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        op: za_parser::ast::Opcode,
        rhv: &algebra::Value,
    ) -> Result<algebra::Value> {
        match algebra::eval_prefix(op, rhv) {
            Err(err) => self.register_error(meta, scope, Err(Error::Algebra(err))),
            Ok(v) => Ok(v),
        }
    }

    fn alg_eval_infix(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        lhv: &algebra::Value,
        op: za_parser::ast::Opcode,
        rhv: &algebra::Value,
    ) -> Result<algebra::Value> {
        match algebra::eval_infix(lhv, op, rhv) {
            Err(err) => self.register_error(meta, scope, Err(Error::Algebra(err))),
            Ok(v) => Ok(v),
        }
    }

    fn eval_expression_p(&mut self, scope: &Scope, v: &ExpressionP) -> Result<ReturnValue> {
        use za_parser::ast::ExpressionP::*;
        match v {
            FunctionCall { meta, name, args } => self.eval_function_call(meta, scope, name, args),
            Variable { meta, name } => self.eval_variable(meta, scope, name),
            Number { meta, value } => self.eval_number(meta, scope, value),
            PrefixOp { meta, op, rhe } => self.eval_prefix_op(meta, scope, *op, rhe),
            InfixOp { meta, lhe, op, rhe } => self.eval_infix_op(meta, scope, lhe, *op, rhe),
            Array { meta, values } => self.eval_array(meta, scope, values),
        }
    }

    fn eval_statement_p(&mut self, scope: &mut Scope, v: &StatementP) -> Result<()> {
        use za_parser::ast::StatementP::*;
        match v {
            IfThenElse {
                meta,
                xif,
                xthen,
                xelse,
            } => self.eval_if_then_else(meta, scope, xif, xthen, xelse),
            For {
                meta,
                init,
                cond,
                step,
                stmt,
            } => self.eval_for(meta, scope, init, cond, step, stmt),
            While { meta, cond, stmt } => self.eval_while(meta, scope, cond, stmt),
            Return { meta, value } => self.eval_return(meta, scope, value),
            Declaration {
                meta,
                xtype,
                name,
                init,
            } => self.eval_declaration(meta, scope, *xtype, name, init),
            Substitution {
                meta,
                name,
                op,
                value,
            } => self.eval_substitution(meta, scope, name, *op, value),
            Block { meta, stmts } => self.eval_block(meta, scope, stmts),
            SignalLeft {
                meta,
                name,
                op,
                value,
            } => self.eval_signal_left(meta, scope, name, *op, value),
            SignalRight {
                meta,
                value,
                op,
                name,
            } => self.eval_signal_right(meta, scope, value, *op, name),
            SignalEq { meta, lhe, rhe, .. } => self.eval_signal_eq(meta, scope, lhe, rhe),
            InternalCall { meta, name, args } => self.eval_internal_call(meta, scope, name, args),
        }
    }

    fn eval_body_element_p(&mut self, scope: &mut Scope, v: &BodyElementP) -> Result<()> {
        use za_parser::ast::BodyElementP::*;
        match v {
            Include { meta, path } => self.eval_include(meta, scope, path),
            FunctionDef {
                meta,
                name,
                args,
                stmt,
            } => self.eval_function_def(meta, scope, name, args, stmt),
            TemplateDef {
                meta,
                name,
                args,
                stmt,
            } => self.eval_template_def(meta, scope, name, args, stmt),
            Declaration { decl, .. } => self.eval_statement_p(scope, decl),
        }
    }

    fn eval_internal_call(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        name: &str,
        params: &[Box<ExpressionP>],
    ) -> Result<()> {
        self.trace(meta, || format!("eval_internal_call {}", name));

        let mut internal = || match name {
            "dbg_signals" => Ok(self.dbg_dump_signals()?),
            "dbg" => {
                print!("DBG ");
                for param in params {
                    self.dbg_print(&scope, param)?;
                }
                println!();
                Ok(())
            }
            _ => Err(Error::NotFound(format!("internal funcion {}!", name))),
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_function_call(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        name: &str,
        params: &[Box<ExpressionP>],
    ) -> Result<ReturnValue> {
        self.trace(meta, || format!("eval_function_call {}", name));

        let mut internal = || {
            let err_not_found = || Error::NotFound(format!("function {}", name));

            let function = scope.root().get(name).ok_or_else(err_not_found)?;
            let (args,stmt,path) = match &*function {
                ScopeValue::Function { args, stmt, path } => Ok((args,stmt,path)),
                _ => Err(err_not_found())
            }?;

            if args.len() != params.len() {
                return Err(Error::InvalidParameter(name.to_string()));
            }

            let mut func_scope = Scope::new(
                true,
                Some(scope),
                format!("{}:{}", self.current_file, meta.start),
            );

            for n in 0..args.len() {
                let value = self.eval_expression_p(scope, &*params[n])?;
                func_scope.insert(args[n].clone(), ScopeValue::from(value))?;
            }

            let mut new_current_function = Some(name.to_string());
            let mut new_current_file = path.to_string();

            std::mem::swap(&mut new_current_function, &mut self.current_function);
            std::mem::swap(&mut new_current_file, &mut self.current_file);

            self.eval_statement_p(&mut func_scope, stmt)?;

            std::mem::swap(&mut self.current_function, &mut new_current_function);
            std::mem::swap(&mut self.current_file, &mut new_current_file);

            func_scope
                .take_return()
                .ok_or_else(|| Error::BadFunctionReturn(name.to_string()))
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_component_decl(&mut self, _meta: &Meta, scope: &Scope, name: &VariableP) -> Result<()> {
        for selector_name in self.generate_selectors(scope, &name)? {
            scope.insert(selector_name, ScopeValue::UndefComponent)?;
        }
        Ok(())
    }

    fn eval_component_inst(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        component_name: &str,
        init: &ExpressionP,
    ) -> Result<()> {
        self.trace(meta, || format!("eval_component_inst {}", component_name));

        let mut internal = || {
            let (can_be_expanded_now,component) = {
                let err_invalid_template = || Error::InvalidType(format!("component {} only can be initialized with existingtemplate",&component_name));

                let (template_name,params) = match init {
                    ExpressionP::FunctionCall { name, args, ..} => Ok((name,args)),
                    _ => Err(err_invalid_template()),
                }?;
                let template = scope.root().get(template_name).ok_or_else(err_invalid_template)?;
                let (args,stmt,path) = match &*template {
                    ScopeValue::Template {args, stmt, path, ..} => Ok((args,stmt,path)),
                    _ => Err(err_invalid_template())
                }?;

                if args.len() != params.len() {
                    return Err(Error::InvalidParameter(format!(
                        "Invalid parameter count when instantiating {}",
                        template_name
                    )));
                }
                let mut evalargs = Vec::new();
                let mut all_pending_input_signals: Vec<SignalId> = Vec::new();

                // create a new scope, and put into arguments
                let mut template_scope = Scope::new(
                    true,
                    Some(scope),
                    format!("{}:{}", self.current_file, meta.start),
                );

                for n in 0..args.len() {
                    let value = self.eval_expression_p(scope, &*params[n])?;
                    evalargs.push(value.clone());
                    template_scope.insert(args[n].clone(), ScopeValue::from(value))?;
                }

                let mut new_current_component = self.expand_full_name(component_name);
                let mut new_current_file = path.to_string();

                std::mem::swap(&mut new_current_file, &mut self.current_file);
                std::mem::swap(&mut new_current_component, &mut self.current_component);

                if let StatementP::Block { stmts, .. } = &**stmt {
                    let signals = stmts
                        .iter()
                        .filter_map(|stmt| {
                            if let StatementP::Declaration {
                                meta,
                                name,
                                xtype: VariableType::Signal(xtype),
                                ..
                            } = &**stmt
                            {
                                Some((meta, name, xtype))
                            } else {
                                None
                            }
                        })
                        .sorted_by(|(_, _, xtype1), (_, _, xtype2)| {
                            Ord::cmp(xtype1, xtype2)
                        });

                    for (meta, name, xtype) in signals {
                        let mut pending_signals = self.eval_declaration_signals(
                            meta,
                            &mut template_scope,
                            *xtype,
                            name,
                        )?;
                        let is_public_or_private = *xtype == SignalType::PublicInput || *xtype == SignalType::PrivateInput;
                        let is_not_main_in_genconstraints =!(component_name == "main" && self.mode == Mode::GenConstraints); 
                        if is_public_or_private && is_not_main_in_genconstraints {
                            all_pending_input_signals.append(&mut pending_signals);
                        }
                    }
                } else {
                    unreachable!();
                }
                std::mem::swap(&mut self.current_file, &mut new_current_file);
                std::mem::swap(&mut self.current_component, &mut new_current_component);

                (
                    all_pending_input_signals.is_empty(),
                    ScopeValue::Component {
                        template: template_name.to_string(),
                        path: path.to_string(),
                        args: evalargs,
                        pending_inputs: all_pending_input_signals,
                    }
                )
            };
            {
                let mut scope_component = scope
                    .get_mut(component_name)
                    .ok_or_else(|| Error::NotFound(component_name.to_string()))?;
                
                *scope_component = component;
            }
            
            if can_be_expanded_now {
                self.eval_component_expand(meta, scope, component_name)?;
            }
            Ok(())
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_component_expand(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        component_name: &str,
    ) -> Result<()> {
        self.trace(meta, || format!("eval_component_expand {}", component_name));

        let component = scope.get(component_name).unwrap();
        let (template,values) = match &*component {
            ScopeValue::Component{template,args,..} => (template,args),
            _ => unreachable!(),
        };

        let template = scope.root().get(template).unwrap();
        let (args,stmt,path) = match &*template {
            ScopeValue::Template {args, stmt, path,..} => (args,stmt,path),
            _ => unreachable!(),
        };

        // put arguments in scope
        let mut template_scope = Scope::new(
            true,
            Some(scope),
            format!("{}:{}", self.current_file, meta.start),
        );
        for n in 0..args.len() {
            template_scope
                .insert(args[n].clone(), ScopeValue::from(values[n].clone()))?;
        }

        // set new component & file scope
        let mut new_current_component = self.expand_full_name(component_name);
        let mut new_current_file = path.to_string();

        std::mem::swap(&mut new_current_file, &mut self.current_file);
        std::mem::swap(&mut new_current_component, &mut self.current_component);

        // execute the template
        self.eval_statement_p(&mut template_scope, stmt)?;

        // revert previous state
        std::mem::swap(&mut self.current_file, &mut new_current_file);
        std::mem::swap(&mut self.current_component, &mut new_current_component);

        Ok(())
    }

    fn eval_variable(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        var: &VariableP,
    ) -> Result<ReturnValue> {
        let mut internal = || {
            // check if is a signal
            let name_sel = self.expand_selectors(scope, var, None)?;
            let name_sel_full = &self.expand_full_name(&name_sel);
            if let Some(signal) = self.signals.get_by_name(&name_sel_full) {
                if let Some(algebra::Value::FieldScalar(value)) = &signal.value {
                    return Ok(ReturnValue::Algebra(algebra::Value::FieldScalar(
                        value.clone(),
                    )));
                } else {
                    return ReturnValue::from_signal_id(signal.id);
                }
            }

            // check if is a variable
            let scope_var = scope.get(&var.name).ok_or_else(|| Error::NotFound(name_sel.clone()))?;
            match &*scope_var {
                ScopeValue::Algebra(a) => Ok(ReturnValue::Algebra(a.clone())),
                ScopeValue::Bool(a) => Ok(ReturnValue::Bool(*a)),
                ScopeValue::List(l) => {
                    let mut indexes = Vec::new();
                    for sel in &var.sels {
                        match &**sel {
                            SelectorP::Index { pos, .. } => {
                                indexes
                                    .push(self.eval_expression_p(scope, pos)?.try_into_u64()?
                                        as usize);
                            }
                            _ => {
                                return Err(Error::InvalidSelector(format!(
                                    "Invalid selector {:?}",
                                    sel
                                )));
                            }
                        }
                    }
                    match l.get(&indexes)? {
                        List::Algebra(a) => Ok(ReturnValue::Algebra(a.clone())),
                        List::List(l) => Ok(ReturnValue::List(List::List(l.clone()))),
                    }
                }
                other => Err(Error::InvalidType(format!(
                    "expected valid value from variable '{}' (current is '{:?}') [nameselfull={}]",
                    name_sel, other, name_sel_full
                ))),
            }
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_number(&mut self, meta: &Meta, scope: &Scope, n: &BigInt) -> Result<ReturnValue> {
        let internal = || Ok(ReturnValue::Algebra(algebra::Value::from(n)));
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_prefix_op(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        op: Opcode,
        rhe: &ExpressionP,
    ) -> Result<ReturnValue> {
        let mut internal = || {
            let right = self.eval_expression_p(&scope, &rhe)?.try_into_algebra()?;
            Ok(ReturnValue::Algebra(
                self.alg_eval_prefix(meta, scope, op, &right)?,
            ))
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_infix_op(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        lhe: &ExpressionP,
        op: Opcode,
        rhe: &ExpressionP,
    ) -> Result<ReturnValue> {
        let mut internal = || {
            let left = self.eval_expression_p(&scope, &lhe)?;
            let right = self.eval_expression_p(&scope, &rhe)?;

            use algebra::Value::*;
            use Opcode::*;
            use ReturnValue::*;

            match op {
                Add | Sub | Mul | Div | IntDiv | Mod | ShiftL | ShiftR | BitAnd | BitOr
                | BitXor | Pow => {
                    let left = left.try_into_algebra()?;
                    let right = right.try_into_algebra()?;
                    Ok(ReturnValue::Algebra(
                        self.alg_eval_infix(meta, scope, &left, op, &right)?,
                    ))
                }
                BoolAnd => Ok(Bool(left.try_into_bool()? && right.try_into_bool()?)),
                BoolOr => Ok(Bool(left.try_into_bool()? || right.try_into_bool()?)),
                Greater => Ok(Bool(left.try_into_fs()? > right.try_into_fs()?)),
                GreaterEq => Ok(Bool(left.try_into_fs()? >= right.try_into_fs()?)),
                Lesser => Ok(Bool(left.try_into_fs()? < right.try_into_fs()?)),
                LesserEq => Ok(Bool(left.try_into_fs()? <= right.try_into_fs()?)),
                Eq => match (&left, &right) {
                    (Bool(l), Bool(r)) => Ok(Bool(l == r)),
                    (Algebra(FieldScalar(l)), Algebra(FieldScalar(r))) => Ok(Bool(l == r)),
                    _ => Err(Error::InvalidType(format!(
                        "Cannot compare {:?}=={:?}",
                        left, right
                    ))),
                },
                NotEq => match (&left, &right) {
                    (Bool(l), Bool(r)) => Ok(Bool(l != r)),
                    (Algebra(FieldScalar(l)), Algebra(FieldScalar(r))) => Ok(Bool(l != r)),
                    _ => Err(Error::InvalidType(format!(
                        "Cannot compare {:?}=={:?}",
                        left, right
                    ))),
                },
                _ => Err(Error::NotYetImplemented(format!(
                    "eval_infix_op '{:?}'",
                    op
                ))),
            }
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_array(
        &mut self,
        meta: &Meta,
        scope: &Scope,
        exprs: &[Box<ExpressionP>],
    ) -> Result<ReturnValue> {
        let mut internal = || {
            let out = exprs
                .iter()
                .map(|expr| {
                    Ok(match self.eval_expression_p(scope, expr)? {
                        ReturnValue::Algebra(a) => List::Algebra(a),
                        ReturnValue::List(l) => l,
                        _ => unreachable!(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ReturnValue::List(List::List(out)))
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_if_then_else(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        xif: &ExpressionP,
        xthen: &StatementP,
        xelse: &Option<Box<StatementP>>,
    ) -> Result<()> {
        if self.mode.skip_eval(&meta) {
            return Ok(());
        }

        let mut internal = || {
            use ReturnValue::*;
            match (self.eval_expression_p(scope, xif)?, xelse) {
                (Bool(true), _) => self.eval_statement_p(scope, xthen),
                (Bool(false), Some(xelse)) => self.eval_statement_p(scope, xelse),
                (Bool(false), None) => Ok(()),
                _ => Err(Error::InvalidType(
                    "if condition is not boolean".to_string(),
                )),
            }
        };
        let res = internal();
        self.register_error(meta, scope, res)
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
        if self.mode.skip_eval(&meta) {
            return Ok(());
        }

        let mut scope = Scope::new(
            false,
            Some(scope),
            format!("{}:{}", self.current_file, meta.start),
        );

        let mut internal = || {
            self.eval_statement_p(&mut scope, init)?;
            loop {
                use ReturnValue::*;
                match self.eval_expression_p(&scope, cond)? {
                    Bool(true) => {}
                    Bool(false) => break,
                    _ => {
                        return Err(Error::InvalidType(
                            "for loop condition is not boolean".to_string(),
                        ));
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
        self.register_error(meta, &scope, res)
    }

    fn eval_while(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        cond: &ExpressionP,
        stmt: &StatementP,
    ) -> Result<()> {
        if self.mode.skip_eval(&meta) {
            return Ok(());
        }

        let mut scope = Scope::new(
            false,
            Some(scope),
            format!("{}:{}", self.current_file, meta.start),
        );

        let mut internal = || {
            loop {
                use ReturnValue::*;
                match self.eval_expression_p(&scope, cond)? {
                    Bool(true) => {}
                    Bool(false) => break,
                    _ => {
                        return Err(Error::InvalidType(
                            "while loop condition is not boolean".to_string(),
                        ));
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
        self.register_error(meta, &scope, res)
    }

    fn eval_return(&mut self, meta: &Meta, scope: &mut Scope, expr: &ExpressionP) -> Result<()> {
        self.trace(meta, || "eval_return".to_string());

        if self.mode.skip_eval(&meta) {
            return Ok(());
        }

        let mut internal = || {
            scope.set_return(self.eval_expression_p(scope, expr)?);
            Ok(())
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_declaration_signals(
        &mut self,
        _meta: &Meta,
        scope: &mut Scope,
        xtype: SignalType,
        var: &VariableP,
    ) -> Result<Vec<SignalId>> {
        let mut pending_signals = Vec::new();

        for signal_name in self.generate_selectors(scope, &var)? {
            let full_name = self.expand_full_name(&signal_name);
            if self.signals.get_by_name(&full_name).is_some() {
                return Err(Error::AlreadyExists(format!("signal {}", full_name)));
            }
            if let Some(v) = self.deferred_signal_values.remove(&full_name) {
                self.signals.insert(full_name, xtype, Some(v));
            } else {
                let signal_id = self.signals.insert(full_name, xtype, None);
                pending_signals.push(signal_id);
            }
        }

        Ok(pending_signals)
    }

    fn eval_declaration(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        xtype: VariableType,
        var: &VariableP,
        init: &Option<(Opcode, Box<ExpressionP>)>,
    ) -> Result<()> {
        if self.mode.skip_eval(&meta) {
            return Ok(());
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
                            // var a;
                            scope.insert(var.name.clone(), ScopeValue::UndefVar)?;
                            Ok(())
                        }
                        _ => {
                            let sizes = self.expand_indexes(scope, &var.sels)?;
                            scope.insert(var.name.clone(), ScopeValue::List(List::new(&sizes)))?;
                            Ok(())
                        }
                    }
                }

                (VariableType::Var, Some(init)) => {
                    let value = self.eval_expression_p(&scope, &*init.1)?;
                    match (init.0, value) {
                        (Opcode::Assig, ReturnValue::Algebra(n)) => {
                            scope.insert(var.name.clone(), ScopeValue::Algebra(n))?;
                            Ok(())
                        }
                        (Opcode::Assig, ReturnValue::Bool(b)) => {
                            scope.insert(var.name.clone(), ScopeValue::Bool(b))?;
                            Ok(())
                        }
                        (Opcode::Assig, ReturnValue::List(a)) => {
                            scope.insert(var.name.clone(), ScopeValue::List(a))?;
                            Ok(())
                        }
                        _ => Err(Error::InvalidType(format!(
                            "Unsupported type for var '{}' declaration",
                            &var.name
                        ))),
                    }
                }

                (VariableType::Component, Some(init)) => {
                    self.eval_component_decl(meta, &scope, &var)?;
                    let var_w_selectors = self.expand_selectors(scope, var, None)?;
                    self.eval_component_inst(meta, &scope, &var_w_selectors, &*init.1)?;
                    Ok(())
                }

                (VariableType::Component, None) => {
                    self.eval_component_decl(meta, &scope, &var)?;
                    Ok(())
                }

                (VariableType::Signal(_), None) => Ok(()),
                _ => Err(Error::NotYetImplemented(format!(
                    "eval_declaration {:?}",
                    var
                ))),
            }
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_substitution(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        var: &VariableP,
        op: Opcode,
        expr: &ExpressionP,
    ) -> Result<()> {
        if self.mode.skip_eval(&meta) {
            return Ok(());
        }

        let mut internal = || {

            // check if we are instatianting a UndefComponent
            let var_sel = self.expand_selectors(scope, var, None)?;

            let is_undef_component = scope.get(&var_sel)
                .map(|v| if let ScopeValue::UndefComponent = &*v {
                    true
                } else {
                    false
                })
                .unwrap_or(false);

            if is_undef_component {
                self.eval_component_inst(meta, &scope, &var_sel, expr)?;
                return Ok(());
            }

            // check for variables
            let right = self.eval_expression_p(&scope, &expr)?.try_into_algebra()?;
            let value = if op == Opcode::Assig {
                right
            } else {
                let left = self.eval_variable(meta, scope, var)?.try_into_algebra()?;
                use Opcode::*;
                match op {
                    Assig => right,
                    AssigAdd => self.alg_eval_infix(meta, scope, &left, Add, &right)?,
                    AssigSub => self.alg_eval_infix(meta, scope, &left, Sub, &right)?,
                    AssigMul => self.alg_eval_infix(meta, scope, &left, Mul, &right)?,
                    AssigDiv => self.alg_eval_infix(meta, scope, &left, Div, &right)?,
                    AssigMod => self.alg_eval_infix(meta, scope, &left, Mod, &right)?,
                    AssigShiftL => self.alg_eval_infix(meta, scope, &left, ShiftL, &right)?,
                    AssigShiftR => self.alg_eval_infix(meta, scope, &left, ShiftR, &right)?,
                    AssigBitAnd => self.alg_eval_infix(meta, scope, &left, BitAnd, &right)?,
                    AssigBitOr => self.alg_eval_infix(meta, scope, &left, BitOr, &right)?,
                    AssigBitXor => self.alg_eval_infix(meta, scope, &left, BitXor, &right)?,
                    _ => unreachable!(),
                }
            };

            if var.sels.is_empty() {
                scope.update(&var.name, ScopeValue::Algebra(value))?;
            } else if let SelectorP::Index { .. } = &*var.sels[0] {
                let indexes = self.expand_indexes(scope, &var.sels)?;
                let mut scope_var = scope.get_mut(&var.name).ok_or_else(|| Error::NotFound(var.name.clone()))?;
                match &mut *scope_var {
                    ScopeValue::List(ref mut l) => l.set(&value, &indexes),
                    _ => Err(Error::InvalidType(var.name.clone())),
                }?;
            }
            Ok(())
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_block(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        stmts: &[Box<StatementP>],
    ) -> Result<()> {
        if self.mode.skip_eval(&meta) {
            return Ok(());
        }

        let mut internal = || {
            let mut scope = Scope::new(
                false,
                Some(scope),
                format!("{}:{}", self.current_file, meta.start),
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
        self.register_error(meta, scope, res)
    }

    fn eval_signal_left(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        signal: &VariableP,
        op: Opcode,
        expr: &ExpressionP,
    ) -> Result<()> {
        self.trace(meta, || format!("eval_signal_left {:?}", signal));

        // inv : op == Opcode::SignalContrainLeft || op == Opcode::SignalWireLeft

        let mut internal = || {
            // We have different evaluation tactics depending if we are generating
            //   constraints or if we are running the tests.
            //
            //    S <== 1
            //
            // When generating constraints, first we generate the constrain, and then
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

            // eval == iff in GenContraints
            if self.mode == Mode::GenConstraints && op == Opcode::SignalContrainLeft {
                self.eval_signal_eq(
                    meta,
                    scope,
                    &ExpressionP::Variable {
                        meta: meta.clone(),
                        name: Box::new(signal.clone()),
                    },
                    expr,
                )?;
            }

            // eval <--
            if !self.mode.skip_eval(meta) {
                self.trace(meta, || format!("eval_signal_left <-- {:?}", signal));
                let signal_sel = self.expand_selectors(scope, signal, None)?;
                let signal_full = self.expand_full_name(&signal_sel);
                if let Some(signal_id) = self.signals.get_by_name(&signal_full).map(|s| s.id) {
                    // set the signal valuesignal_elementsignal_element
                    let v = self.eval_expression_p(scope, expr)?;
                    if let ReturnValue::Algebra(a) = v {
                        self.signals.update(signal_id, a);
                    } else {
                        return Err(Error::InvalidType(format!(
                            "Cannot assign {:?} to signal",
                            v
                        )));
                    }

                    if let Some(component_name) = self.signal_component(scope, signal)? {
                        let err_not_found = || Error::NotFound(format!("signal not found '{}' in scope {:?}", signal.name, meta));
                        
                        let needs_expansion = {
                            let mut component = scope.get_mut(&component_name).ok_or_else(err_not_found)?;
                            match &mut *component {
                                ScopeValue::Component { pending_inputs, .. } => {
                                    if !pending_inputs.is_empty() {
                                        pending_inputs.retain(|s| *s != signal_id);
                                        Ok(pending_inputs.is_empty())
                                    } else {
                                        Ok(false)
                                    }
                                },
                                _ => Err(err_not_found())
                            }
                        }?;

                        // if all input signals has been set, then expand the component
                        if needs_expansion {
                            self.trace(meta, || {
                                format!("eval_signal_left_lazy_eval {}", component_name)
                            });
                            self.eval_component_expand(meta, scope, &component_name)?;
                        }
                    }
                } else {
                    return Err(Error::NotFound(format!("Signal {}", signal_full)));
                }
            }

            // eval == iff when generating witness
            if self.mode == Mode::GenWitness && op == Opcode::SignalContrainLeft {
                self.eval_signal_eq(
                    meta,
                    scope,
                    &ExpressionP::Variable {
                        meta: meta.clone(),
                        name: Box::new(signal.clone()),
                    },
                    expr,
                )?;
            }

            Ok(())
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_signal_right(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        expr: &ExpressionP,
        op: Opcode,
        signal: &VariableP,
    ) -> Result<()> {
        self.trace(meta, || format!("eval_signal_right {:?}", signal));
        if self.mode.skip_eval(&meta) {
            return Ok(());
        }
        let mut internal = || {
            use Opcode::*;
            match op {
                SignalContrainRight => {
                    self.eval_signal_left(meta, scope, signal, SignalContrainLeft, expr)
                }
                SignalWireRight => self.eval_signal_left(meta, scope, signal, SignalWireLeft, expr),
                _ => unreachable!(),
            }
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_signal_eq(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        lhe: &ExpressionP,
        rhe: &ExpressionP,
    ) -> Result<()> {
        self.trace(meta, || format!("eval_signal_eq {:?} {:?}", lhe, rhe));
        let mut internal = || {
            let left = self.eval_expression_p(&scope, &lhe)?.try_into_algebra()?;
            let right = self.eval_expression_p(&scope, &rhe)?.try_into_algebra()?;
            let constrain = self.alg_eval_infix(meta, scope, &left, Opcode::Sub, &right)?;

            if self.mode == Mode::GenWitness {
                // checks the constraintsTest
                match constrain {
                    algebra::Value::FieldScalar(ref fs) if fs.is_zero() => {}
                    _ => {
                        return Err(Error::CannotTestConstrain(format!(
                            "{:?}==={:?} => {}==={}",
                            lhe,
                            rhe,
                            self.signals.format(&left),
                            self.signals.format(&right)
                        )));
                    }
                }
            } else if self.mode == Mode::GenConstraints {
                // generates constraints
                let qeq = match constrain {
                    algebra::Value::FieldScalar(_) => {
                        return Err(Error::CannotGenerateConstrain(format!(
                            "{}==={}",
                            self.signals.format(&left),
                            self.signals.format(&right)
                        )));
                    }
                    _ => constrain.into_qeq(),
                };

                let count = if self.debug {
                    self.constraints
                        .push(qeq, Some(format!("{}:{}", self.current_file, meta.start)))
                } else {
                    self.constraints.push(qeq, None)
                };

                if count > 0 && count % 100_000 == 0 {
                    let now = std::time::Instant::now();
                    let diff = now.duration_since(self.debug_last_constraint);
                    println!(
                        "Generated {} constrains, @ {} c/s",
                        count,
                        100_000_000 / diff.as_millis()
                    );
                    self.debug_last_constraint = now;
                }
            }

            Ok(())
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_include(&mut self, meta: &Meta, scope: &mut Scope, filename: &str) -> Result<()> {
        let mut internal = || {
            let mut full_path = PathBuf::new();
            full_path.push(&self.path);
            full_path.push(&filename);

            let mut code = String::new();
            if let Err(ioerr) =
                File::open(&full_path).and_then(|ref mut file| file.read_to_string(&mut code))
            {
                return Err(Error::Io(format!("{:?}", full_path), ioerr.to_string()));
            }

            let mut hasher = Blake2b::new(64);
            hasher.update(&code.as_bytes());

            let hash = hasher.finalize();
            let hash_hex = hex::encode(hash.as_bytes());

            if !self.processed_files.iter().any(|h| h == &hash_hex) {
                self.processed_files.push(hash_hex);

                let mut new_current_file = full_path.to_str().unwrap().to_string();
                let mut new_path = full_path.parent().unwrap().to_path_buf();
                std::mem::swap(&mut new_current_file, &mut self.current_file);
                std::mem::swap(&mut new_path, &mut self.path);

                match za_parser::parse(&code) {
                    Ok(mut elements) => {
                        self.eval_body_elements_p(&Meta::new(0, 0, None), scope, &elements)?;
                        self.collected_asts.append(&mut elements);
                    }
                    Err(za_parser::Error::ParseError(err, meta)) => {
                        let err: Result<()> = Err(Error::Parse(err));
                        return self.register_error(&meta, scope, err);
                    }
                }

                std::mem::swap(&mut self.path, &mut new_path);
                std::mem::swap(&mut self.current_file, &mut new_current_file);
            }

            Ok(())
        };

        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_function_def(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        name: &str,
        args: &[String],
        stmt: &StatementP,
    ) -> Result<()> {
        let internal = || {
            scope.insert(
                name.to_string(),
                ScopeValue::Function {
                    args: args.to_vec(),
                    stmt: Box::new(stmt.clone()),
                    path: self.current_file.to_string(),
                },
            )?;
            Ok(())
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_template_def(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        name: &str,
        args: &[String],
        stmt: &StatementP,
    ) -> Result<()> {
        let internal = || {
            scope.insert(
                name.to_string(),
                ScopeValue::Template {
                    attrs: meta.attrs.clone(),
                    args: args.to_vec(),
                    stmt: Box::new(stmt.clone()),
                    path: self.current_file.clone(),
                },
            )?;
            Ok(())
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    fn eval_body_elements_p(
        &mut self,
        meta: &Meta,
        scope: &mut Scope,
        bes: &[BodyElementP],
    ) -> Result<()> {
        let mut internal = || {
            for be in bes {
                self.eval_body_element_p(scope, &be)?;
            }
            Ok(())
        };
        let res = internal();
        self.register_error(meta, scope, res)
    }

    // helpers  -------------------------------------------------------------------------------

    fn trace<F>(&self, meta: &Meta, f: F)
    where
        F: FnOnce() -> String,
    {
        if self.debug {
            println!("*trace {} {}:{:?}", f(), self.current_file, meta.start);
        }
    }

    fn generate_selectors(&mut self, scope: &Scope, var: &VariableP) -> Result<Vec<String>> {
        // TODO: convert into iterable collection?
        fn generate_selectors_1(
            base_name: &str,
            sizes: &[u64],
            stack: &mut Vec<u64>,
            out: &mut Vec<String>,
        ) {
            if !sizes.is_empty() {
                for i in 0..sizes[0] {
                    stack.push(i);
                    generate_selectors_1(base_name, &sizes[1..], stack, out);
                    stack.pop();
                }
            } else {
                let accessors = stack
                    .iter()
                    .map(|i| format!("[{}]", i))
                    .collect::<Vec<_>>()
                    .join("");
                out.push(format!("{}{}", base_name, accessors));
            }
        }

        let mut sizes: Vec<u64> = Vec::new();
        for selector in var.sels.iter() {
            if let SelectorP::Index { pos, .. } = &**selector {
                sizes.push(self.eval_expression_p(scope, &*pos)?.try_into_u64()?);
            } else {
                return Err(Error::InvalidType(format!("selectors for {}", &var.name)));
            }
        }

        let mut stack: Vec<u64> = Vec::new();
        let mut out: Vec<String> = Vec::new();
        generate_selectors_1(&var.name, &sizes, &mut stack, &mut out);

        Ok(out)
    }

    fn expand_selectors(
        &mut self,
        scope: &Scope,
        v: &VariableP,
        limit: Option<usize>,
    ) -> Result<String> {
        let mut v_sel = v.name.clone();

        for (i, selector) in v.sels.iter().enumerate() {
            if let Some(limit) = limit {
                if i == limit {
                    return Ok(v_sel);
                }
            }

            match &**selector {
                SelectorP::Index { pos, .. } => {
                    let index = self.eval_expression_p(scope, &*pos)?.try_into_u64()?;
                    v_sel.push_str(&format!("[{}]", index));
                }
                SelectorP::Pin { name, .. } => {
                    v_sel.push_str(&format!(".{}", name));
                }
            }
        }
        Ok(v_sel)
    }

    fn expand_indexes(&mut self, scope: &Scope, sels: &[Box<SelectorP>]) -> Result<Vec<usize>> {
        let mut indexes = Vec::new();
        for sel in sels {
            match &**sel {
                SelectorP::Index { pos, .. } => {
                    indexes.push(self.eval_expression_p(scope, pos)?.try_into_u64()? as usize);
                }
                _ => {
                    return Err(Error::InvalidSelector(format!(
                        "Invalid selector {:?}",
                        sel
                    )));
                }
            }
        }
        Ok(indexes)
    }

    fn signal_component(&mut self, scope: &Scope, signal: &VariableP) -> Result<Option<String>> {
        // find the name of the component
        // a.b => a
        // a.b.c => a.b
        // a[1].b => a[1]
        // a[1].b[1].c => a[1].b[1]

        let mut last_pin = signal.sels.len();
        let mut found = false;
        while !found && last_pin > 0 {
            match &*signal.sels[last_pin - 1] {
                SelectorP::Index { .. } => last_pin -= 1,
                SelectorP::Pin { .. } => found = true,
            }
        }

        if found {
            // remove the signal from the list of the pending signals to process
            Ok(Some(self.expand_selectors(
                scope,
                signal,
                Some(last_pin - 1),
            )?))
        } else {
            Ok(None)
        }
    }

    fn expand_full_name(&self, s: &str) -> String {
        if self.current_component.is_empty() {
            s.to_string()
        } else {
            format!("{}.{}", self.current_component, s)
        }
    }

    fn dbg_dump_signals(&self) -> Result<()> {
        for n in 0..self.signals.len() {
            println!("{:?}", self.signals.get_by_id(n).unwrap());
        }
        Ok(())
    }

    fn dbg_print(&mut self, scope: &Scope, expr: &ExpressionP) -> Result<()> {
        let mut processed = false;

        if let ExpressionP::Variable {
            name: var, meta, ..
        } = &expr
        {
            if var.name == "CTX" {
                println!(
                    "CTX => {} {}:{}",
                    self.current_component, self.current_file, meta.start
                );
                return Ok(());
            }
            if var.name == "SCOPE" {
                println!("{:?}",scope);
                return Ok(());
            }
            if var.name == "TRACEON" {
                self.debug = true;
                return Ok(());
            }
            if var.name == "TRACEOFF" {
                self.debug = false;
                return Ok(());
            }
        }
        
        if let ExpressionP::Variable { name: var, .. } = &expr {
            let full_name = self.expand_selectors(scope, var, None)?;
            let scope_var = scope.get(&full_name).ok_or_else(|| Error::NotFound(full_name.to_string()))?;
            if let ScopeValue::Component { pending_inputs, .. } = &*scope_var {
                let pending_inputs_str = pending_inputs
                    .iter()
                    .map(|signal| {
                        format!(
                            "{:?}",
                            self.signals.get_by_id(*signal).unwrap().full_name
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");

                println!(
                    "{} ⇨ pending_inputs {{{}}} ",
                    &full_name, pending_inputs_str
                );
                processed = true;
            }
        }
        
        if !processed {
            let value = self.eval_expression_p(scope, expr)?;
            print!("{:?} ⇨ ", expr);
            match value {
                ReturnValue::Algebra(value) => print!("{} ", self.signals.format(&value)),
                _ => print!("{:?} ", value),
            }
        }
        Ok(())
    }
}
