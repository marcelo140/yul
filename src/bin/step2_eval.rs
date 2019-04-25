extern crate rust;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use rust::reader::read_form;
use rust::types::*;

use std::collections::HashMap;

fn eval_ast(val: MalVal, repl_env: &HashMap<&str, FnExpr>) -> Result<MalVal> {
    match val {
        MalVal::Sym(x) => {
            repl_env.get(x.as_str())
                .map(|f| MalVal::Fun(*f))
                .ok_or(Error::NoSymbolFound)
        },

        MalVal::List(vec) => {
            vec.into_iter()
                .map(|x| eval(Ok(x), repl_env))
                .collect::<Result<Vec<MalVal>>>()
                .map(MalVal::List)
        },

        MalVal::HashMap(hm) => {
            let hm = hm.into_iter()
                .map(|(k, v)| (k, eval(Ok(v), repl_env).unwrap()))
                .collect::<HashMap<String, MalVal>>();
                Ok(MalVal::HashMap(hm))
        },

        MalVal::Vector(vec) => {
            vec.into_iter()
                .map(|x| eval(Ok(x), repl_env))
                .collect::<Result<Vec<MalVal>>>()
                .map(MalVal::Vector)
        }

        x => Ok(x),
    }
}

fn read(input: &str) -> Result<MalVal> {
    read_form().parse(input.as_bytes()).map_err(From::from)
}

fn eval(input: Result<MalVal>, repl_env: &HashMap<&str, FnExpr>) -> Result<MalVal> {
    let input = input?;

    if !input.is_list() {
        return Ok(eval_ast(input, repl_env)?);
    }
    
    let l = input.cast_to_list()?;

    if l.is_empty() {
        return Ok(MalVal::List(l));
    }

    let evaluated_l = eval_ast(MalVal::List(l), repl_env)?.cast_to_list()?;

    if let MalVal::Fun(fun) = evaluated_l[0] {
        Ok(fun(evaluated_l[1..].to_vec())?)
    } else {
        Err(Error::EvalError)
    }
}

fn print(input: Result<MalVal>) -> String {
    match input {
        Ok(input) => input.to_string(),
        Err(err) => err.to_string(),
    }
}

fn rep(input: &str, repl_env: &HashMap<&str, FnExpr>) -> String {
    print(eval(read(input), repl_env))
}

fn add(args: Vec<MalVal>) -> Result<MalVal> {
    let x = args.iter()
        .flat_map(MalVal::cast_to_int)
        .sum();

    Ok(MalVal::Int(x))
}

fn sub(args: Vec<MalVal>) -> Result<MalVal> {
    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x -= y.cast_to_int()?;
    }

    Ok(MalVal::Int(x))
}

fn mul(args: Vec<MalVal>) -> Result<MalVal> {
    let x = args.iter()
        .flat_map(MalVal::cast_to_int)
        .product();

    Ok(MalVal::Int(x))
}

fn div(args: Vec<MalVal>) -> Result<MalVal> {
    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x /= y.cast_to_int()?;
    }

    Ok(MalVal::Int(x))
}

fn main() {
    let mut ed = Editor::<()>::new();
    ed.load_history(".mal_history").ok();

    let mut repl_env = HashMap::new();
    repl_env.insert("+", add as FnExpr);
    repl_env.insert("-", sub as FnExpr);
    repl_env.insert("*", mul as FnExpr);
    repl_env.insert("/", div as FnExpr);

    loop {
        let line = ed.readline("user> ");

        match line {
            Ok(line) => {
                println!("{}", &rep(&line, &repl_env));
                ed.add_history_entry(line);
            },
            Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    ed.save_history(".mal_history").ok();
}
