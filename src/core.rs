use crate::types::*;
use crate::env::Env;
use crate::reader::read_form;
use std::convert::TryFrom;
use std::fs::read_to_string;

pub fn list(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::list(args))
}

pub fn vector(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::vector(args))
}

pub fn hashmap(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::hashmap(&mut args.clone()))
}

pub fn symbol(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::symbol(args[0].clone().cast_to_string()?))
}

pub fn keyword(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::keyword(args[0].clone().cast_to_string()?))
}

pub fn list_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().is_list();

    Ok(MValue::bool(x))
}

pub fn vector_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().is_vector();

    Ok(MValue::bool(x))
}

pub fn sequential_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone();

    Ok(MValue::bool(x.is_list() || x.is_vector()))
}

pub fn map_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().is_hashmap();

    Ok(MValue::bool(x))
}

pub fn symbol_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().is_symbol();

    Ok(MValue::bool(x))
}

pub fn nil_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().is_nil();

    Ok(MValue::bool(x))
}

pub fn true_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().cast_to_bool();

    Ok(MValue::bool(x == true))
}

pub fn false_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().cast_to_bool();

    Ok(MValue::bool(x == false))
}

pub fn keyword_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().is_keyword();

    Ok(MValue::bool(x))
}

pub fn empty_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let list = args[0].clone().cast_to_list()?;

    Ok(MValue::bool(list.is_empty()))
}

pub fn count(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone()
        .cast_to_list()
        .map(|v| v.len())
        .unwrap_or(0);

    Ok(MValue::integer(i32::try_from(x).unwrap()))
}

pub fn add(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter()
        .flat_map(MValue::cast_to_int)
        .sum();

    Ok(MValue::integer(x))
}

pub fn sub(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x -= y.cast_to_int()?;
    }

    Ok(MValue::integer(x))
}

pub fn mul(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter()
        .flat_map(MValue::cast_to_int)
        .product();

    Ok(MValue::integer(x))
}

pub fn div(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x /= y.cast_to_int()?;
    }

    Ok(MValue::integer(x))
}

pub fn eq(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone();
    let y = args[1].clone();

    Ok(MValue::bool(x == y))
}

pub fn lt(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x < y))
}

pub fn gt(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x > y))
}

pub fn lte(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x <= y))
}

pub fn gte(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x >= y))
}

pub fn print_str(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter()
        .map(|x| x.pr_str(true))
        .collect::<Vec<String>>();

    let r = x.join(" ");

    Ok(MValue::string(r))
}

pub fn string(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter()
        .map(|x| x.pr_str(false))
        .collect::<Vec<String>>();

    let r = x.join("");

    Ok(MValue::string(r))
}

pub fn prn(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter()
        .map(|x| x.pr_str(true))
        .collect::<Vec<String>>();

    let r = x.join(" ");

    println!("{}", r);
    Ok(MValue::nil())
}

pub fn println(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args.iter()
        .map(|x| x.pr_str(false))
        .collect::<Vec<String>>();

    let r = x.join(" ");

    println!("{}", r);
    Ok(MValue::nil())
}

pub fn read_str(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let string = args[0].cast_to_string()?;
    let parser = read_form();

    parser.parse(string.as_bytes()).map_err(From::from)
}

pub fn slurp(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let filename = args[0].cast_to_string()?;

    read_to_string(filename)
        .map(MValue::string)
        .map_err(From::from)
}

pub fn atom(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::atom(args[0].clone()))
}

pub fn atom_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::bool(args[0].is_atom()))
}

pub fn deref(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    args[0].atom_deref()
}

pub fn reset(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    args[0].atom_reset(args[1].clone())
}

pub fn cons(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let mut v = args[1].clone().cast_to_list()?;
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
    let k = usize::try_from(args[1].cast_to_int()?).unwrap();

    args[0].clone()
        .cast_to_list()?
        .get(k)
        .cloned()
        .ok_or_else(|| Error::EvalError("Out of bounds".to_string()))
}

pub fn first(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let value = &args[0];
    if !value.is_list() && !value.is_vector() {
        Ok(MValue::nil())
    } else {
        let list = value.clone().cast_to_list()?;
        if list.is_empty() {
            return Ok(MValue::nil());
        }

        Ok(list[0].clone())
    }
}

pub fn rest(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let value = &args[0];
    let l = vec![];

    if !value.is_list() && !value.is_vector() {
        Ok(MValue::list(l))
    } else {
        let list = value.clone().cast_to_list()?;

        if list.is_empty() {
            return Ok(MValue::list(l));
        }

        let l = list[1..].to_vec();
        Ok(MValue::list(l))
    }
}

pub fn throw(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Err(Error::Throw(args[0].clone()))
}

pub fn assoc(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let hm = args[0].clone();
    let mut args = args[1..].to_vec();
    hm.hassoc(&mut args)
}

pub fn dissoc(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let mut hm = args[0].clone().cast_to_hashmap()?;

    for key in args[1..].to_vec() {
        hm.remove(&(key.cast_to_string()?, key.enum_key()));
    }

    Ok(MValue::from_hashmap(hm))
}

pub fn get(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    if !args[0].is_hashmap() {
        return Ok(MValue::nil())
    }

    let map = args[0].clone().cast_to_hashmap().unwrap();
    let key = args[1].clone();

    Ok(map.get(&(key.cast_to_string()?, key.enum_key()))
        .cloned()
        .unwrap_or_else(|| MValue::nil()))
}

pub fn contains_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let map = args[0].clone().cast_to_hashmap().unwrap();
    let key = args[1].clone();

    Ok(MValue::bool(
            map.get(&(key.cast_to_string()?, key.enum_key()))
                .is_some()))
}

pub fn keys(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let map = args[0].clone().cast_to_hashmap().unwrap();
    let keys = map.keys()
        .map(MValue::reconstruct)
        .collect::<Result<Vec<MValue>>>();

    Ok(MValue::list(keys?))
}

pub fn values(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let map = args[0].clone().cast_to_hashmap().unwrap();
    let keys = map.values()
        .cloned()
        .collect::<Vec<MValue>>();

    Ok(MValue::list(keys))
}
