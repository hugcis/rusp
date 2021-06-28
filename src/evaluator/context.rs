use crate::parser::Expr::Atomic;
use crate::parser::Expr::List;
use crate::parser::Expr::Qexpr;
use crate::parser::{Atom, Expr, Num, Ops};
use std::collections::HashMap;

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

impl Context {
    fn get_funcs(&self) -> &HashMap<String, Function> {
        &self.funcs
    }

    fn get_function_from_name(&self, name: &str, args: &[Expr]) -> Result<Function, &'static str> {
        let fun_res = self.get_funcs().get(name);
        match fun_res {
            Some(fun) => {
                if args.len() != fun.args.len() {
                    Err("Wrong number of arguments")
                } else {
                    Ok(fun.clone())
                }
            }
            None => Err("Void function"),
        }
    }

    fn eval(&mut self, function: &Expr, args: Vec<Expr>) -> Result<Expr, &'static str> {
        match function {
            Atomic(Atom::Name(name)) => {
                let fun = self.get_function_from_name(name, &args)?;
                // Add the local variables to the context
                for (i, arg_name) in fun.args.iter().enumerate() {
                    self.vars.insert(arg_name.to_string(), args[i].clone());
                }
                let res = self.eval_ast(&fun.body);
                // Pop local variables from the context
                for arg_name in fun.args.iter() {
                    self.vars.remove(&arg_name.to_string());
                }
                res

            }
            Atomic(Atom::Op(op)) => match op {
                Ops::Add => self.add(args),
                Ops::Sub => self.sub(args),
                Ops::Mul => self.mul(args),
                Ops::Div => self.div(args),
                Ops::Defun => self.defun(args),
                _ => Err("Built-in not implemented"),
            },
            _ => Err("Invalid function"),
        }
    }

    pub fn eval_ast(&mut self, ast: &Expr) -> Result<Expr, &'static str> {
        match ast {
            Atomic(Atom::Name(name)) => match self.vars.get(name.as_str()) {
                Some(var) => {
                    let var = var.clone();
                    self.eval_ast(&var)
                }
                None => Err("Void variable"),
            },
            Atomic(Atom::Op(_op)) => Err("Void variable"),
            Atomic(atom) => Ok(Expr::Atomic(atom.clone())),
            List(sexp_list) => {
                if sexp_list.is_empty() {
                    Err("Empty s-expression")
                } else {
                    self.eval(&sexp_list[0], sexp_list[1..].to_vec())
                }
            }
            Qexpr(sexp_list) => Ok(Qexpr(sexp_list.to_vec())),
        }
    }

    fn args_to_numbers(&mut self, args: Vec<Expr>) -> Result<Vec<Num>, &'static str> {
        args.into_iter()
            .map(|x| self.eval_ast(&x))
            .collect::<Result<Vec<Expr>, _>>()?
            .into_iter()
            .map(|x| match x {
                Atomic(Atom::Number(n)) => Ok(n),
                _ => Err("Invalid argument should be number"),
            })
            .collect::<Result<Vec<Num>, _>>()
    }

    fn add(&mut self, args: Vec<Expr>) -> Result<Expr, &'static str> {
        let num = self.args_to_numbers(args)?;
        // Double type will spread and int will be cast to double
        Ok(Atomic(Atom::Number(
            if num.iter().any(|x| matches!(x, Num::Double(_))) {
                Num::Double(
                    num.iter()
                        .map(|x| match x {
                            Num::Int(i) => *i as f64,
                            Num::Double(d) => *d,
                        })
                        .sum(),
                )
            } else {
                Num::Int(
                    num.iter()
                        .map(|x| match x {
                            Num::Int(i) => *i,
                            _ => 0,
                        })
                        .try_fold(0, |acc: i64, x: i64| acc.checked_add(x))
                        .ok_or("Addition error : integer overflow")?,
                )
            },
        )))
    }

    fn mul(&mut self, args: Vec<Expr>) -> Result<Expr, &'static str> {
        let num = self.args_to_numbers(args)?;
        // Double type will spread and int will be cast to double
        Ok(Atomic(Atom::Number(
            if num.iter().any(|x| matches!(x, Num::Double(_))) {
                Num::Double(
                    num.iter()
                        .map(|x| match x {
                            Num::Int(i) => *i as f64,
                            Num::Double(d) => *d,
                        })
                        .fold(1., |acc, x| acc * x),
                )
            } else {
                Num::Int(
                    num.iter()
                        .map(|x| match x {
                            Num::Int(i) => *i,
                            _ => 0,
                        })
                        .try_fold(1, |acc: i64, x: i64| acc.checked_mul(x))
                        .ok_or("Integer overflow")?,
                )
            },
        )))
    }

    fn sub(&mut self, args: Vec<Expr>) -> Result<Expr, &'static str> {
        if args.len() != 2 {
            return Err("Substraction takes 2 arguments");
        }
        let num = self.args_to_numbers(args)?;
        Ok(Atomic(Atom::Number(
            if num.iter().any(|x| matches!(x, Num::Double(_))) {
                Num::Double({
                    let neg: Vec<f64> = num
                        .iter()
                        .map(|x| match x {
                            Num::Int(i) => *i as f64,
                            Num::Double(d) => *d,
                        })
                        .collect();
                    neg[0] - neg[1]
                })
            } else {
                Num::Int({
                    let neg: Vec<i64> = num
                        .iter()
                        .map(|x| match x {
                            Num::Int(i) => *i,
                            _ => 0,
                        })
                        .collect();
                    neg[0].checked_sub(neg[1]).ok_or("Integer overflow")?
                })
            },
        )))
    }

    fn div(&mut self, args: Vec<Expr>) -> Result<Expr, &'static str> {
        if args.len() != 2 {
            return Err("Substraction takes 2 arguments");
        }
        let num = self.args_to_numbers(args)?;
        Ok(Atomic(Atom::Number(
            if num.iter().any(|x| matches!(x, Num::Double(_))) {
                Num::Double({
                    let neg: Vec<f64> = num
                        .iter()
                        .map(|x| match x {
                            Num::Int(i) => *i as f64,
                            Num::Double(d) => *d,
                        })
                        .collect();
                    neg[0] / neg[1]
                })
            } else {
                Num::Int({
                    let neg: Vec<i64> = num
                        .iter()
                        .map(|x| match x {
                            Num::Int(i) => *i,
                            _ => 0,
                        })
                        .collect();
                    neg[0].checked_div(neg[1]).ok_or("Division by 0")?
                })
            },
        )))
    }

    fn defun(&mut self, args: Vec<Expr>) -> Result<Expr, &'static str> {
        if args.len() != 3 {
            Err("Invalid defun syntax")
        } else {
            match args.as_slice() {
                [Atomic(Atom::Name(name)), List(fn_args), List(fn_body)] => {
                    self.funcs.insert(
                        name.to_string(),
                        Function {
                            args: fn_args
                                .into_iter()
                                .map(|x| {
                                    if let Atomic(Atom::Name(s)) = x {
                                        Ok(s.clone())
                                    } else {
                                        Err("Wrong argument")
                                    }
                                })
                                .collect::<Result<Vec<String>, _>>()?,
                            body: List(fn_body.to_vec()),
                        },
                    );
                    Ok(Atomic(Atom::Name(name.to_string())))
                }
                _ => Err("Invalid defun syntax"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::evaluator::context::Function;
    use crate::parser::{Atom, Expr, Expr::Atomic, Expr::List, Num, Ops};

    #[test]
    fn should_eval_atomic() {
        let mut ctx = Context::default();
        let ast = Expr::Atomic(Atom::Number(Num::Int(3)));
        let result = ctx.eval_ast(&ast);
        assert_eq!(result, Ok(Expr::Atomic(Atom::Number(Num::Int(3)))));
    }

    #[test]
    fn should_eval_polish() {
        let mut ctx = Context::default();
        let ast = Expr::List(
            [
                Atomic(Atom::Op(Ops::Add)),
                Atomic(Atom::Number(Num::Int(3))),
                Atomic(Atom::Number(Num::Int(3))),
                List(
                    [
                        Atomic(Atom::Op(Ops::Add)),
                        Atomic(Atom::Number(Num::Int(5))),
                        Atomic(Atom::Number(Num::Int(9))),
                    ]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );
        let result = ctx.eval_ast(&ast);
        assert_eq!(result, Ok(Expr::Atomic(Atom::Number(Num::Int(20)))));
    }

    #[test]
    fn should_eval_function() {
        let mut ctx = Context::default();
        ctx.funcs.insert(
            "square".to_string(),
            Function {
                args: vec!["x".to_string()],
                body: Expr::List(
                    [
                        Atomic(Atom::Op(Ops::Mul)),
                        Atomic(Atom::Name("x".to_string())),
                        Atomic(Atom::Name("x".to_string())),
                    ]
                    .to_vec(),
                ),
            },
        );
        let ast = Expr::List(
            [
                Atomic(Atom::Name("square".to_string())),
                List(
                    [
                        Atomic(Atom::Op(Ops::Add)),
                        Atomic(Atom::Number(Num::Int(5))),
                        Atomic(Atom::Number(Num::Int(9))),
                    ]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );
        let result = ctx.eval_ast(&ast);
        assert_eq!(result, Ok(Expr::Atomic(Atom::Number(Num::Int(196)))));
    }

    #[test]
    fn should_define_function() {
        let mut ctx = Context::default();
        let ast = Expr::List(
            [
                Atomic(Atom::Name("defun".to_string())),
                Atomic(Atom::Name("square".to_string())),
                List([Atomic(Atom::Name("x".to_string()))].to_vec()),
                List(
                    [
                        Atomic(Atom::Op(Ops::Mul)),
                        Atomic(Atom::Name("x".to_string())),
                        Atomic(Atom::Name("x".to_string())),
                    ]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );
        let result = ctx.eval_ast(&ast);
        assert_eq!(result, Ok(Expr::Atomic(Atom::Name("square".to_string()))));
    }
}
