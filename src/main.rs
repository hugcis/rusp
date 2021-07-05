extern crate custom_error;
extern crate rustyline;

mod evaluator;
pub mod parser;

use crate::evaluator::Context;
use crate::parser::parse_str;
use std::io::{stdin, stdout, Write};

use rustyline::error::ReadlineError;
use rustyline::Editor;

const PROMPT: &str = "rlisp> ";

fn main() {
    println!(
        "{}RSLisp Version 0.0.1\r\nPress Ctrl+c to Exit{}\r",
        style::Bold,
        style::Reset,
    );

    let mut ctx = Context::default();
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut buf: Vec<char> = vec![];
    let mut history: Vec<String> = vec![];
    let mut count = 0;
    let mut hist_height = 1;
    // print_prompt(&mut stdout);

    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                match parse_str(&line) {
                    Ok(ast) => {
                        let result = ctx.eval_ast(&ast);
                        match result {
                            Ok(result) => print!("{}\r\n", result),
                            Err(e) => eprint!("Eval error: {}\r\n", e),
                        }
                    }
                    Err(e) => eprint!("{}\r\n", e),
                };
                rl.add_history_entry(line.as_str());
            }

            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            } // for c in stdin.keys() {

              //     match c.unwrap() {
              //         Key::Up => {
              //             if history.len() >= hist_height {
              //                 let last = &history[history.len() - hist_height];
              //                 print!("\r{}", termion::clear::CurrentLine);
              //                 print_prompt(&mut stdout);
              //                 print!("{}", last);
              //                 buf = last.chars().collect();
              //                 count = buf.len();
              //                 hist_height += 1;
              //             }
              //         }
              //         Key::Char('\n') => {
              //             print!("\r\n");
              //             let buf_str = buf.iter().collect::<String>();
              //             // Debug command to dump the whole context
              //             if buf_str.as_str() == "?ctx" {
              //                 print!("{:?}\r\n", ctx)
              //             };
              //             match parse_str(&buf_str) {
              //                 Ok(ast) => {
              //                     let result = ctx.eval_ast(&ast);
              //                     match result {
              //                         Ok(result) => print!("{}\r\n", result),
              //                         Err(e) => eprint!("Eval error: {}\r\n", e),
              //                     }
              //                 }
              //                 Err(e) => eprint!("{}\r\n", e),
              //             }
              //             print_prompt(&mut stdout);
              //             if count > 0 {
              //                 history.push(buf_str);
              //             }
              //             count = 0;
              //             buf = vec![];
              //             hist_height = 1;
              //         }
              //         Key::Ctrl('c') => break,
              //         Key::Char(c) => {
              //             print!("{}", c);
              //             count += 1;
              //             buf.push(c);
              //         }
              //         Key::Backspace => {
              //             if buf.pop().is_some() {
              //                 count -= 1;
              //                 print!(
              //                     "{}{}",
              //                     termion::cursor::Left(1),
              //                     termion::clear::AfterCursor
              //                 );
              //             }
              //         }
              //         _ => {}
              //     }
              //     stdout.flush().unwrap();
              // }
        }
    }
    rl.save_history("history.txt").unwrap();
}
