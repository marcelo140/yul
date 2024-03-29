use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::env::Env;
use crate::reader::read_form;
use crate::types::*;

use std::convert::TryFrom;
use std::fs::read_to_string;
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! assert_min_args {
    ($args:expr, $min:literal) => {{
        if $args.len() < $min {
            return Err(Error::EvalError(format!(
                "Function requires at least {} argument(s).",
                $min
            )));
        }
    }};
}

pub fn list(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::list(args))
}

pub fn vector(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::vector(args))
}

pub fn hashmap(mut args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::hashmap(&mut args))
}

pub fn symbol(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Ok(MValue::symbol(args[0].cast_to_string()?))
}

pub fn keyword(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Ok(MValue::keyword(args[0].cast_to_string()?))
}

pub fn list_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].is_list();

    Ok(MValue::bool(x))
}

pub fn vector_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].is_vector();

    Ok(MValue::bool(x))
}

pub fn sequential_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].is_list() || args[0].is_vector();

    Ok(MValue::bool(x))
}

pub fn map_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].is_hashmap();

    Ok(MValue::bool(x))
}

pub fn symbol_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].is_symbol();

    Ok(MValue::bool(x))
}

pub fn nil_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].is_nil();

    Ok(MValue::bool(x))
}

pub fn true_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].cast_to_bool();

    Ok(MValue::bool(x))
}

pub fn false_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].cast_to_bool();

    Ok(MValue::bool(!x))
}

pub fn keyword_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let x = args[0].is_keyword();

    Ok(MValue::bool(x))
}

pub fn empty_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let list = args[0].cast_to_list()?;

    Ok(MValue::bool(list.is_empty()))
}

pub fn count(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let x = args[0].cast_to_list().map(|v| v.len()).unwrap_or(0);
    let x = i64::try_from(x)?;

    Ok(MValue::integer(x))
}

pub fn add(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter().flat_map(MValue::cast_to_int).sum();

    Ok(MValue::integer(x))
}

pub fn sub(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x -= y.cast_to_int()?;
    }

    Ok(MValue::integer(x))
}

pub fn mul(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter().flat_map(MValue::cast_to_int).product();

    Ok(MValue::integer(x))
}

pub fn div(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x /= y.cast_to_int()?;
    }

    Ok(MValue::integer(x))
}

pub fn eq(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);
    Ok(MValue::bool(args[0] == args[1]))
}

pub fn lt(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x < y))
}

pub fn gt(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x > y))
}

pub fn lte(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x <= y))
}

pub fn gte(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x >= y))
}

pub fn print_str(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter().map(|x| x.pr_str(true)).collect::<Vec<String>>();

    let r = x.join(" ");

    Ok(MValue::string(r))
}

pub fn string(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args
        .iter()
        .map(|x| x.pr_str(false))
        .collect::<Vec<String>>();

    let r = x.join("");

    Ok(MValue::string(r))
}

pub fn prn(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter().map(|x| x.pr_str(true)).collect::<Vec<String>>();

    let r = x.join(" ");

    println!("{}", r);
    Ok(MValue::nil())
}

pub fn println(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args
        .iter()
        .map(|x| x.pr_str(false))
        .collect::<Vec<String>>();

    let r = x.join(" ");

    println!("{}", r);
    Ok(MValue::nil())
}

pub fn read_str(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let string = args[0].cast_to_string()?;
    let parser = read_form();

    parser.parse(string.as_bytes()).map_err(From::from)
}

pub fn slurp(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let filename = args[0].cast_to_string()?;

    read_to_string(filename)
        .map(MValue::string)
        .map_err(From::from)
}

pub fn atom(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Ok(MValue::atom(args[0].clone()))
}

pub fn atom_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Ok(MValue::bool(args[0].is_atom()))
}

pub fn deref(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    args[0].atom_deref()
}

pub fn reset(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);
    args[0].atom_reset(args[1].clone())
}

pub fn cons(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let mut v = args[1].cast_to_list()?;
    v.insert(0, args[0].clone());

    Ok(MValue::list(v))
}

pub fn concat(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let mut v = Vec::new();

    for arg in args {
        v.append(&mut arg.cast_to_list()?);
    }

    Ok(MValue::list(v))
}

