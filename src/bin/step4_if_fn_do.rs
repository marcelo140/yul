extern crate rust;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use itertools::Itertools;

use rust::reader::read_form;
use rust::types::*;
use rust::env::Env;
use rust::core::*;

fn eval_ast(value: MValue, env: &mut Env) -> Result<MValue> {
    if value.is_symbol() {
        let x = value.cast_to_symbol()?;
        env.get(&x)
           .ok_or_else(|| Error::NoSymbolFound(x))
    } else if value.is_list() {
        value.cast_to_list()?.into_iter()
           .map(|x| eval(x, &mut *env))
           .collect::<Result<_>>()
           .map(MValue::list) 
    } else if value.is_hashmap() {
        value.cast_to_hashmap()?.into_iter()
           .map(|(k, v)| eval(v, &mut *env).map(|v| (k,v)) )
           .collect::<Result<_>>()
           .map(MValue::hashmap)
    } else if value.is_vector() {
        value.cast_to_list()?.into_iter()
           .map(|x| eval(x, &mut *env))
           .collect::<Result<_>>()
           .map(MValue::vector)
    } else {
        Ok(value)
    }
}

fn read(input: &str) -> Result<MValue> {
    read_form().parse(input.as_bytes()).map_err(From::from)
}

fn eval(input: MValue, env: &mut Env) -> Result<MValue> {
    if !input.is_list() {
        return eval_ast(input, env);
    }

    let mut l = input.cast_to_list()?; // unpack

    if l.is_empty() {
        return Ok(MValue::list(l)); // pack
    }

    match *l[0].0 {
        MalVal::Sym(ref sym) if sym == "do" => {
            let v = MValue::list(l.split_off(1));
            eval_ast(v, env)?.cast_to_list()?.pop().ok_or(Error::EvalError)
        },

        MalVal::Sym(ref sym) if sym == "if" => {
            let condition = eval(l[1].clone(), env)?; // MValue clone
            match *condition.0 {
                MalVal::Bool(false) | MalVal::Nil if l.len() >= 4 =>
                    eval(l[3].clone(), env), // MValue clone
                MalVal::Bool(false) | MalVal::Nil =>
                    Ok(MValue::nil()),
                _ =>
                    eval(l[2].clone(), env), // MValue clone
            }
        },

        MalVal::Sym(ref sym) if sym == "fn*" => {
            Ok(MValue::nil())
        },

        MalVal::Sym(ref sym) if sym == "def!" => {
            let key = l[1].cast_to_symbol()?;
            let v = eval(l[2].clone(), env)?; // malval clone
            env.set(key, v.clone()); // malval clone
            Ok(v)
        },

        MalVal::Sym(ref sym) if sym == "let*" => {
            let mut env = Env::new(Some(env));

            let binds = l[1].clone().cast_to_list()?; // malval clone

            for (bind, expr) in binds.clone().into_iter().tuples() {
                let bind = bind.cast_to_symbol()?;
                let v = eval(expr, &mut env)?;

                env.set(bind, v);
            }

            eval(l[2].clone(), &mut env) // malval clone
        },

        _ => {
            let evaluated_list = eval_ast(MValue::list(l), env)?.cast_to_list()?;

            if let MalVal::Fun(fun) = *evaluated_list[0].0 {
                fun(evaluated_list[1..].to_vec())
            } else {
                Err(Error::EvalError)
            }
        },
    }
}

fn print(input: Result<MValue>) -> String {
    match input {
        Ok(mvalue) => mvalue.to_string(),
        Err(error) => error.to_string(),
    }
}

fn rep(input: &str, env: &mut Env) -> String {
    let v = read(input).and_then(|v| eval(v, &mut *env));

    print(v)
}

fn main() {
    let mut ed = Editor::<()>::new();
    ed.load_history(".mal_history").ok();

    let mut repl_env = Env::new(None);
    repl_env.set("+".to_string(), MValue::function(add)); // to string
    repl_env.set("-".to_string(), MValue::function(sub)); // to string
    repl_env.set("*".to_string(), MValue::function(mul)); // to string
    repl_env.set("/".to_string(), MValue::function(div)); // to string
    repl_env.set("list".to_string(), MValue::function(list)); // to string
    repl_env.set("list?".to_string(), MValue::function(list_q)); // to string
    repl_env.set("empty?".to_string(), MValue::function(empty_q)); // to string
    repl_env.set("count".to_string(), MValue::function(count)); // to string
    repl_env.set("=".to_string(), MValue::function(eq)); // to string
    repl_env.set(">".to_string(), MValue::function(gt)); // to string
    repl_env.set("<".to_string(), MValue::function(lt)); // to string
    repl_env.set(">=".to_string(), MValue::function(gte)); // to string
    repl_env.set("<=".to_string(), MValue::function(lte)); // to string
    repl_env.set("prn".to_string(), MValue::function(prn)); // to string

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
