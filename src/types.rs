use MalVal::*;

use std::collections::HashMap;
use std::error;
use std::fmt::{self, Display};

use std::rc::Rc;

pub type FnExpr = fn(Vec<MValue>) -> Result<MValue>;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MValue(pub Rc<MalVal>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalVal {
    Int(i32),
    Bool(bool),
    List(Vec<MValue>),
    Vector(Vec<MValue>),
    HashMap(HashMap<String, MValue>),
    Sym(String),
    Str(String),
    Fun(FnExpr),
    Nil,
}

impl MValue {
    pub fn integer(value: i32) -> MValue {
        MValue(Rc::new(MalVal::Int(value)))
    }

    pub fn bool(value: bool) -> MValue {
        MValue(Rc::new(MalVal::Bool(value)))
    }

    pub fn list(value: Vec<MValue>) -> MValue {
        MValue(Rc::new(MalVal::List(value)))
    }

    pub fn vector(value: Vec<MValue>) -> MValue {
        MValue(Rc::new(MalVal::Vector(value)))
    }

    pub fn hashmap(value: HashMap<String, MValue>) -> MValue {
        MValue(Rc::new(MalVal::HashMap(value)))
    }

    pub fn symbol(value: String) -> MValue {
        MValue(Rc::new(MalVal::Sym(value)))
    }

    pub fn string<T: Into<String>>(value: T) -> MValue {
        MValue(Rc::new(MalVal::Str(value.into())))
    }

    pub fn function(value: FnExpr) -> MValue {
        MValue(Rc::new(MalVal::Fun(value)))
    }

    pub fn nil() -> MValue {
        MValue(Rc::new(MalVal::Nil))
    }

    pub fn is_list(&self) -> bool {
        match *self.0 {
            MalVal::List(_) => true,
            _ => false,
        }
    }

    pub fn is_hashmap(&self) -> bool {
        match *self.0 {
            MalVal::HashMap(_) => true,
            _ => false,
        }
    }

    pub fn is_vector(&self) -> bool {
        match *self.0 {
            MalVal::Vector(_) => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match *self.0 {
            MalVal::Sym(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match *self.0 {
            MalVal::Str(_) => true,
            _ => false,
        }
    }
    //
    // TODO: cast_to_int and cast_to_list are not consistent in term of borrowing
    pub fn cast_to_int(&self) -> Result<i32> {
        match *self.0 {
            MalVal::Int(x) => Ok(x),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_symbol(&self) -> Result<String> {
        match *self.0 {
            MalVal::Sym(ref x) => Ok(x.clone()),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_string(&self) -> Result<String> {
        match *self.0 {
            MalVal::Str(ref x) => Ok(x.clone()),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_bool(&self) -> Result<bool> {
        match *self.0 {
            MalVal::Bool(x) => Ok(x),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_fn(&self) -> Result<FnExpr> {
        match *self.0 {
            MalVal::Fun(x) => Ok(x),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_list(self) -> Result<Vec<MValue>> {
        match *self.0 {
            MalVal::List(ref x) | MalVal::Vector(ref x) => Ok(x.to_vec()),
            _ => Err(Error::EvalError),
        }
    }

    pub fn cast_to_hashmap(self) -> Result<HashMap<String, MValue>> {
        match *self.0 {
            MalVal::HashMap(ref x) => Ok(x.clone()),
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
impl Display for MValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            Int(ref k) =>     write!(f, "{}", k),
            Bool(ref b) =>    write!(f, "{}", b),
            List(ref l) =>    write!(f, "{}", print_sequence(&l, "(", ")")),
            Vector(ref l) =>  write!(f, "{}", print_sequence(&l, "[", "]")),
            HashMap(ref l) => {
                let l = l.iter()
                    .flat_map(|(k, v)| vec![MValue::string(k.to_string()), v.clone()])
                    .collect::<Vec<MValue>>();
                write!(f, "{}", print_sequence(&l, "{", "}"))
            },
            Sym(ref s) =>     write!(f, "{}", s),
            Str(ref s) =>     write!(f, "{}", s),
            Nil =>        write!(f, "nil"),
            Fun(fun) =>     write!(f, "{:?}", fun),
        }
    }
}

fn print_sequence(seq: &[MValue], start: &str, end: &str) -> String {
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

