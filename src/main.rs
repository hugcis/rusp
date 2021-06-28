extern crate termion;

mod evaluator;
pub mod parser;

use crate::evaluator::Context;
use crate::parser::parse_str;
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

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin.lock();
    println!(
        "{}RSLisp Version 0.0.1\r\nPress Ctrl+c to Exit{}\r",
        style::Bold,
        style::Reset,
    );

    let mut ctx = Context::default();

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
                match parse_str(&buf_str) {
                    Ok(ast) => {
                        let result = ctx.eval_ast(&ast);
                        match result {
                            Ok(result) => print!("{:?}\r\n", result),
                            Err(e) => eprint!("Eval error: {}\r\n", e),
                        }
                    }
                    Err(_e) => eprint!("Invalid syntax\r\n"),
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
                if buf.pop().is_some() {
                    count -= 1;
                    print!(
                        "{}{}",
                        termion::cursor::Left(1),
                        termion::clear::AfterCursor
                    );
                }
            }
            _ => {}
        }
        stdout.flush().unwrap();
    }
}