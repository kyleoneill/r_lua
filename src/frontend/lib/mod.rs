use super::function_map::FunctionMap;
use super::function::FunctionKind;
use super::function::InternalFunctionTypes;

pub fn register_std_lib(function_map: &mut FunctionMap) {
    let kind = FunctionKind::Internal(InternalFunctionTypes::TakesOneStr(Box::new(print_fn)));
    let _ = function_map.register_function("print".to_string(), kind);
}

fn print_fn(inp: &str) {
    println!("{}", inp);
}
