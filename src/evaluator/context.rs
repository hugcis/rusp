use super::{Context, EvalError, Function, Result};
use crate::parser::Expr::Atomic;
use crate::parser::Expr::Qexpr;
use crate::parser::{Atom, Bool, Expr, Num, Ops};
use std::collections::HashMap;

impl Context {
    fn get_funcs(&self) -> &HashMap<String, Function> {
        &self.funcs
    }

    fn add_var(&mut self, name: &str, var: Expr) {
        self.vars.insert(name.to_owned(), var);
    }

    fn get_function_from_name(&self, name: &str, args: &[Expr]) -> Result<Function> {
        let fun_res = self.get_funcs().get(name);
        match fun_res {
            Some(fun) => {
                if args.len() != fun.args.len() {
                    Err(EvalError::ArgumentNumber {
                        exp: fun.args.len(),
                        got: args.len(),
                    })
                } else {
                    Ok(fun.clone())
                }
            }
            None => Err(EvalError::VoidFunction {
                name: name.to_string(),
            }),
        }
    }

    fn apply(&mut self, function: &Expr, args: Vec<Expr>) -> Result<Expr> {
        match function {
            Atomic(Atom::Name(name)) => {
                let fun = self.get_function_from_name(name, &args)?;
                // Add the local variables to the context
                for (i, arg_name) in fun.args.iter().enumerate() {
                    let arg = args[i].clone();
                    self.add_var(arg_name, arg);
                }
                println!("{:?} {:?}", fun, self.vars);
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
                Ops::Nth => self.nth(args),
                Ops::List => self.list(args),
                Ops::Eval => self.eval_builtin(args),
                Ops::Car => self.car(args),
                Ops::Map => self.map(args),
                _ => Err(EvalError::Unimplemented {
                    name: format!("{:?}", op),
                }),
            },
            _ => Err(EvalError::InvalidFunction {
                function: function.to_string(),
            }),
        }
    }

    pub fn eval_ast(&mut self, ast: &Expr) -> Result<Expr> {
        match ast {
            Atomic(Atom::Name(name)) => match self.vars.get(name.as_str()) {
                Some(var) => {
                    let var = var.clone();
                    self.eval_ast(&var)
                }
                None => Err(EvalError::VoidVariable {
                    name: name.to_string(),
                }),
            },
            Atomic(Atom::Op(_op)) => Err(EvalError::InvalidVarName),
            Atomic(atom) => Ok(Expr::Atomic(atom.clone())),
            Expr::List(sexp_list) => {
                if sexp_list.is_empty() {
                    Ok(Atomic(Atom::Boolean(Bool::Nil)))
                } else {
                    print!("in:{:?}\r\n", sexp_list);
                    let res = self.apply(&sexp_list[0], sexp_list[1..].to_vec());
                    print!("out:{:?}\r\n", res);
                    res
                }
            }
            Qexpr(sexp_list) => Ok(Expr::List(sexp_list.to_vec())),
        }
    }

    fn nth(&mut self, args: Vec<Expr>) -> Result<Expr> {
        match args.as_slice() {
            [Atomic(Atom::Number(Num::Int(idx))), Qexpr(vec)] => {
                let answer = vec[*idx as usize].clone();
                Ok(answer)
            }
            _ => Err(EvalError::InvalidArguments {
                args: args
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
            }),
        }
    }

    fn map(&mut self, args: Vec<Expr>) -> Result<Expr> {
        match args.as_slice() {
            [func, Qexpr(vec)] => Ok(Expr::List(
                vec.iter()
                    .map(|arg| match arg {
                        Qexpr(arg_vec) => {println!("{} {:?}", func, arg_vec);self.apply(func, arg_vec.to_vec())},
                        Atomic(arg) => self.apply(func, vec![Atomic(arg.clone())]),
                        Expr::List(arg) => self.apply(func, vec![Expr::List(arg.to_vec())]),
                    })
                    .collect::<Result<Vec<Expr>>>()?,
            )),
            _ => Err(EvalError::InvalidArguments {
                args: args
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
            }),
        }
    }

    fn defun(&mut self, args: Vec<Expr>) -> Result<Expr> {
        if args.len() != 3 {
            Err(EvalError::ArgumentNumber {
                exp: 3,
                got: args.len(),
            })
        } else {
            match args.as_slice() {
                [Atomic(Atom::Name(name)), Expr::List(fn_args), Expr::List(fn_body)] => {
                    let function_args = fn_args
                        .iter()
                        .map(|x| {
                            if let Atomic(Atom::Name(s)) = x {
                                Ok(s.clone())
                            } else {
                                Err(EvalError::InvalidArguments {
                                    args: fn_args
                                        .iter()
                                        .map(|x| format!("{}", x))
                                        .collect::<Vec<String>>()
                                        .join(" "),
                                })
                            }
                        })
                        .collect::<Result<Vec<String>>>()?;
                    let function = Function {
                        args: function_args,
                        body: Expr::List(fn_body.to_vec()),
                    };
                    self.funcs.insert(name.to_string(), function);
                    Ok(Atomic(Atom::Name(name.to_string())))
                }
                _ => Err(EvalError::InvalidSyntax),
            }
        }
    }

    fn list(&mut self, args: Vec<Expr>) -> Result<Expr> {
        Ok(Qexpr(
            args.iter()
                .map(|x| self.eval_ast(x))
                .collect::<Result<Vec<Expr>>>()?,
        ))
    }

