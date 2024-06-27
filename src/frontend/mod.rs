mod variable_map;
mod data;

use data::Data;
use variable_map::VariableMap;

pub struct Context<'a> {
    pub current_line: i32,
    variable_map: &'a mut VariableMap,
}

impl <'a> Context<'a> {
    pub fn store_data(&mut self, var_name: String, data: Data) {
        self.variable_map.insert(var_name, data);
    }
}
