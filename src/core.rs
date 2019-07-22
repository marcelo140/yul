use crate::types::*;
use std::convert::TryFrom;

pub fn list(args: Vec<MValue>) -> Result<MValue> {
    Ok(MValue::list(args))
}

pub fn list_q(args: Vec<MValue>) -> Result<MValue> {
    let x = args[0].clone().is_list();

    Ok(MValue::bool(x))
}

pub fn empty_q(args: Vec<MValue>) -> Result<MValue> {
    let list = args[0].clone().cast_to_list()?;

    Ok(MValue::bool(list.is_empty()))
}

pub fn count(args: Vec<MValue>) -> Result<MValue> {
    let x = args[0].clone()
        .cast_to_list()
        .map(|v| v.len())
        .unwrap_or(0);

    Ok(MValue::integer(i32::try_from(x).unwrap()))
}

pub fn add(args: Vec<MValue>) -> Result<MValue> {
    let x = args.iter()
        .flat_map(MValue::cast_to_int)
        .sum();

    Ok(MValue::integer(x))
}

pub fn sub(args: Vec<MValue>) -> Result<MValue> {
    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x -= y.cast_to_int()?;
    }

    Ok(MValue::integer(x))
}

pub fn mul(args: Vec<MValue>) -> Result<MValue> {
    let x = args.iter()
        .flat_map(MValue::cast_to_int)
        .product();

    Ok(MValue::integer(x))
}

pub fn div(args: Vec<MValue>) -> Result<MValue> {
    let mut x = args[0].cast_to_int()?;

    for y in args[1..].iter() {
        x /= y.cast_to_int()?;
    }

    Ok(MValue::integer(x))
}

pub fn eq(args: Vec<MValue>) -> Result<MValue> {
    let x = args[0].clone();
    let y = args[1].clone();

    Ok(MValue::bool(x == y))
}

pub fn lt(args: Vec<MValue>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x < y))
}

pub fn gt(args: Vec<MValue>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x > y))
}

pub fn lte(args: Vec<MValue>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x <= y))
}

pub fn gte(args: Vec<MValue>) -> Result<MValue> {
    let x = args[0].cast_to_int()?;
    let y = args[1].cast_to_int()?;

    Ok(MValue::bool(x >= y))
}

pub fn prn(args: Vec<MValue>) -> Result<MValue> {
    let x = args.get(0).map(ToString::to_string).unwrap_or_else(|| "".to_string());

    println!("{}", x);
    Ok(MValue::nil())
}
