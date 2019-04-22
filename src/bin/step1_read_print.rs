extern crate rust;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use pom::Error;

use rust::reader::read_form;
use rust::types::MalVal;

fn read(input: &str) -> Result<MalVal, Error> {
    read_form().parse(input.as_bytes())
}

fn eval(input: Result<MalVal, Error>) -> Result<MalVal, Error> {
    input
}

fn print(input: Result<MalVal, Error>) -> String {
    match input {
        Ok(input) => input.to_string(),
        Err(err) => err.to_string() + "GOT EOF",
    }
}

fn rep(input: &str) -> String {
    print(eval(read(input)))
}

fn main() {
    let mut ed = Editor::<()>::new();
    ed.load_history(".mal_history").ok();

    loop {
        let line = ed.readline("user> ");

        match line {
            Ok(line) => {
                println!("{}", &rep(&line));
                ed.add_history_entry(line);
            },
            Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    ed.save_history(".mal_history").ok();
}
