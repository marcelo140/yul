extern crate rust;

use std::collections::HashMap;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use itertools::Itertools;

use rust::reader::read_form;
use rust::types::*;
use rust::env::Env;

fn eval_ast(val: MalVal, repl_env: &mut Env) -> Result<MalVal> {
    let mut repl_env = repl_env;

    match val {
        MalVal::Sym(x) => {
            repl_env.get(&x.as_str().to_string())
                .ok_or_else(|| Error::NoSymbolFound(x))
        },

        MalVal::List(vec) => {
            vec.into_iter()
                .map(|x| eval(Ok(x), &mut repl_env))
                .collect::<Result<Vec<MalVal>>>()
                .map(MalVal::List)
        },

        MalVal::HashMap(hm) => {
            let hm = hm.into_iter()
                .map(|(k, v)| (k, eval(Ok(v), &mut repl_env).unwrap()))
                .collect::<HashMap<String, MalVal>>();
                Ok(MalVal::HashMap(hm))
        },

        MalVal::Vector(vec) => {
            vec.into_iter()
                .map(|x| eval(Ok(x), &mut repl_env))
                .collect::<Result<Vec<MalVal>>>()
                .map(MalVal::Vector)
        }

        x => Ok(x),
    }
}

fn read(input: &str) -> Result<MalVal> {
    read_form().parse(input.as_bytes()).map_err(From::from)
}

fn eval(input: Result<MalVal>, env: &mut Env) -> Result<MalVal> {
    let input = input?;

    if !input.is_list() {
        return Ok(eval_ast(input, env)?);
    }
    
    let l = input.cast_to_list()?;

    if l.is_empty() {
        return Ok(MalVal::List(l));
    }

    match l[0] {
        MalVal::Sym(ref sym) if sym == "def!" => {
            let key = l[1].to_string();
            let v = eval(Ok(l[2].clone()), env)?;
            env.set(key, v.clone());
            Ok(v)
        },

        MalVal::Sym(ref sym) if sym == "let*" => {
            let mut env = Env::new(Some(env.clone()));

            let binds = l[1].clone().cast_to_list()?;

            for (bind, expr) in binds.clone().into_iter().tuples() {
                let bind = bind.cast_to_sym()?;
                let v = eval(Ok(expr), &mut env)?;
                env.set(bind, v);
            }
            
            eval(Ok(l[2].clone()), &mut env)
        },

        _ => {
            let evaluated_l = eval_ast(MalVal::List(l), env)?.cast_to_list()?;

            if let MalVal::Fun(fun) = evaluated_l[0] {
                Ok(fun(evaluated_l[1..].to_vec())?)
            } else {
                Err(Error::EvalError)
            }
        },
    }
}

fn print(input: Result<MalVal>) -> String {
    match input {
        Ok(input) => input.to_string(),
        Err(err) => err.to_string(),
    }
}

fn rep(input: &str, env: &mut Env) -> String {
    let mut env = env;
    print(eval(read(input), &mut env))
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

    let mut repl_env = Env::new(None);
    repl_env.set("+".to_string(), MalVal::Fun(add as FnExpr));
    repl_env.set("-".to_string(), MalVal::Fun(sub as FnExpr));
    repl_env.set("*".to_string(), MalVal::Fun(mul as FnExpr));
    repl_env.set("/".to_string(), MalVal::Fun(div as FnExpr));

    loop {
        let line = ed.readline("user> ");

        match line {
            Ok(line) => {
                println!("{}", &rep(&line, &mut repl_env));
                ed.add_history_entry(line);
            },
            Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    ed.save_history(".mal_history").ok();
}
