use std::collections::HashMap;
use std::ops::Deref;

use crate::types::*;

#[derive(Clone)]
pub struct Env {
    mappings: HashMap<String, MalVal>,
    outer: Box<Option<Env>>, 
}

impl Env {
    pub fn new(outer: Option<Env>) -> Self {
        Env {
            mappings: HashMap::new(),
            outer: Box::new(outer),
        }
    }

    pub fn get(&self, key: &str) -> Option<MalVal> {
        match self.mappings.get(key) {
            Some(v) => Some(v.clone()),
            None => match Deref::deref(&self.outer) {
                Some(env) => env.get(key),
                None => None,
            }
        }
    }

    pub fn set(&mut self, key: String, value: MalVal) {
        self.mappings.insert(key, value);
    }
}
