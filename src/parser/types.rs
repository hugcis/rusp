use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Atomic(Atom),
    Qexpr(Vec<Expr>),
    List(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Name(String),
    Quoted(String),
    Op(Ops),
    Number(Num),
    Boolean(Bool),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bool {
    True,
    Nil,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Num {
    Double(f64),
    Int(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ops {
    Sub,
    Mul,
    Div,
    Add,
    Rem,
    Defun,
    Nth,
    List,
    Eval,
    Car,
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Num::Double(d) => write!(f, "{}", d),
            Num::Int(d) => write!(f, "{}", d),
        }
    }
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bool::True => write!(f, "t"),
            Bool::Nil => write!(f, "nil"),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Name(n) => write!(f, "{}", n),
            Atom::Quoted(n) => write!(f, "\"{}\"", n),
            Atom::Op(op) => write!(f, "{:?}", op),
            Atom::Number(num) => write!(f, "{}", num),
            Atom::Boolean(bl) => write!(f, "{}", bl),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Atomic(atom) => write!(f, "{}", atom),
            Expr::Qexpr(exprs) => write!(
                f,
                "'({})",
                exprs
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Expr::List(exprs) => write!(
                f,
                "({})",
                exprs
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}
