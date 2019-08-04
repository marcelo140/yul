use MalVal::*;

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::cell::RefCell;
use std::string::ToString;

use std::rc::Rc;
use crate::env::Env;

pub type FnExpr = fn(Vec<MValue>, Option<Env>) -> Result<MValue>;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct MValue(pub Rc<MalVal>, bool);

#[derive(Debug, Clone)]
pub enum MalVal {
    Int(i32),
    Bool(bool),
    List(Vec<MValue>),
    Vector(Vec<MValue>),
    HashMap(HashMap<(String, String), MValue>),
    Sym(String),
    Str(String),
    Keyword(String),
    Fun(FnExpr, Option<Env>), // bool = is macro?
    Atom(RefCell<MValue>),
    Lambda(MClosure),
    Nil,
}

impl Default for MValue {
    fn default() -> Self {
        MValue(Rc::new(MalVal::Nil), false)
    }
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

    pub fn apply(&self, exprs: Vec<MValue>) -> (MValue, Env) {
        let copy = self.clone();
        let env = Env::with_binds(Some(copy.env), copy.parameters, exprs);

        (copy.body, env)
    }
}

impl MValue {
    pub fn enum_key(&self) -> String {
        match *self.0 {
            Int(_) => "Int".to_string(),
            Bool(_) => "Bool".to_string(),
            Sym(_) => "Symbol".to_string(),
            Keyword(_) => "Keyword".to_string(),
            Atom(_) => "Atom".to_string(),
            Str(_) => "String".to_string(),
            Nil => "Nil".to_string(),
            List(_) => "List".to_string(),
            Vector(_) => "Vector".to_string(),
            HashMap(_) => "Hashmap".to_string(),
            Fun(_,_) | Lambda(_) => "Function".to_string(),
        }
    }

    pub fn integer(value: i32) -> MValue {
        MValue(Rc::new(MalVal::Int(value)), false)
    }

    pub fn bool(value: bool) -> MValue {
        MValue(Rc::new(MalVal::Bool(value)), false)
    }

    pub fn list(value: Vec<MValue>) -> MValue {
        MValue(Rc::new(MalVal::List(value)), false)
    }

    pub fn vector(value: Vec<MValue>) -> MValue {
        MValue(Rc::new(MalVal::Vector(value)), false)
    }

    pub fn from_hashmap(hm: HashMap<(String, String), MValue>) -> MValue {
        MValue(Rc::new(MalVal::HashMap(hm)), false)
    }

    pub fn hashmap(values: &mut Vec<MValue>) -> MValue {
        let v = MValue(Rc::new(MalVal::HashMap(HashMap::new())), false);
        v.hassoc(values).unwrap()
    }

    pub fn symbol(value: String) -> MValue {
        MValue(Rc::new(MalVal::Sym(value)), false)
    }

    pub fn string<T: Into<String>>(value: T) -> MValue {
        MValue(Rc::new(MalVal::Str(value.into())), false)
    }

    pub fn keyword(value: String) -> MValue {
        MValue(Rc::new(MalVal::Keyword(value)), false)
    }

    pub fn atom(value: MValue) -> MValue {
        MValue(Rc::new(MalVal::Atom(RefCell::new(value))), false)
    }

    pub fn function(value: FnExpr, env: Option<Env>) -> MValue {
        MValue(Rc::new(MalVal::Fun(value, env)), false)
    }

    pub fn lambda(env: Env, parameters: Vec<String>, body: MValue) -> MValue {
        MValue(Rc::new(MalVal::Lambda(MClosure {
            env,
            parameters,
            body,
        })), false)
    }

    pub fn nil() -> MValue {
        MValue(Rc::new(MalVal::Nil), false)
    }

    pub fn is_lambda(&self) -> bool {
        match *self.0 {
            MalVal::Lambda(_) => true,
            _ => false,
        }
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
            MalVal::List(ref l) => {
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

    // TODO: cast_to_int and cast_to_list are not consistent in term of borrowing
    pub fn cast_to_int(&self) -> Result<i32> {
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
            Lambda(ref closure) => Ok(closure.clone()),
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
            (v, key) if key == "Symbol" => Ok(MValue::symbol(v.to_string())),
            (v, key) if key == "Keyword" => Ok(MValue::keyword(v.to_string())),
            (v, key) if key == "String" => Ok(MValue::string(v.to_string())),
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

        Ok(MValue(Rc::new(MalVal::HashMap(hm)), false))
    }

    pub fn cast_to_list(self) -> Result<Vec<MValue>> {
        match *self.0 {
            MalVal::List(ref x) | MalVal::Vector(ref x) => Ok(x.to_vec()),
            _ => Err(Error::EvalError(format!("{} is not a list", self))),
        }
    }

    pub fn cast_to_hashmap(self) -> Result<HashMap<(String, String), MValue>> {
        match *self.0 {
            MalVal::HashMap(ref x) => Ok(x.clone()),
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
            List(ref l) => print_sequence(&l, "(", ")", readably),
            Vector(ref l) => print_sequence(&l, "[", "]", readably),
            HashMap(ref l) => {
                let l = l.iter()
                    .flat_map(|(k, v)| vec![MValue::reconstruct(k).unwrap(), v.clone()])
                    .collect::<Vec<MValue>>();
                print_sequence(&l, "{", "}", readably)
            },
            Fun(_,_) | Lambda(_) => "#<function>".to_string(),
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
            Error::ParseError(_) => MValue::string(self.to_string()),
            Error::EvalError(_) => MValue::string(self.to_string()),
            Error::ArgsError => MValue::string(self.to_string()),
            Error::NoSymbolFound(_) => MValue::string(self.to_string()),
            Error::IoError(_) => MValue::string(self.to_string()),
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
      (List(ref x), List(ref y)) |
      (Vector(ref x), Vector(ref y)) |
      (List(ref x), Vector(ref y)) |
      (Vector(ref x), List(ref y)) => x == y,
      (HashMap(ref x), HashMap(ref y)) => x == y,
      _ => false,
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
