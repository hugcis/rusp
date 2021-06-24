extern crate nom;

use nom::branch::alt;
use nom::character::complete::digit1;
use nom::combinator::{map, map_res};
// use nom::number::complete::double;
use nom::IResult;
use nom::{
    alt, delimited, many0, many1, named, preceded, separated_list0, separated_pair, tag, terminated,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    // Double(f64),
    Obj((Ops, Vec<Expr>)),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ops {
    Sub,
    Mul,
    Div,
    Add,
    Rem,
}

fn expression(input: &[u8]) -> IResult<&[u8], Expr> {
    use Expr::*;
    alt((
        map_res(digit1, |digit_str: &[u8]| {
            String::from_utf8_lossy(digit_str).parse::<i64>().map(Int)
        }),
        // map(double, Double),
        map(lispy, Obj),
    ))(input)
}

named!(
    operator<Ops>,
    alt!(
        tag!("add") => { |_| Ops::Add }
            | tag!("sub") => { |_| Ops::Sub }
            | tag!("mul") => { |_| Ops::Mul }
            | tag!("div") => { |_| Ops::Div }
            | tag!("+") => { |_| Ops::Add }
            | tag!("*") => { |_| Ops::Mul }
            | tag!("-") => { |_| Ops::Sub }
            | tag!("/") => { |_| Ops::Div }
            | tag!("%") => { |_| Ops::Rem }
    )
);

named!(
    pub lispy<(Ops, Vec<Expr>)>,
    delimited!(
        preceded!(tag!("("), many0!(tag!(" "))),
        separated_pair!(operator, many0!(tag!(" ")),
                        separated_list0!(many1!(tag!(" ")), expression)),
        terminated!(many0!(tag!(" ")), tag!(")"))
    )
);

#[test]
fn should_parse_polish() {
    use Expr::*;
    let inp = "(+ 3 1232 312 (- 2 2))\n".as_bytes();
    assert_eq!(
        lispy(inp),
        Ok((
            &[10][..],
            (
                Ops::Add,
                [
                    Int(3),
                    Int(1232),
                    Int(312),
                    Obj((Ops::Sub, [Int(2), Int(2)].to_vec()))
                ]
                .to_vec()
            )
        ))
    );
}
