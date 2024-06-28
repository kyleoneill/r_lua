use crate::ast::lua_program::FunctionBody;
use crate::err_handle::RuntimeFailure;
use crate::frontend::Context;
use crate::frontend::data::DataKind;

pub enum FunctionKind {
    External(FunctionBody),
    Internal(InternalFunctionTypes)
}

// Todo: There _has_ to be a macro to take care of this for me
pub enum InternalFunctionTypes {
    TakesOneStr(Box<dyn Fn(&str)>)
}

impl InternalFunctionTypes {
    // TODO: there _has_ to be a macro to take care of this for me
    //       This is a very bad hack job to get the print function working for a hackathon demo
    pub fn run_function_one_string(&self, context: &Context, args: Vec<DataKind>) -> Result<(), RuntimeFailure> {
        match self {
            InternalFunctionTypes::TakesOneStr(func) => {
                if args.len() != 1 {
                    return Err(RuntimeFailure::BadFunctionArgs("Function should have 1 arg".to_string(), context.current_line));
                }
                match &args[0] {
                    DataKind::String(arg) => {
                        func(arg.as_str());
                        Ok(())
                    },
                    _ => return Err(RuntimeFailure::BadFunctionArgs("Function expected a string".to_string(), context.current_line))
                }
            },
            _ => Err(RuntimeFailure::InternalError("running a function that takes one string".to_string()))
        }
    }
}
