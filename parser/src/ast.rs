use num_bigint::BigInt;


#[derive(Clone, Debug)]
pub struct Attributes(Vec<String>);

impl Attributes {
    pub fn has_tag(&self, t : &str) -> bool {
        self.0.iter().any(|e| e==t)
    }
    pub fn has_tag_w(&self) -> bool {
        self.has_tag("w")
    }
    pub fn has_tag_test(&self) -> bool {
        self.has_tag("test")        
    }
}


#[derive(Clone, Debug)]
pub struct Meta {
    pub start   : usize,
    pub end     : usize,

    pub attrs   : Attributes,
}

impl Meta {
    pub fn new(start : usize, end : usize, attrs : Option<Vec<String>>) -> Self {
        if let Some(attrs) = attrs {
            Self { start, end, attrs : Attributes(attrs) }
        } else {
            Self { start, end, attrs : Attributes(Vec::new()) }
        }
    }
}

#[derive(Clone)]
pub enum SelectorP {
    Pin {
        meta : Meta,
        name : String,
    },
    Index { 
        meta : Meta,
        pos : Box<ExpressionP>,
    },
}

#[derive(Clone)]
pub struct VariableP {
    pub meta : Meta,
    pub name : String,
    pub sels : Vec<Box<SelectorP>>
}

#[derive(Clone)]
pub enum ExpressionP {
    FunctionCall {
        meta : Meta,
        name : String,
        args : Vec<Box<ExpressionP>>,
    },
    Variable {
        meta : Meta,        
        name : Box<VariableP>,
    },
    Number {
        meta : Meta,                
        value : BigInt
    },
    PrefixOp {
        meta : Meta,                
        op :  Opcode,
        rhe: Box<ExpressionP>,
    },
    InfixOp {
        meta : Meta,                        
        lhe : Box<ExpressionP>,
        op : Opcode,
        rhe : Box<ExpressionP>
    },
    Array {
        meta : Meta,                        
        values : Vec<Box<ExpressionP>>
    },
}

#[derive(Clone)]
pub enum StatementP {
    IfThenElse {
        meta : Meta,                        
        xif : Box<ExpressionP>,
        xthen : Box<StatementP>,
        xelse : Option<Box<StatementP>>
    },
    For {
        meta : Meta,                        
        init : Box<StatementP>,
        cond : Box<ExpressionP>,
        step : Box<StatementP>,
        stmt : Box<StatementP>,
    },
    While {
        meta : Meta,                        
        cond : Box<ExpressionP>,
        stmt : Box<StatementP>
    },
    Return {
        meta : Meta,
        value : Box<ExpressionP>
    },
    Declaration {
        meta : Meta,                        
        xtype : VariableType,
        name : Box<VariableP>,
        init : Option<(Opcode, Box<ExpressionP>)>,
    },
    Substitution {
        meta : Meta,                        
        name : Box<VariableP>,
        op : Opcode,
        value : Box<ExpressionP>
    },
    Block {
        meta : Meta,                        
        stmts : Vec<Box<StatementP>>
    },
    SignalLeft {
        meta : Meta,                        
        name : Box<VariableP>,
        op : Opcode,
        value : Box<ExpressionP>
    },
    SignalRight {
        meta : Meta,                        
        value : Box<ExpressionP>,
        op : Opcode,
        name : Box<VariableP>
    },
    SignalEq {
        meta : Meta,                        
        lhe : Box<ExpressionP>,
        op : Opcode,
        rhe : Box<ExpressionP>
    },
    InternalCall {
        meta : Meta,
        name : String,
        args : Vec<Box<ExpressionP>>,
    },
}

#[derive(Clone)]
pub enum BodyElementP {
    Include {
        meta : Meta,                        
        path : String
    },
    FunctionDef {
        meta : Meta,                        
        name : String,
        args : Vec<String>,
        stmt : Box<StatementP>
    },
    TemplateDef {
        meta : Meta,                        
        name : String,
        args : Vec<String>,
        stmt : Box<StatementP>
    },
    Declaration {
        meta : Meta,                        
        decl : Box<StatementP>
    },
}

#[derive(Debug,Copy, Clone, PartialEq)]
pub enum SignalType {
    Internal,
    PublicInput,
    PrivateInput,
    Output,
}

#[derive(Copy, Clone, PartialEq)]
pub enum VariableType {
    Empty,
    Var,
    Signal(SignalType),
    Component,
}

#[derive(Copy, Clone,PartialEq)]
pub enum Opcode {
    Mul,
    Div,
    Add,
    Sub,
    Pow,
    IntDiv,
    Mod,
    ShiftL,
    ShiftR,
    LesserEq,
    GreaterEq,
    Lesser,
    Greater,
    Eq,
    NotEq,
    BoolOr,
    BoolAnd,
    BoolNot,
    BitOr,
    BitAnd,
    BitXor,
    Assig,
    AssigAdd,
    AssigSub,
    AssigMul,
    AssigDiv,
    AssigMod,
    AssigShiftL,
    AssigShiftR,
    AssigBitAnd,
    AssigBitOr,
    AssigBitXor,
    SignalWireLeft,
    SignalWireRight,
    SignalContrainLeft,
    SignalContrainRight,
    SignalContrainEq,
}

