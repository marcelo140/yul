extern crate rust;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use itertools::Itertools;

use rust::reader::*;
use rust::types::*;
use rust::env::Env;
use rust::core::*;

fn eval_ast(value: MValue, env: &Env) -> Result<MValue> {
    if value.is_symbol() {
        let x = value.cast_to_symbol()?;
        env.get(&x)
           .ok_or_else(|| Error::NoSymbolFound(x))
    } else if value.is_list() {
        value.cast_to_list()?.into_iter()
           .map(|x| eval(x, &env))
           .collect::<Result<_>>()
           .map(MValue::list)
    } else if value.is_hashmap() {
        value.cast_to_hashmap()?.into_iter()
           .map(|(k, v)| eval(v, &env).map(|v| (k,v)) )
           .collect::<Result<_>>()
           .map(MValue::hashmap)
    } else if value.is_vector() {
        value.cast_to_list()?.into_iter()
           .map(|x| eval(x, &env))
           .collect::<Result<_>>()
           .map(MValue::vector)
    } else {
        Ok(value)
    }
}

fn read(input: &str) -> Result<MValue> {
    read_form().parse(input.as_bytes()).map_err(From::from)
}

fn eval(input: MValue, env: &Env) -> Result<MValue> {
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
            eval_ast(v, env)?.cast_to_list()?.pop().ok_or_else(|| Error::EvalError("Expected additional element".to_string()))
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
            let binds = l[1].clone()
                .cast_to_list()?
                .iter()
                .flat_map(MValue::cast_to_symbol)
                .collect::<Vec<String>>();

            let body = l[2].clone();

            Ok(MValue::lambda(env.clone(), binds, body))
        },

        MalVal::Sym(ref sym) if sym == "def!" => {
            let key = l[1].cast_to_symbol()?;
            let v = eval(l[2].clone(), env)?; // malval clone
            env.set(key, v.clone()); // malval clone
            Ok(v)
        },

        MalVal::Sym(ref sym) if sym == "let*" => {
            let env = Env::new(Some(env.clone()));

            let binds = l[1].clone().cast_to_list()?; // malval clone

            for (bind, expr) in binds.clone().into_iter().tuples() {
                let bind = bind.cast_to_symbol()?;
                let v = eval(expr, &env)?;

                env.set(bind, v);
            }

            eval(l[2].clone(), &env) // malval clone
        },

        _ => {
            let evaluated_list = eval_ast(MValue::list(l), env)?.cast_to_list()?;
            let args = evaluated_list[1..].to_vec();

            if let MalVal::Fun(fun) = *evaluated_list[0].0 {
                fun(args)
            } else if let MalVal::Lambda(ref fun) = *evaluated_list[0].0 {
                let (val, env) = fun.apply(args);
                eval(val, &env)
            } else {
                Err(Error::EvalError(format!("{:?}", evaluated_list)))
            }
        },
    }
}

fn print(input: Result<MValue>) -> String {
    match input {
        Ok(mvalue) => mvalue.pr_str(true),
        Err(error) => error.to_string(),
    }
}

fn rep(input: &str, env: &Env) -> String {
    let v = read(input).and_then(|v| eval(v, &env));

    print(v)
}

fn main() {
    let mut ed = Editor::<()>::new();
    ed.load_history(".mal_history").ok();

    let repl_env = Env::new(None);
    repl_env.set("+".to_string(), MValue::function(add));
    repl_env.set("-".to_string(), MValue::function(sub));
    repl_env.set("*".to_string(), MValue::function(mul));
    repl_env.set("/".to_string(), MValue::function(div));
    repl_env.set("list".to_string(), MValue::function(list));
    repl_env.set("list?".to_string(), MValue::function(list_q));
    repl_env.set("empty?".to_string(), MValue::function(empty_q));
    repl_env.set("count".to_string(), MValue::function(count));
    repl_env.set("=".to_string(), MValue::function(eq));
    repl_env.set(">".to_string(), MValue::function(gt));
    repl_env.set("<".to_string(), MValue::function(lt));
    repl_env.set(">=".to_string(), MValue::function(gte));
    repl_env.set("<=".to_string(), MValue::function(lte));
    repl_env.set("pr-str".to_string(), MValue::function(print_str));
    repl_env.set("str".to_string(), MValue::function(string));
    repl_env.set("prn".to_string(), MValue::function(prn));
    repl_env.set("println".to_string(), MValue::function(println));

    rep("(def! not (fn* (a) (if a false true)))", &repl_env);

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
