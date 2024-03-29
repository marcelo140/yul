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
           .map(MValue::from_hashmap)
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

        input = macro_expand(input, &env)?;

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
                let condition = eval(l[1].clone(), &env)?;
                match *condition.0 {
                    MalVal::Bool(false) | MalVal::Nil if l.len() >= 4 =>
                        input = l[3].clone(),
                    MalVal::Bool(false) | MalVal::Nil =>
                        return Ok(MValue::nil()),
                    _ =>
                        input = l[2].clone(),
                }
            },

            MalVal::Sym(ref sym) if sym == "macroexpand" => {
                return macro_expand(l[1].clone(), &env);
            },

            MalVal::Sym(ref sym) if sym == "quote" => {
                return Ok(l[1].clone());
            },

            MalVal::Sym(ref sym) if sym == "quasiquote" => {
                input = quasiquote(l[1].clone())?;
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
                let v = eval(l[2].clone(), &env)?;
                env.set(key, v.clone());
                return Ok(v);
            },

            MalVal::Sym(ref sym) if sym == "try*" => {
                let try_expr = l[1].clone();

                let result = eval(try_expr, &env);

                if result.is_ok() || l.len() < 3 {
                    return result;
                }

                let error = result.unwrap_err();
                let catch_block = l[2].clone().cast_to_list()?;
                let err_symbol = catch_block[1].clone().cast_to_string()?;
                let catch_expr = catch_block[2].clone();
                env.set(err_symbol, error.catch());
                return eval(catch_expr, &env);
            },

            MalVal::Sym(ref sym) if sym == "defmacro!" => {
                let key = l[1].cast_to_string()?;
                let mut v = eval(l[2].clone(), &env)?;
                v.set_macro();
                env.set(key, v.clone());
                return Ok(v);
            },

            MalVal::Sym(ref sym) if sym == "let*" => {
                env = Env::new(Some(env.clone()));

                let binds = l[1].clone().cast_to_list()?;

                for (bind, expr) in binds.clone().into_iter().tuples() {
                    let bind = bind.cast_to_string()?;
                    let v = eval(expr, &env)?;

                    env.set(bind, v);
                }

                input = l[2].clone();
            },


            _ => {
                let evaluated_list = eval_ast(MValue::list(l), &env)?.cast_to_list()?;

                return handle_function(&evaluated_list[0], evaluated_list[1..].to_vec());
            },
        }
    }
}

fn is_nonempty_list(value: &MValue) -> bool {
    (value.is_list() || value.is_vector()) && !value.clone().cast_to_list().unwrap().is_empty()
}

fn quasiquote(value: MValue) -> Result<MValue> {
    if !is_nonempty_list(&value) {
        return Ok(MValue::list(vec![MValue::symbol("quote"), value.clone()]));
    }

    let ast = value.clone().cast_to_list()?;

    if let MalVal::Sym(ref unquote) = *ast[0].0 {
        if unquote == "unquote" {
            return Ok(value.cast_to_list()?[1].clone());
        }
    }

    let rest = MValue::list(ast[1..].to_vec());

    if is_nonempty_list(&ast[0]) {
        let m1 = ast[0].clone().cast_to_list()?;
        if let MalVal::Sym(ref splice_unquote) = *m1[0].0 {
            if splice_unquote == "splice-unquote" {
                return Ok(MValue::list(
                        vec![MValue::symbol("concat"),
                        m1[1].clone(),
                        quasiquote(rest)?]));
            }
        }
    }

    Ok(MValue::list(vec![
                    MValue::symbol("cons"),
                    quasiquote(ast[0].clone())?,
                    quasiquote(rest)?]))
}

fn meval(args: Vec<MValue>, env: Option<Env>) -> Result<MValue> {
    eval(args[0].clone(), &env.unwrap())
}

pub fn swap(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let mut args = args.clone();
    let atom = args.remove(0);
    let f = args.remove(0);

    args.insert(0, atom.atom_deref()?);

    let v = handle_function(&f, args)?;
    Ok(atom.atom_reset(v)?)
}

fn handle_function(function: &MValue, args: Vec<MValue>) -> Result<MValue> {
    match *function.0 {
        MalVal::Fun(fun, ref env, _) => {
            fun(args, env.clone())
        }, 

        MalVal::Lambda(ref fun, _) => {
            let (body, new_env) = fun.apply(args)?;
            eval(body, &new_env)
        },

        _ => Err(Error::EvalError(format!("No function provided: {:?}", *function.0)))
    }
}

pub fn apply(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let mut args = args.clone();
    let f = args.remove(0);

    let mut last_arguments = args.pop()
        .ok_or_else(|| Error::EvalError("Not enough arguments".to_string()))?
        .cast_to_list()?;

    args.append(&mut last_arguments);

    handle_function(&f, args)
}

pub fn map(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let f = args[0].clone();
    let values: Vec<MValue> = args[1].clone().cast_to_list()?;

    values.iter()
        .map(|v| handle_function(&f, vec![v.clone()]))
        .collect::<Result<Vec<MValue>>>()
        .map(MValue::list)
}

