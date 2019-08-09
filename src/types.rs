use MalVal::*;

use std::collections::HashMap;
use std::fmt::{self, Display, Debug};
use std::cell::RefCell;
use std::string::ToString;

use std::rc::Rc;
use crate::env::Env;

pub type FnExpr = fn(Vec<MValue>, Option<Env>) -> Result<MValue>;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct MValue(pub Rc<MalVal>, bool);

#[derive(Clone)]
pub enum MalVal {
    Int(i64),
    Bool(bool),
    List(Vec<MValue>, MValue),
    Vector(Vec<MValue>, MValue),
    HashMap(HashMap<(String, String), MValue>, MValue),
    Sym(String),
    Str(String),
    Keyword(String),
    Fun(FnExpr, Option<Env>, MValue),
    Atom(RefCell<MValue>),
    Lambda(MClosure, MValue),
    Nil,
}

#[derive(Debug, Clone)]
pub struct MClosure {
    env: Env,
    parameters: Vec<String>,
    body: MValue,
}

impl MClosure {
    pub fn new(env: Env, parameters: Vec<String>, body: MValue) -> Self {
        MClosure {
            env,
            parameters,
            body,
        }
    }

    pub fn apply(&self, exprs: Vec<MValue>) -> Result<(MValue, Env)> {
        let copy = self.clone();
        let env = Env::with_binds(Some(copy.env), copy.parameters, exprs)?;

        Ok((copy.body, env))
    }
}

impl MValue {
    pub fn meta(&self) -> Result<MValue> {
        match *self.0 {
            List(_, ref v) | Vector(_, ref v) | HashMap(_, ref v)
                | Fun(_,_, ref v) | Lambda(_, ref v) => Ok(v.clone()),
            _ => Err(Error::EvalError(format!("{} has no metadata", self))),
        }
    }

    pub fn with_meta(&self, meta: MValue) -> Result<MValue> {
        let r = match *self.0 {
            List(ref v, _) => List(v.clone(), meta),
            Vector(ref v, _) => Vector(v.clone(), meta), 
            HashMap(ref v, _) => HashMap(v.clone(), meta),
            Fun(f, ref env, _) => Fun(f, env.clone(), meta),
            Lambda(ref v, _) => Lambda(v.clone(), meta),
            _ => return Err(Error::EvalError(format!("{} can't hold metadata", self))),
        };

        Ok(MValue(Rc::new(r), self.1))
    }

    pub fn enum_key(&self) -> String {
        match *self.0 {
            Int(_) => "Int".to_string(),
            Bool(_) => "Bool".to_string(),
            Sym(_) => "Symbol".to_string(),
            Keyword(_) => "Keyword".to_string(),
            Atom(_) => "Atom".to_string(),
            Str(_) => "String".to_string(),
            Nil => "Nil".to_string(),
            List(_,_) => "List".to_string(),
            Vector(_,_) => "Vector".to_string(),
            HashMap(_,_) => "Hashmap".to_string(),
            Fun(_,_,_) | Lambda(_,_) => "Function".to_string(),
        }
    }

    pub fn integer(value: i64) -> MValue {
        MValue(Rc::new(MalVal::Int(value)), false)
    }

    pub fn bool(value: bool) -> MValue {
        MValue(Rc::new(MalVal::Bool(value)), false)
    }

    pub fn list(value: Vec<MValue>) -> MValue {
        MValue(Rc::new(MalVal::List(value, MValue::nil())), false)
    }

    pub fn vector(value: Vec<MValue>) -> MValue {
        MValue(Rc::new(MalVal::Vector(value, MValue::nil())), false)
    }

    pub fn from_hashmap(hm: HashMap<(String, String), MValue>) -> MValue {
        MValue(Rc::new(MalVal::HashMap(hm, MValue::nil())), false)
    }

    pub fn hashmap(values: &mut Vec<MValue>) -> MValue {
        let v = MValue(Rc::new(MalVal::HashMap(HashMap::new(), MValue::nil())), false);
        v.hassoc(values).unwrap()
    }

    pub fn symbol<T: ToString>(value: T) -> MValue {
        MValue(Rc::new(MalVal::Sym(value.to_string())), false)
    }

    pub fn string<T: ToString>(value: T) -> MValue {
        MValue(Rc::new(MalVal::Str(value.to_string())), false)
    }

    pub fn keyword<T: ToString>(value: T) -> MValue {
        MValue(Rc::new(MalVal::Keyword(value.to_string())), false)
    }

    pub fn atom(value: MValue) -> MValue {
        MValue(Rc::new(MalVal::Atom(RefCell::new(value))), false)
    }

