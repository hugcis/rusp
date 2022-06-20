extern crate custom_error;
extern crate rustyline;

mod evaluator;
pub mod parser;

use crate::evaluator::Context;
use crate::parser::parse_str;
use rustyline::{Cmd, KeyCode, KeyEvent, Modifiers};

use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    expr: Option<String>,

    #[clap(short, long, action)]
    debug: bool,
}

const PROMPT: &str = "rlisp> ";

fn main() {
    let args = Args::parse();
    match args.expr {
        Some(expr) => {
            let mut ctx = Context::new(args.debug);
            for line in expr.lines() {
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
            }
        }
        None => repl(args.debug),
    }
}

fn repl(debug: bool) {
    println!("RSLisp Version 0.0.1\r\nPress Ctrl+c to Exit\r",);

    let mut ctx = Context::new(debug);
    let mut rl = Editor::<()>::new();
    rl.bind_sequence(
        KeyEvent {
            0: KeyCode::Up,
            1: Modifiers::NONE,
        },
        Cmd::HistorySearchForward,
    );
    rl.bind_sequence(
        KeyEvent {
            0: KeyCode::Down,
            1: Modifiers::NONE,
        },
        Cmd::HistorySearchBackward,
    );
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

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