    fn eval_builtin(&mut self, args: Vec<Expr>) -> Result<Expr> {
        if args.len() != 1 {
            Err(EvalError::ArgumentNumber {
                exp: 1,
                got: args.len(),
            })
        } else {
            print!("eval-arg:{:?}\r\n", &args[0]);
            let interm = &self.eval_ast(&args[0])?;
            Ok(self.eval_ast(interm)?)
        }
    }

    fn args_to_numbers(&mut self, args: Vec<Expr>) -> Result<Vec<Num>> {
        args.into_iter()
            .map(|x| self.eval_ast(&x))
            .collect::<Result<Vec<Expr>>>()?
            .into_iter()
            .map(|x| match x {
                Atomic(Atom::Number(n)) => Ok(n),
                _ => Err(EvalError::ShouldBeNum),
            })
            .collect::<Result<Vec<Num>>>()
    }

    fn add(&mut self, args: Vec<Expr>) -> Result<Expr> {
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
                        .ok_or(EvalError::IntOverflow)?,
                )
            },
        )))
    }

    fn mul(&mut self, args: Vec<Expr>) -> Result<Expr> {
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
                        .ok_or(EvalError::IntOverflow)?,
                )
            },
        )))
    }

    fn sub(&mut self, args: Vec<Expr>) -> Result<Expr> {
        if args.len() != 2 {
            return Err(EvalError::ArgumentNumber {
                exp: 2,
                got: args.len(),
            });
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
                    neg[0].checked_sub(neg[1]).ok_or(EvalError::IntOverflow)?
                })
            },
        )))
    }

    fn div(&mut self, args: Vec<Expr>) -> Result<Expr> {
        if args.len() != 2 {
            return Err(EvalError::ArgumentNumber {
                exp: 2,
                got: args.len(),
            });
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
                    neg[0].checked_div(neg[1]).ok_or(EvalError::DivBy0)?
                })
            },
        )))
    }

    /// Return the first element of a list or nil if empty
    fn car(&mut self, args: Vec<Expr>) -> Result<Expr> {
        if args.len() != 1 {
            Err(EvalError::ArgumentNumber {
                exp: 1,
                got: args.len(),
            })
        } else {
            match self.eval_ast(&args[0])? {
                Expr::List(c) => {
                    if c.is_empty() {
                        Ok(Expr::Atomic(Atom::Boolean(Bool::Nil)))
                    } else {
                        Ok(c[0].clone())
                    }
                }
                Qexpr(_) | Atomic(_) => Err(EvalError::WrongTypeArgumentList),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::evaluator::context::Function;
    use crate::parser::{Atom, Expr, Expr::Atomic, Num, Ops};

    #[test]
    fn should_eval_atomic() {
        let mut ctx = Context::default();
        let ast = Expr::Atomic(Atom::Number(Num::Int(3)));
        let result = ctx.eval_ast(&ast);
        assert_eq!(result.unwrap(), Expr::Atomic(Atom::Number(Num::Int(3))));
    }

    #[test]
    fn should_eval_polish() {
        let mut ctx = Context::default();
        let ast = Expr::List(
            [
                Atomic(Atom::Op(Ops::Add)),
                Atomic(Atom::Number(Num::Int(3))),
                Atomic(Atom::Number(Num::Int(3))),
                Expr::List(
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
        assert_eq!(result.unwrap(), Expr::Atomic(Atom::Number(Num::Int(20))));
    }

    #[test]
    fn should_eval_polish_with_vars() {
        let mut ctx = Context::default();
        ctx.add_var("x", Atomic(Atom::Number(Num::Int(12))));
        let ast = Expr::List(
            [
                Atomic(Atom::Op(Ops::Add)),
                Atomic(Atom::Number(Num::Int(3))),
                Atomic(Atom::Name("x".to_string())),
                Expr::List(
                    [
                        Atomic(Atom::Op(Ops::Mul)),
                        Atomic(Atom::Number(Num::Int(9))),
                        Atomic(Atom::Name("x".to_string())),
                    ]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );
        let result = ctx.eval_ast(&ast);
        assert_eq!(result.unwrap(), Atomic(Atom::Number(Num::Int(123))));
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
                Expr::List(
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
        assert_eq!(result.unwrap(), Expr::Atomic(Atom::Number(Num::Int(196))));
    }

    #[test]
    fn should_define_function() {
        let mut ctx = Context::default();
        let ast = Expr::List(
            [
                Atomic(Atom::Op(Ops::Defun)),
                Atomic(Atom::Name("square".to_string())),
                Expr::List([Atomic(Atom::Name("x".to_string()))].to_vec()),
                Expr::List(
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
        assert_eq!(
            result.unwrap(),
            Expr::Atomic(Atom::Name("square".to_string()))
        );
    }

    #[test]
    fn should_define_function_from_car() {
        let mut ctx = Context::default();
        let ast = Expr::List(
            [
                Atomic(Atom::Op(Ops::Defun)),
                Atomic(Atom::Name("first".to_string())),
                Expr::List([Atomic(Atom::Name("x".to_string()))].to_vec()),
                Expr::List(
                    [
                        Atomic(Atom::Op(Ops::Car)),
                        Atomic(Atom::Name("x".to_string())),
                    ]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );
        let result = ctx.eval_ast(&ast);
        assert_eq!(result.unwrap(), Atomic(Atom::Name("first".to_string())));
        let ast = Expr::List(
            [
                Atomic(Atom::Name("first".to_string())),
                Expr::Qexpr(
                    [
                        Atomic(Atom::Number(Num::Int(5))),
                        Atomic(Atom::Number(Num::Int(6))),
                    ]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );
        let result = ctx.eval_ast(&ast);
        assert_eq!(result.unwrap(), Atomic(Atom::Number(Num::Int(5))));
    }
}
