mod context;

use crate::parser::Expr;
use custom_error::custom_error;
use std::collections::HashMap;

custom_error! {
    pub EvalError
    ArgumentNumber{exp: usize, got: usize} = "Wrong number of arguments, expected {exp}, got {got}",
    InvalidArguments{args: String} = "Invalid arguments for function: {args}",
    VoidFunction{name: String} = "Function `{name}` not found",
    VoidVariable{name: String} = "Variable `{name}` not found",
    ShouldBeNum = "Argument should be number",
    InvalidVarName = "Invalid variable name",
    Unimplemented{name: String} = "Built-in `{name}` not implemented",
    InvalidFunction{function: String} = "Invalid function `{}`",
    IntOverflow = "Integer overflow",
    DivBy0 = "Division by 0",
    InvalidSyntax = "Invalid syntax",
    WrongTypeArgumentList = "Wrong type argument, expected list"
}

type Result<T> = std::result::Result<T, EvalError>;

#[derive(Default, Debug)]
pub struct Context {
    vars: HashMap<String, Expr>,
    funcs: HashMap<String, Function>,
}

#[derive(Clone, Debug)]
struct Function {
    args: Vec<String>,
    body: Expr,
}
