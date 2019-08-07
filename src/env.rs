use std::collections::HashMap;

use crate::types::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Env(Rc<RefCell<SharedEnv>>);

#[derive(Debug, Clone)]
struct SharedEnv {
    mappings: HashMap<String, MValue>,
    outer: Option<Env>,
}

impl Env {
    pub fn new(outer: Option<Env>) -> Self {
        Env(Rc::new(RefCell::new(SharedEnv {
            mappings: HashMap::new(),
            outer,
        })))
    }

    pub fn with_binds(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MValue>) -> Result<Self> {
        let mut mappings = HashMap::new();

        for (b, e) in itertools::zip(&binds, &exprs) {
            if b == "&" {
                break;
            }

            mappings.insert(b.clone(), e.clone());
        }

        let consumed = mappings.len();
        if binds.len() > consumed && binds[consumed] == "&" {
            if binds.len() == consumed + 1 {
                return Err(Error::EvalError("No bind provided for variadic".to_string()));
            }

            let rest = exprs.iter().skip(consumed).cloned().collect();
            mappings.insert(binds[consumed + 1].clone(), MValue::list(rest));
        }

        Ok(Env(Rc::new(RefCell::new(SharedEnv { mappings, outer }))))
    }

    pub fn get(&self, key: &str) -> Option<MValue> {
        match self.0.borrow().mappings.get(key) {
            Some(v) => Some(v.clone()),
            None => match &self.0.borrow().outer {
                Some(env) => env.get(key),
                None => None,
            },
        }
    }

    pub fn set<T: ToString>(&self, key: T, value: MValue) {
        self.0.borrow_mut().mappings.insert(key.to_string(), value);
    }
}
