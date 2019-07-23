use std::collections::HashMap;

use crate::types::*;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env(Rc<RefCell<PEnv>>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct PEnv {
    mappings: HashMap<String, MValue>,
    outer: Option<Env>, 
}

impl Env {
    pub fn new(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MValue>) -> Self {
        let mut mappings = HashMap::new();

        for (b, e) in binds.iter().zip(exprs.iter()) {
            mappings.insert(b.clone(), e.clone());
        }

        Env(Rc::new(RefCell::new(PEnv {
            mappings,
            outer
        })))
    }

    pub fn get(&self, key: &str) -> Option<MValue> {
        match self.0.borrow().mappings.get(key) {
            Some(v) => Some(v.clone()),
            None => match &self.0.borrow().outer {
                Some(env) => env.get(key),
                None => None,
            }
        }
    }

    pub fn set(&mut self, key: String, value: MValue) {
        self.0.borrow_mut().mappings.insert(key, value);
    }
}
