extern crate rust;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use itertools::Itertools;

use rust::reader::*;
use rust::types::*;
use rust::env::Env;
use rust::core::*;

use std::env::args;

fn eval_ast(value: MValue, env: &Env) -> Result<MValue> {
    if value.is_symbol() {
        let x = value.cast_to_string()?;
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
    let mut env = env.clone();
    let mut input = input.clone();

    loop {
        if !input.is_list() {
            return eval_ast(input, &env);
        }

        let mut l = input.clone().cast_to_list()?;

        if l.is_empty() {
            return Ok(MValue::list(l));
        }

        match *l[0].0 {
            MalVal::Sym(ref sym) if sym == "do" => {
                input = l
                    .pop()
                    .ok_or_else(|| Error::EvalError(
                            "No argument was provided".to_string()))?;

                let v = MValue::list(l[1..].to_vec());
                eval_ast(v, &env)?;
            },

            MalVal::Sym(ref sym) if sym == "if" => {
                let condition = eval(l[1].clone(), &env)?; // MValue clone
                match *condition.0 {
                    MalVal::Bool(false) | MalVal::Nil if l.len() >= 4 =>
                        input = l[3].clone(),
                    MalVal::Bool(false) | MalVal::Nil =>
                        return Ok(MValue::nil()),
                    _ =>
                        input = l[2].clone(),
                }
            },

            MalVal::Sym(ref sym) if sym == "fn*" => {
                let parameters = l[1].clone()
                    .cast_to_list()?
                    .iter()
                    .flat_map(MValue::cast_to_string)
                    .collect::<Vec<String>>();

                let body = l[2].clone();

                return Ok(MValue::lambda(env.clone(), parameters, body));
            },

            MalVal::Sym(ref sym) if sym == "def!" => {
                let key = l[1].cast_to_string()?;
                let v = eval(l[2].clone(), &env)?; // malval clone
                env.set(key, v.clone()); // malval clone
                return Ok(v);
            },

            MalVal::Sym(ref sym) if sym == "let*" => {
                env = Env::new(Some(env.clone()));

                let binds = l[1].clone().cast_to_list()?; // malval clone

                for (bind, expr) in binds.clone().into_iter().tuples() {
                    let bind = bind.cast_to_string()?;
                    let v = eval(expr, &env)?;

                    env.set(bind, v);
                }

                input = l[2].clone();
            },

            _ => {
                let evaluated_list = eval_ast(MValue::list(l), &env)?.cast_to_list()?;
                let args = evaluated_list[1..].to_vec();

                if let MalVal::Fun(fun, ref env) = *evaluated_list[0].0 {
                    return fun(args, env.clone());
                } else if let MalVal::Lambda(ref fun) = *evaluated_list[0].0 {
                    let (body, new_env) = fun.apply(args);
                    input = body;
                    env = new_env;
                } else {
                    return Err(Error::EvalError(format!("{:?}", evaluated_list)));
                }
            },
        }
    }
}

fn meval(args: Vec<MValue>, env: Option<Env>) -> Result<MValue> {
    eval(args[0].clone(), &env.unwrap())
}

pub fn swap(args: Vec<MValue>, env: Option<Env>) -> Result<MValue> {
    let mut args = args.clone();
    let atom = args.remove(0);
    args.insert(1, atom.atom_deref()?);

    let v = eval(MValue::list(args), &env.unwrap())?;
    atom.atom_reset(v)
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
    repl_env.set("+".to_string(), MValue::function(add, None));
    repl_env.set("-".to_string(), MValue::function(sub, None));
    repl_env.set("*".to_string(), MValue::function(mul, None));
    repl_env.set("/".to_string(), MValue::function(div, None));
    repl_env.set("list".to_string(), MValue::function(list, None));
    repl_env.set("list?".to_string(), MValue::function(list_q, None));
    repl_env.set("empty?".to_string(), MValue::function(empty_q, None));
    repl_env.set("count".to_string(), MValue::function(count, None));
    repl_env.set("=".to_string(), MValue::function(eq, None));
    repl_env.set(">".to_string(), MValue::function(gt, None));
    repl_env.set("<".to_string(), MValue::function(lt, None));
    repl_env.set(">=".to_string(), MValue::function(gte, None));
    repl_env.set("<=".to_string(), MValue::function(lte, None));
    repl_env.set("pr-str".to_string(), MValue::function(print_str, None));
    repl_env.set("str".to_string(), MValue::function(string, None));
    repl_env.set("prn".to_string(), MValue::function(prn, None));
    repl_env.set("println".to_string(), MValue::function(println, None));
    repl_env.set("read-string".to_string(), MValue::function(read_str, None));
    repl_env.set("slurp".to_string(), MValue::function(slurp, None));
    repl_env.set("atom".to_string(), MValue::function(atom, None));
    repl_env.set("atom?".to_string(), MValue::function(atom_q, None));
    repl_env.set("deref".to_string(), MValue::function(deref, None));
    repl_env.set("reset!".to_string(), MValue::function(reset, None));
    repl_env.set("swap!".to_string(), MValue::function(swap, Some(repl_env.clone())));
    repl_env.set("eval".to_string(), MValue::function(meval, Some(repl_env.clone())));

    rep("(def! not (fn* (a) (if a false true)))", &repl_env);
    rep("(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))"
        , &repl_env);

    let mut argv = args().skip(1);
    let path = argv.next();
    let margv = argv.map(MValue::string).collect();
    repl_env.set("*ARGV*".to_string(), MValue::list(margv));

    if let Some(path) = path {
        let command = format!("(load-file \"{}\")", path);
        rep(&command, &repl_env);
        return;
    }

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