pub fn macro_expand(value: MValue, env: &Env) -> Result<MValue> {
    let mut value = value;

    while value.is_macro_call(&env) {
        let list = value.clone().cast_to_list()?;
        let lambda = env.get(&list[0].cast_to_string()?).unwrap();
        value = handle_function(&lambda, list[1..].to_vec())?;
    }

    Ok(value)
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
    repl_env.set("+", MValue::function(add, None));
    repl_env.set("-", MValue::function(sub, None));
    repl_env.set("*", MValue::function(mul, None));
    repl_env.set("/", MValue::function(div, None));
    repl_env.set("list", MValue::function(list, None));
    repl_env.set("vector", MValue::function(vector, None));
    repl_env.set("hash-map", MValue::function(hashmap, None));
    repl_env.set("symbol", MValue::function(symbol, None));
    repl_env.set("keyword", MValue::function(keyword, None));
    repl_env.set("list?", MValue::function(list_q, None));
    repl_env.set("vector?", MValue::function(vector_q, None));
    repl_env.set("sequential?", MValue::function(sequential_q, None));
    repl_env.set("map?", MValue::function(map_q, None));
    repl_env.set("empty?", MValue::function(empty_q, None));
    repl_env.set("assoc", MValue::function(assoc, None));
    repl_env.set("dissoc", MValue::function(dissoc, None));
    repl_env.set("count", MValue::function(count, None));
    repl_env.set("=", MValue::function(eq, None));
    repl_env.set(">", MValue::function(gt, None));
    repl_env.set("<", MValue::function(lt, None));
    repl_env.set(">=", MValue::function(gte, None));
    repl_env.set("<=", MValue::function(lte, None));
    repl_env.set("pr-str", MValue::function(print_str, None));
    repl_env.set("str", MValue::function(string, None));
    repl_env.set("prn", MValue::function(prn, None));
    repl_env.set("println", MValue::function(println, None));
    repl_env.set("read-string", MValue::function(read_str, None));
    repl_env.set("slurp", MValue::function(slurp, None));
    repl_env.set("atom", MValue::function(atom, None));
    repl_env.set("atom?", MValue::function(atom_q, None));
    repl_env.set("deref", MValue::function(deref, None));
    repl_env.set("reset!", MValue::function(reset, None));
    repl_env.set("swap!", MValue::function(swap, Some(repl_env.clone())));
    repl_env.set("cons", MValue::function(cons, None));
    repl_env.set("concat", MValue::function(concat, None));
    repl_env.set("nth", MValue::function(nth, None));
    repl_env.set("first", MValue::function(first, None));
    repl_env.set("rest", MValue::function(rest, None));
    repl_env.set("throw", MValue::function(throw, None));
    repl_env.set("apply", MValue::function(apply, None));
    repl_env.set("map", MValue::function(map, None));
    repl_env.set("symbol?", MValue::function(symbol_q, None));
    repl_env.set("nil?", MValue::function(nil_q, None));
    repl_env.set("true?", MValue::function(true_q, None));
    repl_env.set("false?", MValue::function(false_q, None));
    repl_env.set("keyword?", MValue::function(keyword_q, None));
    repl_env.set("get", MValue::function(get, None));
    repl_env.set("contains?", MValue::function(contains_q, None));
    repl_env.set("keys", MValue::function(keys, None));
    repl_env.set("vals", MValue::function(values, None));
    repl_env.set("readline", MValue::function(readline, None));
    repl_env.set("time-ms", MValue::function(time_ms, None));
    repl_env.set("meta", MValue::function(meta, None));
    repl_env.set("with-meta", MValue::function(with_meta, None));
    repl_env.set("fn?", MValue::function(fn_q, None));
    repl_env.set("string?", MValue::function(string_q, None));
    repl_env.set("number?", MValue::function(number_q, None));
    repl_env.set("macro?", MValue::function(macro_q, None));
    repl_env.set("seq", MValue::function(seq, None));
    repl_env.set("conj", MValue::function(conj, None));
    repl_env.set("*host-language*", MValue::string("Rust"));
    repl_env.set("eval", MValue::function(meval, Some(repl_env.clone())));

    rep("(def! not (fn* (a) (if a false true)))", &repl_env);
    rep("(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))"
        , &repl_env);
    rep("(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw \"odd number of forms to cond\")) (cons 'cond (rest (rest xs)))))))", &repl_env);
    rep("(def! *gensym-counter* (atom 0))", &repl_env);
    rep("(def! gensym (fn* [] (symbol (str \"G__\" (swap! *gensym-counter* (fn* [x] (+ 1 x)))))))", &repl_env);
    rep("(defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) (let* (condvar (gensym)) `(let* (~condvar ~(first xs)) (if ~condvar ~condvar (or ~@(rest xs)))))))))", &repl_env);

    let mut argv = args().skip(1);
    let path = argv.next();
    let margv = argv.map(MValue::string).collect();
    repl_env.set("*ARGV*", MValue::list(margv));

    if let Some(path) = path {
        let command = format!("(load-file \"{}\")", path);
        rep(&command, &repl_env);
        return;
    }

    rep("(println (str \"Mal [\" *host-language* \"]\"))", &repl_env);

    loop {
        let line = ed.readline("user> ");

        match line {
            Ok(line) => {
                println!("{}", &rep(&line, &repl_env));
                ed.add_history_entry(line);
                ed.save_history(".mal_history").ok();
            },
            Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }
}
