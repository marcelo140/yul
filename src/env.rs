use std::collections::HashMap;

use crate::types::*;
use std::rc::Rc;
use std::cell::RefCell;

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
            outer
        })))
    }

    pub fn with_binds(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MValue>) -> Self {
        let mut mappings = HashMap::new();

        for (i, b) in binds.iter().enumerate() {
            if binds[i] == "&" {
                let final_expr = exprs.into_iter().skip(i).collect();
                mappings.insert(binds[i+1].clone(), MValue::list(final_expr));
                break;
            } else {
                mappings.insert(b.clone(), exprs[i].clone());
            }
        }

        Env(Rc::new(RefCell::new(SharedEnv {
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

    pub fn set(&self, key: String, value: MValue) {
        self.0.borrow_mut().mappings.insert(key, value);
    }
}
