use crate::types::*;
use crate::env::Env;
use crate::reader::read_form;
use std::convert::TryFrom;
use std::fs::read_to_string;

pub fn list(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    Ok(MValue::list(args))
}

pub fn list_q(args: Vec<MValue>, _env: Option<Env>) -> Result<MValue> {
    let x = args[0].clone().is_list();

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
        .ok_or_else(|| Error::EvalError("Out of bonds".to_string()))
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
