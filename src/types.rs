use MalVal::*;

use std::collections::HashMap;
use std::error;
use std::fmt::{self, Display};

pub type FnExpr = fn(Vec<MalVal>) -> Result<MalVal>;
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum MalVal {
    Int(i32),
    Bool(bool),
    List(Vec<MalVal>),
    Vector(Vec<MalVal>),
    HashMap(HashMap<String, MalVal>),
    Sym(String),
    Str(String),
    Fun(FnExpr),
    Nil,
}

impl MalVal {
    // TODO: cast_to_int and cast_to_list are not consistent in term of borrowing
    pub fn cast_to_int(&self) -> Result<i32> {
        match *self {
            MalVal::Int(x) => Ok(x),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_sym(&self) -> Result<String> {
        match self {
            MalVal::Sym(x) => Ok(x.clone()),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_fn(&self) -> Result<FnExpr> {
        match *self {
            MalVal::Fun(x) => Ok(x),
            _ => Err(Error::EvalError),
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            MalVal::List(_x) => true,
            _ => false,
        }
    }

    pub fn cast_to_list(self) -> Result<Vec<MalVal>> {
        match self {
            MalVal::List(x) | MalVal::Vector(x) => Ok(x),
            _ => Err(Error::EvalError),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ParseError,
    EvalError,
    ArgsError,
    NoSymbolFound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError => write!(f, "Parse error"),
            Error::EvalError => write!(f, "Eval error"),
            Error::ArgsError => write!(f, "Args error"),
            Error::NoSymbolFound(s) => write!(f, "{} not found", s),
        }
    }
}

// TODO: Refactor fmt
impl Display for MalVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Int(k) =>     write!(f, "{}", k),
            Bool(b) =>    write!(f, "{}", b),
            List(l) =>    write!(f, "{}", print_sequence(l, "(", ")")),
            Vector(l) =>  write!(f, "{}", print_sequence(l, "[", "]")),
            HashMap(l) => {
                let l = l.iter()
                    .flat_map(|(k, v)| vec![MalVal::Str(k.to_string()), v.clone()])
                    .collect::<Vec<MalVal>>();
                write!(f, "{}", print_sequence(&l, "{", "}"))
            },
            Sym(s) =>     write!(f, "{}", s),
            Str(s) =>     write!(f, "{}", s),
            Nil =>        write!(f, "nil"),
            Fun(fun) =>     write!(f, "{:?}", fun),
        }
    }
}

fn print_sequence(seq: &[MalVal], start: &str, end: &str) -> String {
    let seq: Vec<String> = seq
        .iter()
        .map(ToString::to_string)
        .collect();

    format!("{}{}{}", start, seq.join(" "), end)
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::ParseError => "Parse error",
            Error::EvalError => "Eval error",
            Error::ArgsError => "Args error",
            Error::NoSymbolFound(_s) => "No symbol found",
        }
    }
}

impl From<pom::Error> for Error {
    fn from(_error: pom::Error) -> Error {
        Error::ParseError
    }
}

