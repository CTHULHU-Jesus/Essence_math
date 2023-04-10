mod lib;
use std::{ascii::AsciiExt, str::FromStr};
use std::{io,io::Write};
use lib::{CalqAst, Essence};
use anyhow::Result;

fn main() -> Result<()> {
    let mut user_input = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.

                             // while(user_input.to_ascii_lowercase().trim() != "exit") {
    println!("Enter your essence calqulations bellow (ex 4*Fire+3*Air), enter \"exit\" to exit.");
    loop {
        // print!("> ");
// io::stdout().write_all(b"> ")?;
        stdin.read_line(&mut user_input)?;

        if user_input.to_ascii_lowercase().trim() == "exit" {
            break;
        }
        match CalqAst::from_str(&user_input) {
            Ok(ast) => {
                println!("= {}", ast.eval());
            }
            Err(e) => {
                println!("Parseing Error");
            }
        }
        user_input = String::new();
    }

    Ok(())
}
