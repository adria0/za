use super::ast::*;
use std::fmt::{Debug, Error, Formatter};

impl Debug for BodyElementP {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::BodyElementP::*;

        match self {
            Include { path, .. } => write!(fmt, "include \"{}\";", path),
            FunctionDef {
                name, args, stmt, ..
            } => write!(fmt, "function {}({}) {:?}", name, args.join(","), stmt),
            TemplateDef {
                name, args, stmt, ..
            } => write!(fmt, "template {}({}) {:?}", name, args.join(","), stmt),
            Declaration { decl, .. } => write!(fmt, "{:?}", decl),
        }
    }
}

impl Debug for VariableType {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::SignalType::*;
        use self::VariableType::*;
        match self {
            Empty => Ok(()),
            Var => write!(fmt, "var"),
            Signal(Internal) => write!(fmt, "signal"),
            Signal(PublicInput) => write!(fmt, "signal input"),
            Signal(PrivateInput) => write!(fmt, "signal private input"),
            Signal(Output) => write!(fmt, "signal output"),
            Component => write!(fmt, "component"),
        }
    }
}

impl Debug for StatementP {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::StatementP::*;

        let for_item = |stp: &StatementP| match stp {
            Declaration {
                xtype,
                name,
                init: Some((op, value)),
                ..
            } => format!("{:?} {:?} {:?} {:?}", xtype, name, op, value),
            Declaration {
                xtype,
                name,
                init: None,
                ..
            } => format!("{:?} {:?}", xtype, name),
            Substitution {
                name, op, value, ..
            } => format!("{:?} {:?} {:?}", name, op, value),
            _ => unreachable!(),
        };

        let sl = |l: &Vec<Box<StatementP>>| {
            l.into_iter()
                .map(|arg| format!("{:?}", arg))
                .collect::<Vec<String>>()
                .join(" ")
        };

        let comma_concat = |l: &Vec<Box<ExpressionP>>| {
            l.into_iter()
                .map(|arg| format!("{:?}", arg))
                .collect::<Vec<String>>()
                .join(",")
        };

        match self {
            Block { stmts, .. } => write!(fmt, "{{{}}}", sl(stmts)),
            IfThenElse {
                xif,
                xthen,
                xelse: Some(xelse),
                ..
            } => write!(fmt, "if ({:?}) {:?} else {:?}", xif, xthen, xelse),
            IfThenElse {
                xif,
                xthen,
                xelse: None,
                ..
            } => write!(fmt, "if ({:?}) {:?}", xif, xthen),
            For {
                init,
                cond,
                step,
                stmt,
                ..
            } => write!(
                fmt,
                "for ({};{:?};{}) {:?}",
                for_item(init),
                cond,
                for_item(step),
                stmt
            ),
            While { cond, stmt, .. } => write!(fmt, "while ({:?}) {:?}", cond, stmt),
            Return { value, .. } => write!(fmt, "return {:?};", value),
            Declaration {
                xtype,
                name,
                init: Some((op, value)),
                ..
            } => write!(fmt, "{:?} {:?} {:?} {:?};", xtype, name, op, value),
            Declaration {
                xtype,
                name,
                init: None,
                ..
            } => write!(fmt, "{:?} {:?};", xtype, name),
            Substitution {
                name, op, value, ..
            } => write!(fmt, "{:?} {:?} {:?};", name, op, value),
            SignalLeft {
                name, op, value, ..
            } => write!(fmt, "{:?} {:?} {:?};", name, op, value),
            SignalRight {
                value, op, name, ..
            } => write!(fmt, "{:?} {:?} {:?};", value, op, name),
            SignalEq { lhe, op, rhe, .. } => write!(fmt, "{:?} {:?} {:?};", lhe, op, rhe),
            InternalCall { name, args, .. } => write!(fmt, "{}!({});", name, comma_concat(args)),
        }
    }
}

impl Debug for SelectorP {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::SelectorP::*;
        match self {
            Pin { name, .. } => write!(fmt, ".{}", name),
            Index { pos, .. } => write!(fmt, "[{:?}]", pos),
        }
    }
}

impl Debug for VariableP {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let concat = |l: &Vec<Box<SelectorP>>| {
            l.into_iter()
                .map(|arg| format!("{:?}", arg))
                .collect::<Vec<String>>()
                .join("")
        };
        write!(fmt, "{}{}", &self.name, concat(&self.sels))
    }
}

impl Debug for ExpressionP {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::ExpressionP::*;

        let comma_concat = |l: &Vec<Box<ExpressionP>>| {
            l.into_iter()
                .map(|arg| format!("{:?}", arg))
                .collect::<Vec<String>>()
                .join(",")
        };

        match self {
            Variable { name, .. } => write!(fmt, "{:?}", name),
            Number { value, .. } => write!(fmt, "{}", value.to_string()),
            PrefixOp { op, rhe, .. } => write!(fmt, "({:?} {:?})", op, rhe),
            InfixOp { lhe, op, rhe, .. } => write!(fmt, "({:?} {:?} {:?})", lhe, op, rhe),
            Array { values, .. } => write!(fmt, "[{}]", comma_concat(values)),
            FunctionCall { name, args, .. } => write!(fmt, "{}({})", name, comma_concat(args)),
        }
    }
}

impl Debug for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Opcode::*;
        match *self {
            Mul => write!(fmt, "*"),
            Div => write!(fmt, "/"),
            Add => write!(fmt, "+"),
            Sub => write!(fmt, "-"),
            Pow => write!(fmt, "**"),
            IntDiv => write!(fmt, "\\"),
            Mod => write!(fmt, "%"),
            ShiftL => write!(fmt, "<<"),
            ShiftR => write!(fmt, ">>"),
            LesserEq => write!(fmt, "<="),
            GreaterEq => write!(fmt, ">="),
            Lesser => write!(fmt, "<"),
            Greater => write!(fmt, ">"),
            Eq => write!(fmt, "=="),
            NotEq => write!(fmt, "!="),
            BoolOr => write!(fmt, "||"),
            BoolAnd => write!(fmt, "&&"),
            BitOr => write!(fmt, "|"),
            BitAnd => write!(fmt, "&"),
            BitXor => write!(fmt, "^"),
            BoolNot => write!(fmt, "!"),
            Assig => write!(fmt, "="),
            AssigAdd => write!(fmt, "+="),
            AssigSub => write!(fmt, "-="),
            AssigMul => write!(fmt, "*="),
            AssigDiv => write!(fmt, "/="),
            AssigMod => write!(fmt, "%="),
            AssigShiftL => write!(fmt, "<<="),
            AssigShiftR => write!(fmt, ">>="),
            AssigBitAnd => write!(fmt, "&="),
            AssigBitOr => write!(fmt, "|="),
            AssigBitXor => write!(fmt, "^="),
            SignalWireLeft => write!(fmt, "<--"),
            SignalWireRight => write!(fmt, "-->"),
            SignalContrainLeft => write!(fmt, "<=="),
            SignalContrainRight => write!(fmt, "==>"),
            SignalContrainEq => write!(fmt, "==="),
        }
    }
}
