extern crate nom;

mod strparser;
mod types;

use custom_error::custom_error;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{alpha1, alphanumeric1, char, one_of};
use nom::combinator::{map, map_res, not, recognize};
use nom::multi::{many0, many1, separated_list0};
use nom::number::complete::double;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;
use nom::{alt, named, tag};

pub use types::{Atom, Bool, Expr, Num, Ops};

custom_error! {
    pub SyntaxError
    TrailingGarbage = "Trailing garbage following expression",
    ParsingError{message: String} = "Invalid syntax: {message}"
}

fn decimal(input: &str) -> IResult<&str, i64> {
    map_res(
        recognize(terminated(
            many1(terminated(one_of("0123456789"), many0(char('_')))),
            not(alt((tag("."), tag_no_case("e")))),
        )),
        |out: &str| str::replace(out, "_", "").parse::<i64>(),
    )(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

named!(
    operator<&str, Ops>,
    alt!(
        tag!("add") => { |_| Ops::Add }
            | tag!("sub") => { |_| Ops::Sub }
            | tag!("mul") => { |_| Ops::Mul }
            | tag!("div") => { |_| Ops::Div }
            | tag!("defun") => { |_| Ops::Defun }
            | tag!("+") => { |_| Ops::Add }
            | tag!("*") => { |_| Ops::Mul }
            | tag!("-") => { |_| Ops::Sub }
            | tag!("/") => { |_| Ops::Div }
            | tag!("%") => { |_| Ops::Rem }
            | tag!("nth") => { |_| Ops::Nth }
            | tag!("list") => { |_| Ops::List }
            | tag!("eval") => { |_| Ops::Eval }
            | tag!("car") => { |_| Ops::Car }
            // | tag!("map") => { |_| Ops::Map }
    )
);

fn sexpr(input: &str) -> IResult<&str, Vec<Expr>> {
    delimited(
        preceded(tag("("), many0(tag(" "))),
        separated_list0(many1(tag(" ")), expression),
        terminated(many0(tag(" ")), tag(")")),
    )(input)
}

pub fn expression(input: &str) -> IResult<&str, Expr> {
    use Expr::*;
    alt((
        map(operator, |op: Ops| Atomic(Atom::Op(op))),
        map(strparser::parse_string, |x: String| Atomic(Atom::Quoted(x))),
        map(identifier, |id: &str| Atomic(Atom::Name(id.to_string()))),
        map(decimal, |digit: i64| Atomic(Atom::Number(Num::Int(digit)))),
        map(double, |digit: f64| {
            Atomic(Atom::Number(Num::Double(digit)))
        }),
        map(preceded(tag("'"), sexpr), Qexpr),
        map(sexpr, List),
    ))(input)
}

pub fn parse_str(buf_str: &str) -> Result<Expr, SyntaxError> {
    expression(buf_str)
        .map_err(|e: nom::Err<_>| SyntaxError::ParsingError {
            message: e.to_string(),
        })
        .map(|(r, exp)| {
            if r.is_empty() {
                Ok(exp)
            } else {
                Err(SyntaxError::TrailingGarbage)
            }
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_numbers_polish() {
        use Atom::{Number, Op};
        use Expr::Atomic;
        use Num::{Double, Int};
        use Ops::{Add, Div, Sub};
        let inp = "(+ 3 12 2 (- 2. (/ 4 5E-3)))";
        let res = sexpr(inp).expect("Parsing error");
        assert_eq!(
            res.1,
            [
                Atomic(Op(Add)),
                Atomic(Number(Int(3))),
                Atomic(Number(Int(12))),
                Atomic(Number(Int(2))),
                Expr::List(
                    [
                        Atomic(Op(Sub)),
                        Atomic(Number(Double(2.0))),
                        Expr::List(
                            [
                                Atomic(Op(Div)),
                                Atomic(Number(Int(4))),
                                Atomic(Number(Double(0.005)))
                            ]
                            .to_vec()
                        )
                    ]
                    .to_vec()
                )
            ]
        );
    }

    #[test]
    fn should_parse_names() {
        use Atom::*;
        use Expr::*;
        use Num::*;
        let inp = "(lexp 3  2)";
        let res = sexpr(inp).expect("Parsing error");
        assert_eq!(
            res.1,
            [
                Atomic(Name("lexp".to_string())),
                Atomic(Number(Int(3))),
                Atomic(Number(Int(2)))
            ]
        )
    }

    #[test]
    fn should_parse_quoted_strings() {
        use Atom::{Name, Number, Quoted};
        use Expr::Atomic;
        use Num::Int;
        let inp = "(lexp 3  2 \"123qweQWE,./][]\")";
        let res = sexpr(inp).expect("Parsing error");
        assert_eq!(
            res.1,
            [
                Atomic(Name("lexp".to_string())),
                Atomic(Number(Int(3))),
                Atomic(Number(Int(2))),
                Atomic(Quoted("123qweQWE,./][]".to_string()))
            ]
        )
    }

    #[test]
    fn should_parse_qexpr() {
        use Atom::{Name, Number, Op, Quoted};
        use Expr::{Atomic, Qexpr};
        use Num::Int;
        use Ops;
        let inp = "(lexp 3  2 '(3 (+ 2 5)  \"ok\" )  )";
        let res = expression(inp).expect("Parsing error");
        assert_eq!(
            res.1,
            Expr::List(
                [
                    Atomic(Name("lexp".to_string())),
                    Atomic(Number(Int(3))),
                    Atomic(Number(Int(2))),
                    Qexpr(
                        [
                            Atomic(Number(Int(3))),
                            Expr::List(
                                [
                                    Atomic(Op(Ops::Add)),
                                    Atomic(Number(Int(2))),
                                    Atomic(Number(Int(5))),
                                ]
                                .to_vec()
                            ),
                            Atomic(Quoted("ok".to_string()))
                        ]
                        .to_vec()
                    )
                ]
                .to_vec()
            )
        )
    }

    #[test]
    fn should_parse_singles() {
        use Atom::{Name, Number, Quoted};
        use Expr::Atomic;
        use Num::Int;
        let inp_int = "32";
        let res_int = expression(inp_int).expect("Parsing error");
        assert_eq!(res_int.1, Atomic(Number(Int(32))));
        let inp_name = "test";
        let res_name = expression(inp_name).expect("Parsing error");
        assert_eq!(res_name.1, Atomic(Name("test".to_string())));
        let inp_string = "\"qwe123;'[],/\\u{1F602}\"";
        let res_string = expression(inp_string).expect("Parsing error");
        assert_eq!(res_string.1, Atomic(Quoted("qwe123;'[],/😂".to_string())));
    }
}
