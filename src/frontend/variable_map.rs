use std::cell::RefMut;
use std::collections::HashMap;

use crate::err_handle::RuntimeFailure;
use crate::frontend::Context;
use crate::frontend::data::{Data, DataKind};

pub struct VariableMap {
    map: HashMap<String, Data>
}

impl VariableMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    pub fn get(&self, context: &Context, key: &str) -> Result<&Data, RuntimeFailure> {
        match self.map.get(key) {
            Some(val) => Ok(val),
            None => Err(RuntimeFailure::VarNotFound(key.to_owned(), context.current_line))
        }
    }

    pub fn get_mut(&self, context: &Context, key: &str) -> Result<RefMut<DataKind>, RuntimeFailure> {
        match self.map.get(key) {
            Some(val) => val.borrow_mut(context),
            None => Err(RuntimeFailure::VarNotFound(key.to_owned(), context.current_line))
        }
    }

    pub fn insert(&mut self, key: String, value: Data) {
        self.map.insert(key, value);
    }

    pub fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }
}
