use std::collections::HashMap;
use crate::err_handle::RuntimeFailure;
use super::function::FunctionKind;

pub struct FunctionMap {
    map: HashMap<String, FunctionKind>
}

impl FunctionMap {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }
    pub fn get(&self, key: &str) -> Option<&FunctionKind> {
        self.map.get(key)
    }
    pub fn register_function(&mut self, name: String, function: FunctionKind) -> Result<(), RuntimeFailure> {
        match self.map.contains_key(name.as_str()) {
            true => Err(RuntimeFailure::DuplicateFunction(name)),
            false => {
                self.map.insert(name, function);
                Ok(())
            }
        }
    }
}