pub fn nth(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let i = args[1].cast_to_int()?;
    let k = usize::try_from(i)?;

    args[0]
        .cast_to_list()?
        .get(k)
        .cloned()
        .ok_or_else(|| Error::EvalError("Out of bounds".to_string()))
}

pub fn first(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let value = &args[0];
    if !value.is_list() && !value.is_vector() {
        Ok(MValue::nil())
    } else {
        let list = value.cast_to_list()?;
        if list.is_empty() {
            return Ok(MValue::nil());
        }

        Ok(list[0].clone())
    }
}

pub fn rest(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let value = &args[0];
    let l = vec![];

    if !value.is_list() && !value.is_vector() {
        Ok(MValue::list(l))
    } else {
        let list = value.cast_to_list()?;

        if list.is_empty() {
            return Ok(MValue::list(l));
        }

        let l = list[1..].to_vec();
        Ok(MValue::list(l))
    }
}

pub fn throw(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Err(Error::Throw(args[0].clone()))
}

pub fn assoc(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let hm = &args[0];
    let mut args = args[1..].to_vec();
    hm.hassoc(&mut args)
}

pub fn dissoc(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let mut hm = args[0].cast_to_hashmap()?;

    for key in args[1..].to_vec() {
        hm.remove(&(key.cast_to_string()?, key.enum_key()));
    }

    Ok(MValue::from_hashmap(hm))
}

pub fn get(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    if !args[0].is_hashmap() {
        return Ok(MValue::nil());
    }

    let map = args[0].cast_to_hashmap()?;
    let key = args[1].cast_to_string()?;
    let key_type = args[1].enum_key();

    let r = map
        .get(&(key, key_type))
        .cloned()
        .unwrap_or_else(MValue::nil);
    Ok(r)
}

pub fn contains_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);

    let map = args[0].cast_to_hashmap()?;
    let key = args[1].cast_to_string()?;
    let key_type = args[1].enum_key();

    let r = map.get(&(key, key_type)).is_some();
    Ok(MValue::bool(r))
}

pub fn keys(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let map = args[0].cast_to_hashmap()?;
    let keys = map.keys().map(MValue::reconstruct).collect::<Result<_>>()?;

    Ok(MValue::list(keys))
}

pub fn values(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let map = args[0].cast_to_hashmap()?;
    let keys = map.values().cloned().collect::<Vec<_>>();

    Ok(MValue::list(keys))
}

pub fn readline(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    let prompt = args[0].cast_to_string()?;
    let mut ed = Editor::<()>::new();

    let usr_str = ed.readline(&prompt);

    match usr_str {
        Ok(usr_str) => Ok(MValue::string(usr_str)),
        Err(ReadlineError::Eof) => Ok(MValue::nil()),
        Err(err) => Err(Error::IoError(format!("Failed readling line: {:?}", err))),
    }
}

pub fn time_ms(_args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::EvalError("System time is earlier than epoch".to_string()))
        .and_then(|d| i64::try_from(d.as_millis()).map_err(From::from))
        .map(MValue::integer)
}

pub fn meta(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    args[0].meta()
}

pub fn with_meta(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);
    args[0].with_meta(args[1].clone())
}

pub fn fn_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    let r = (args[0].is_builtin() || args[0].is_lambda()) && !args[0].is_macro();
    Ok(MValue::bool(r))
}

pub fn string_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Ok(MValue::bool(args[0].is_string()))
}

pub fn number_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Ok(MValue::bool(args[0].is_number()))
}

pub fn macro_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);
    Ok(MValue::bool(args[0].is_macro()))
}

pub fn seq(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 1);

    if args[0].is_nil() {
        return Ok(args[0].clone());
    }

    let l = args[0].cast_to_list()?;

    if l.is_empty() {
        return Ok(MValue::nil());
    }

    Ok(MValue::list(l))
}

pub fn conj(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    assert_min_args!(&args, 2);
    let v = &args[0];

    if v.is_list() {
        let mut v = v.cast_to_list()?;
        let mut head = args[1..].to_vec();
        head.reverse();
        head.append(&mut v);
        return Ok(MValue::list(head));
    }

    if v.is_vector() {
        let mut v = v.cast_to_list()?;
        let mut head = args[1..].to_vec();
        v.append(&mut head);
        return Ok(MValue::vector(v));
    }

    Ok(MValue::nil())
}
