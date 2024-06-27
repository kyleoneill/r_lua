use crate::ast::Rule;

#[derive(Debug)]
pub struct CompileError {
    error_msg: String,
    line: usize,
    column: usize,
}

impl CompileError {
    pub fn new(error_str: &str, line_col: (usize, usize)) -> Self {
        Self {
            error_msg: error_str.to_string(),
            line: line_col.0,
            column: line_col.1,
        }
    }
    pub fn from_pest_error(e: pest::error::Error<Rule>) -> Self {
        let line_col = match e.line_col {
            pest::error::LineColLocation::Pos(pos) => pos,
            pest::error::LineColLocation::Span(start, _end) => start,
        };
        Self::new("Invalid Lua", line_col)
    }
    pub fn print_error(&self) {
        eprintln!(
            "Failed to compile Lua with error '{}' on line {} column {}",
            self.error_msg, self.line, self.column
        )
    }
}

#[derive(Debug)]
pub enum RuntimeFailure {
    VarNotFound(String, i32),
    BorrowError(String, i32),
}

impl RuntimeFailure {
    pub fn print_error(&self) {
        match self {
            RuntimeFailure::VarNotFound(var_name, line) => eprintln!("Error on line {}: Variable '{}' not found", line, var_name),
            RuntimeFailure::BorrowError(msg, line) => eprintln!("Error on line {}: {}", line, msg)
        }
    }
}