    pub fn function(value: FnExpr, env: Option<Env>) -> MValue {
        MValue(Rc::new(MalVal::Fun(value, env, MValue::nil())), false)
    }

    pub fn lambda(env: Env, parameters: Vec<String>, body: MValue) -> MValue {
        MValue(Rc::new(MalVal::Lambda(MClosure {
            env,
            parameters,
            body,
        }, MValue::nil())), false)
    }

    pub fn nil() -> MValue {
        MValue(Rc::new(MalVal::Nil), false)
    }

    pub fn is_lambda(&self) -> bool {
        match *self.0 {
            MalVal::Lambda(_,_) => true,
            _ => false,
        }
    }

    pub fn is_builtin(&self) -> bool {
        match *self.0 {
            MalVal::Fun(_,_,_) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        match *self.0 {
            MalVal::List(_,_) => true,
            _ => false,
        }
    }

    pub fn is_hashmap(&self) -> bool {
        match *self.0 {
            MalVal::HashMap(_,_) => true,
            _ => false,
        }
    }

    pub fn is_vector(&self) -> bool {
        match *self.0 {
            MalVal::Vector(_,_) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match *self.0 {
            MalVal::Nil => true,
            _ => false,
        }
    }

    pub fn is_macro(&self) -> bool {
        self.1
    }

    pub fn is_macro_call(&self, env: &Env) -> bool {
        match *self.0 {
            MalVal::List(ref l, _) => {
                if l.is_empty() {
                    return false;
                }

                match *l[0].0 {
                    MalVal::Sym(ref symbol) => {
                        let mapping = env.get(&symbol);
                        mapping
                            .map(|v| v.is_lambda() && v.is_macro())
                            .unwrap_or(false)
                    },
                    _ => false,
                }
            },
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match *self.0 {
            MalVal::Sym(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self.0 {
            MalVal::Int(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match *self.0 {
            MalVal::Str(_) => true,
            _ => false,
        }
    }

    pub fn is_keyword(&self) -> bool {
        match *self.0 {
            MalVal::Keyword(_) => true,
            _ => false,
        }
    }

    pub fn is_atom(&self) -> bool {
        match *self.0 {
            MalVal::Atom(_) => true,
            _ => false,
        }
    }

    pub fn set_macro(&mut self) {
        self.1 = true;
    }

    pub fn cast_to_int(&self) -> Result<i64> {
        match *self.0 {
            MalVal::Int(x) => Ok(x),
            _ => Err(Error::EvalError(format!("{} is not a list!", self))),
        }
    }

    pub fn atom_deref(&self) -> Result<MValue> {
        match *self.0 {
            MalVal::Atom(ref x) => Ok(x.borrow().clone()),
            _ => Err(Error::EvalError(format!("{} is not an atom!", self))),
        }
    }

    pub fn atom_reset(&self, new: MValue) -> Result<MValue> {
        match *self.0 {
            MalVal::Atom(ref x) => {
                x.replace(new.clone());
                Ok(new)
            },
            _ => Err(Error::EvalError(format!("{} is not an atom!", self))),
        }
    }

    pub fn cast_to_lambda(&self) -> Result<MClosure> {
        match *self.0 {
            Lambda(ref closure, _) => Ok(closure.clone()),
            _ => Err(Error::EvalError(format!("{} is not a closure", self))),
        }
    }

    pub fn cast_to_string(&self) -> Result<String> {
        match *self.0 {
            Sym(ref x) | Keyword(ref x) | Str(ref x) => Ok(x.clone()),
            _ => Err(Error::EvalError(format!("{} is not a string", self))),
        }
    }

    pub fn cast_to_bool(&self) -> bool {
        match *self.0 {
            Bool(true) => true,
            _ => false,
        }
    }

    pub fn reconstruct(value: &(String, String)) -> Result<MValue> {
        match value {
            (v, key) if key == "Symbol" => Ok(MValue::symbol(v)),
            (v, key) if key == "Keyword" => Ok(MValue::keyword(v)),
            (v, key) if key == "String" => Ok(MValue::string(v)),
            x => Err(Error::EvalError(format!("Can't reconstruct {:?}", x))),
        }
    }

    pub fn hassoc(&self, list: &mut Vec<MValue>) -> Result<MValue> {
        let mut hm = self.clone().cast_to_hashmap()?;

        while !list.is_empty() {
            let v = list.pop().ok_or_else(|| Error::ParseError(
                    "Could not extract value for hashmap".to_string()))?;

            match list.pop() {
                Some(ref k) if k.is_symbol() || k.is_string() || k.is_keyword() =>
                    hm.insert((k.cast_to_string()?, k.enum_key()), v),
                r => return Err(Error::ParseError(
                        format!("Could not extract key for hashmap: {:?}", r))),
            };
        }

        Ok(
            MValue(
                Rc::new(
                    MalVal::HashMap(hm, MValue::nil())
                ),
                false
            )
        )
    }

    pub fn cast_to_list(&self) -> Result<Vec<MValue>> {
        match *self.0 {
            List(ref x, _) | Vector(ref x, _) => Ok(x.to_vec()),
            Str(ref s) => Ok(s.chars().map(MValue::string).collect()),
            _ => Err(Error::EvalError(format!("{} is not a list", self))),
        }
    }

    pub fn cast_to_hashmap(&self) -> Result<HashMap<(String, String), MValue>> {
        match *self.0 {
            MalVal::HashMap(ref x, _) => Ok(x.clone()),
            _ => Err(Error::EvalError(format!("{} is not a hashmap", self))),
        }
    }

    pub fn pr_str(&self, readably: bool) -> String {
        match *self.0 {
            Int(ref k) => k.to_string(),
            Bool(ref b) => b.to_string(),
            Sym(ref s) => s.to_string(),
            Keyword(ref s) => format!(":{}", s),
            Atom(ref v) => format!("(atom {})", v.borrow()),
            Str(ref s) => {
                if readably {
                    format!("\"{}\"", escape_str(s))
                } else {
                    s.to_string()
                }
            },
            Nil => "nil".to_string(),
            List(ref l, _) => print_sequence(&l, "(", ")", readably),
            Vector(ref l, _) => print_sequence(&l, "[", "]", readably),
            HashMap(ref l, _) => {
                let l = l.iter()
                    .flat_map(|(k, v)| vec![MValue::reconstruct(k).unwrap(), v.clone()])
                    .collect::<Vec<MValue>>();
                print_sequence(&l, "{", "}", readably)
            },
            Fun(_,_,_) | Lambda(_,_) => "#<function>".to_string(),
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
    Throw(MValue),
    ParseError(String),
    EvalError(String),
    ArgsError,
    NoSymbolFound(String),
    IoError(String),
}

impl Error {
    pub fn catch(&self) -> MValue {
        match self {
            Error::Throw(ref v) => v.clone(),
            Error::ParseError(_) => MValue::string(self),
            Error::EvalError(_) => MValue::string(self),
            Error::ArgsError => MValue::string(self),
            Error::NoSymbolFound(_) => MValue::string(self),
            Error::IoError(_) => MValue::string(self),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Throw(s) => write!(f, "Exception: {}", s),
            Error::ParseError(s) => write!(f, "Parse error: {}", s),
            Error::EvalError(s) => write!(f, "Eval error: {}", s),
            Error::ArgsError => write!(f, "Args error"),
            Error::NoSymbolFound(s) => write!(f, "\'{}\' not found", s),
            Error::IoError(s) => write!(f, "IO Error: {}", s),
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
      (List(ref x, _), List(ref y, _)) |
      (Vector(ref x, _), Vector(ref y, _)) |
      (List(ref x, _), Vector(ref y, _)) |
      (Vector(ref x, _), List(ref y, _)) => x == y,
      (HashMap(ref x, _), HashMap(ref y, _)) => x == y,
      _ => false,
    }
  }
}

impl Debug for MalVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Int(ref k) => write!(f, "{:?}", k),
            Bool(ref b) => write!(f, "{:?}", b),
            Sym(ref s) => write!(f, "{:?}", s),
            Keyword(ref s) => write!(f, "{:?}", s),
            Atom(ref v) => write!(f, "(atom {:?})", v),
            Str(ref s) => write!(f, "{:?}", s),
            Nil => write!(f, "nil"),
            List(ref l, _) => write!(f, "{}", print_sequence(&l, "(", ")", true)),
            Vector(ref l, _) => write!(f, "{}", print_sequence(&l, "[", "]", true)),
            HashMap(ref l, _) => {
                let l = l.iter()
                    .flat_map(|(k, v)| vec![MValue::reconstruct(k).unwrap(), v.clone()])
                    .collect::<Vec<MValue>>();
                write!(f, "{}", print_sequence(&l, "{", "}", true))
            },
            Fun(_,_,_) | Lambda(_,_) => write!(f, "#<function>"),
        }
    }
}

impl From<pom::Error> for Error {
    fn from(error: pom::Error) -> Error {
        Error::ParseError(error.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::ParseError(error.to_string())
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(error: std::num::TryFromIntError) -> Error {
        Error::EvalError(error.to_string())
    }
}
