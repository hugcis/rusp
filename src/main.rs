extern crate termion;

mod parser;

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::style;

const PROMPT: &str = "rlisp> ";

fn print_prompt<W: Write>(out: &mut RawTerminal<W>) {
    write!(
        out,
        "{}{}{}",
        termion::color::Fg(termion::color::Blue),
        PROMPT,
        termion::color::Fg(termion::color::Reset),
    )
    .unwrap();
    out.flush().unwrap();
}

fn parse_str(buf_str: &str) -> Result<(parser::Ops, Vec<parser::Expr>), String> {
    parser::lispy(buf_str.as_bytes())
        .map_err(|e: nom::Err<_>| format!("{:#?}", e))
        .and_then(|(_, exp)| Ok(exp))
}

fn parse_expr(exp: &parser::Expr) -> Result<i64, &str> {
    match exp {
        parser::Expr::Int(a) => Ok(*a),
        parser::Expr::Obj(b) => eval_ast(b),
    }
}

fn eval_ast(ast: &(parser::Ops, Vec<parser::Expr>)) -> Result<i64, &str> {
    match ast {
        (parser::Ops::Add, v) => v.iter().try_fold(0, |sum, i| Ok(sum + parse_expr(i)?)),
        (parser::Ops::Mul, v) => v.iter().try_fold(1, |mul, i| Ok(mul * parse_expr(i)?)),
        (parser::Ops::Sub, v) => {
            if (v.len() != 2) & (v.len() != 1) {
                Err("Substraction operates on two inputs.")
            } else if v.len() == 2 {
                Ok(parse_expr(&v[0])? - parse_expr(&v[1])?)
            } else {
                Ok(-parse_expr(&v[0])?)
            }
        }
        (parser::Ops::Div, v) => {
            if v.len() != 2 {
                Err("Division operates on two inputs.")
            } else {
                Ok(parse_expr(&v[0])?
                    .checked_div(parse_expr(&v[1])?)
                    .ok_or("Invalid division by 0")?)
            }
        }
        (parser::Ops::Rem, v) => {
            if v.len() != 2 {
                Err("Remainder operates on two inputs.")
            } else {
                Ok(parse_expr(&v[0])?
                    .checked_rem(parse_expr(&v[1])?)
                    .ok_or("Invalid remainder by 0")?)
            }
        }
    }
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin.lock();
    println!(
        "{}RSLisp Version 0.0.1\r\nPress Ctrl+c to Exit{}\r",
        style::Bold,
        style::Reset,
    );

    let mut buf: Vec<char> = vec![];
    let mut history: Vec<String> = vec![];
    let mut count = 0;
    let mut hist_height = 1;
    print_prompt(&mut stdout);

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Up => {
                if history.len() >= hist_height {
                    let last = &history[history.len() - hist_height];
                    print!("\r{}", termion::clear::CurrentLine);
                    print_prompt(&mut stdout);
                    print!("{}", last);
                    buf = last.chars().collect();
                    count = buf.len();
                    hist_height += 1;
                }
            }
            Key::Char('\n') => {
                print!("\r\n");
                let buf_str = buf.iter().collect::<String>();
                match parse_str(&format!("{}\r\n", buf_str)) {
                    Ok(ast) => {
                        let result = eval_ast(&ast);
                        match result {
                            Ok(result) => print!("{}\r\n", result),
                            Err(e) => eprint!("Eval error: {}\r\n", e),
                        }
                    }
                    Err(e) => eprint!("Invalid syntax\r\n"),
                }
                print_prompt(&mut stdout);
                if count > 0 {
                    history.push(buf_str);
                }
                count = 0;
                buf = vec![];
                hist_height = 1;
            }
            Key::Ctrl('c') => break,
            Key::Char(c) => {
                print!("{}", c);
                count += 1;
                buf.push(c);
            }
            Key::Backspace => {
                buf.pop().and_then(|_| {
                    count -= 1;
                    print!(
                        "{}{}",
                        termion::cursor::Left(1),
                        termion::clear::AfterCursor
                    );
                    Some(())
                });
            }
            _ => {}
        }
        stdout.flush().unwrap();
    }
}
