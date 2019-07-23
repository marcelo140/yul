use MalVal::*;

use std::collections::HashMap;
use std::fmt::{self, Display};

use std::rc::Rc;
use crate::env::Env;

pub type FnExpr = fn(Vec<MValue>) -> Result<MValue>;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct MValue(pub Rc<MalVal>);

#[derive(Debug, Clone)]
pub enum MalVal {
    Int(i32),
    Bool(bool),
    List(Vec<MValue>),
    Vector(Vec<MValue>),
    HashMap(HashMap<String, MValue>),
    Sym(String),
    Str(String),
    Keyword(String),
    Fun(FnExpr),
    Lambda(MClosure),
    Nil,
}

#[derive(Debug, Clone)]
pub struct MClosure {
    env: Env,
    binds: Vec<String>,
    body: MValue,
}

impl MClosure {
    pub fn new(env: Env, binds: Vec<String>, body: MValue) -> Self {
        MClosure {
            env,
            binds,
            body,
        }
    }

    pub fn apply(&self, exprs: Vec<MValue>) -> (MValue, Env) {
        let copy = self.clone();
        let env = Env::new(Some(copy.env), copy.binds, exprs);

        (copy.body, env)
    }
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

    pub fn keyword(value: String) -> MValue {
        MValue(Rc::new(MalVal::Keyword(value)))
    }

    pub fn function(value: FnExpr) -> MValue {
        MValue(Rc::new(MalVal::Fun(value)))
    }

    pub fn lambda(env: Env, binds: Vec<String>, body: MValue) -> MValue {
        MValue(Rc::new(MalVal::Lambda(MClosure {
            env,
            binds,
            body,
        })))
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
            _ => Err(Error::EvalError(format!("{} is not a list!", self))),
        }
    }

    pub fn cast_to_symbol(&self) -> Result<String> {
        match *self.0 {
            MalVal::Sym(ref x) => Ok(x.clone()),
            _ => Err(Error::EvalError(format!("{} is not a symbol", self))),
        }
    }

    pub fn cast_to_string(&self) -> Result<String> {
        match *self.0 {
            MalVal::Keyword(ref x) | MalVal::Str(ref x) => Ok(x.clone()),
            _ => Err(Error::EvalError(format!("{} is not a string", self))),
        }
    }

    pub fn cast_to_bool(&self) -> Result<bool> {
        match *self.0 {
            MalVal::Bool(x) => Ok(x),
            _ => Err(Error::EvalError(format!("{} is not a bool", self))),
        }
    }

    pub fn cast_to_fn(&self) -> Result<FnExpr> {
        match *self.0 {
            MalVal::Fun(x) => Ok(x),
            _ => Err(Error::EvalError(format!("{} is not a function", self))),
        }
    }

    pub fn cast_to_list(self) -> Result<Vec<MValue>> {
        match *self.0 {
            MalVal::List(ref x) | MalVal::Vector(ref x) => Ok(x.to_vec()),
            _ => Err(Error::EvalError(format!("{} is not a list", self))),
        }
    }

    pub fn cast_to_hashmap(self) -> Result<HashMap<String, MValue>> {
        match *self.0 {
            MalVal::HashMap(ref x) => Ok(x.clone()),
            _ => Err(Error::EvalError(format!("{} is not a hasmap", self))),
        }
    }

    pub fn pr_str(&self, readably: bool) -> String {
        match *self.0 {
            Int(ref k) => k.to_string(),
            Bool(ref b) => b.to_string(),
            Sym(ref s) => s.to_string(),
            Keyword(ref s) => format!(":{}", s),
            Str(ref s) => {
                if readably {
                    format!("\"{}\"", escape_str(s))
                } else {
                    s.to_string()
                }
            },
            Nil => "nil".to_string(),
            List(ref l) => print_sequence(&l, "(", ")", readably),
            Vector(ref l) => print_sequence(&l, "[", "]", readably),
            HashMap(ref l) => {
                let l = l.iter()
                    .flat_map(|(k, v)| vec![MValue::string(k.to_string()), v.clone()])
                    .collect::<Vec<MValue>>();
                print_sequence(&l, "{", "}", readably)
            },
            Fun(_fun) => "#<function>".to_string(),
            Lambda(ref _fun) => "#<function>".to_string(),
        }
    }
}

fn escape_str(s: &str) -> String {
    s.chars().map(|c| {
	match c {
	    '"' => "\\\"".to_string(),
	    '\n' => "\\n".to_string(),
	    '\\' => "\\\\".to_string(),
	    _ => c.to_string(),
	}
    }).collect::<Vec<String>>().join("")
}

impl Display for MValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pr_str(false))
    }
}

#[derive(Debug)]
pub enum Error {
    ParseError,
    EvalError(String),
    ArgsError,
    NoSymbolFound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError => write!(f, "Parse error"),
            Error::EvalError(s) => write!(f, "Eval error: {}", s),
            Error::ArgsError => write!(f, "Args error"),
            Error::NoSymbolFound(s) => write!(f, "{} not found", s),
        }
    }
}

impl PartialEq for MalVal {
  fn eq(&self, other: &MalVal) -> bool {
    match (self, other) {
      (Nil, Nil) => true,
      (Bool(ref x), Bool(ref y)) => x == y,
      (Int(ref x), Int(ref y)) => x == y,
      (Str(ref x), Str(ref y)) => x == y,
      (Keyword(ref x), Keyword(ref y)) => x == y,
      (Sym(ref x), Sym(ref y)) => x == y,
      (List(ref x), List(ref y)) |
      (Vector(ref x), Vector(ref y)) |
      (List(ref x), Vector(ref y)) |
      (Vector(ref x), List(ref y)) => x == y,
      (HashMap(ref x), HashMap(ref y)) => x == y,
      _ => false,
    }
  }
}

fn print_sequence(seq: &[MValue], start: &str, end: &str, readably: bool) -> String {
    let seq: Vec<String> = seq
        .iter()
        .map(|v| v.pr_str(readably))
        .collect();

    format!("{}{}{}", start, seq.join(" "), end)
}

impl From<pom::Error> for Error {
    fn from(_error: pom::Error) -> Error {
        Error::ParseError
    }
}