#[cfg(test)]
mod test {
    use super::super::lang;

    fn test_expression(expr: &str, expected: &str) {
        let expr = lang::ExpressionParser::new().parse(expr).unwrap();
        assert_eq!(&format!("{:?}", expr), expected);
    }

    fn test_statement(expr: &str) {
        let parsed = lang::StatementParser::new().parse(expr).unwrap();
        assert_eq!(&format!("{:?}", parsed), expr);
    }

    fn test_bodyelement(expr: &str) {
        let parsed = lang::BodyElementParser::new().parse(expr).unwrap();
        assert_eq!(&format!("{:?}", parsed), expr);
    }

    #[test]
    fn expression_number() {
        test_expression("255", "0xff");
        test_expression("-255", "(- 0xff)");
        test_expression("0xFF", "0xff");
        test_expression("0xff", "0xff");
    }

    #[test]
    fn expression_intpri() {
        test_expression(
            "- 1 | 2 ^ 3 & 4 << 5 + 6 * 7",
            "((- 0x1) | (0x2 ^ (0x3 & (0x4 << (0x5 + (0x6 * 0x7))))))",
        );
    }

    #[test]
    fn expression_intpri_inv() {
        test_expression(
            "(a | b) ^ c & d << e + f * g",
            "((a | b) ^ (c & (d << (e + (f * g)))))",
        );
    }

    #[test]
    fn expression_boolpri() {
        test_expression(
            "a == b && c == d || e == f",
            "(((a == b) && (c == d)) || (e == f))",
        );
    }

    #[test]
    fn expression_boolexp_pri() {
        test_expression(
            "a > b || c < d || e >=f || g<=h || i==j || k !=l",
            "((((((a > b) || (c < d)) || (e >= f)) || (g <= h)) || (i == j)) || (k != l))",
        );
    }

    #[test]
    fn expression_boolexp_pri_inv() {
        test_expression(
            "(a == b && c == d) || e == f",
            "(((a == b) && (c == d)) || (e == f))",
        );
    }

    #[test]
    fn expression_indexed_pinned_variable() {
        test_expression("a", "a");
        test_expression("a[5]", "a[0x5]");
        test_expression("a.b", "a.b");
        test_expression("a[5].b", "a[0x5].b");
        test_expression("a[c[0x1*0x1].d].b", "a[c[(0x1 * 0x1)].d].b");
    }

    #[test]
    fn expression_function() {
        test_expression("f(a*1,b(),c(1*2))", "f((a * 0x1),b(),c((0x1 * 0x2)))");
    }

    #[test]
    fn statement_declaration() {
        test_statement("var a;");
        test_statement("var a = b;");
        test_statement("component a = b;");
        test_statement("signal a;");
        test_statement("signal input a;");
        test_statement("signal private input a;");
        test_statement("signal output a;");
    }

    #[test]
    fn statement_assigment() {
        test_statement("a = b;");
        test_statement("a -= b;");
        test_statement("a *= b;");
        test_statement("a /= b;");
        test_statement("a %= b;");
        test_statement("a >>= b;");
        test_statement("a <<= b;");
        test_statement("a |= b;");
        test_statement("a &= b;");
        test_statement("a[0x1].a = b;");
    }

    #[test]
    fn statement_ifelse() {
        test_statement("if (a) {b = c;}");
        test_statement("if (a) {b = c;} else {b = c;}");
        test_statement("if (a) {b = c;} else if (b) {d = e;}");
        test_statement("if (a) {b = c;} else if (b) {d = e;} else {i = k;}");
    }

    #[test]
    fn statement_while() {
        test_statement("while (a) {b += c;}");
    }

    #[test]
    fn statement_for() {
        test_statement("for (a = u;(a < b);a += d) {b += c;}");
        test_statement("for (var a = u;(a < b);a += d) {b += c;}");
    }

    #[test]
    fn statement_return() {
        test_statement("return a;");
    }

    #[test]
    fn statement_signal() {
        test_statement("a <-- b;");
        test_statement("a --> b;");
        test_statement("a ==> b;");
        test_statement("a <== b;");
        test_statement("a === b;");
    }

    #[test]
    fn statement_block() {
        test_statement("if (a) {b = c; b = c;}");
        test_statement("if (a) {b = c; b = c;} else {a = a; b = a;}");
    }

    #[test]
    fn body_element() {
        test_bodyelement("include \"hola\";");
        test_bodyelement("function f1(a,b,c) {a += b;}");
        test_bodyelement("template f1(a,b,c) {a += b;}");
        test_bodyelement("var a;");
    }
}
