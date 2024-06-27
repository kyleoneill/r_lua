#![allow(dead_code)]
#![allow(unused_variables)]
// TODO: Remove this once everything is being used

mod variable_map;
mod data;
mod function;
mod lib;
mod function_map;

use crate::ast::lua_program::{Block, Expression, ExpressionList, FunctionCall, LuaProgram, ReturnStatement, Statement, Var, Args, Expr, NumberKind};
use crate::err_handle::RuntimeFailure;

use data::Data;
use variable_map::VariableMap;
use function_map::FunctionMap;
use function::FunctionKind;
use crate::frontend::data::DataKind;

pub struct Context<'a> {
    pub current_line: i32, // Need to find a way to make this actually work. Likely need to just figure out how to store span info from Pest when generating the AST
    variable_map: &'a mut VariableMap,
    registered_functions: &'a mut FunctionMap
}

impl <'a> Context<'a> {
    pub fn store_data(&mut self, var_name: String, data: Data) {
        self.variable_map.insert(var_name, data);
    }
}

pub fn enter_program(input: LuaProgram) -> Result<(), RuntimeFailure> {
    let mut variable_map = VariableMap::new();
    let mut registered_functions = FunctionMap::new();
    lib::register_std_lib(&mut registered_functions);

    // Should ownership of var_map be passed here instead of a mut ref?
    let context = Context{ variable_map: &mut variable_map, registered_functions: &mut registered_functions, current_line: 0 };
    run_block(&context, input.block)
}

pub fn run_block(context: &Context, block: Block) -> Result<(), RuntimeFailure> {
    for statement in block.statements {
        run_statement(context, statement)?;
    }
    if block.return_statement.is_some() {
        run_return_statement(context, block.return_statement.unwrap())?;
    }
    Ok(())
}

fn run_statement(context: &Context, statement: Statement) -> Result<(), RuntimeFailure> {
    let statement_res = match statement {
        Statement::Empty => Ok(()),
        Statement::MultipleAssignment(var_list, expr_list) => {
            todo!("Variable assignment")
        },
        Statement::FunctionCall(call) => function_call(context, call),
        Statement::Label(label) => todo!("Label"),
        Statement::Break => todo!("Break"),
        Statement::GoTo(goto) => todo!("goto"),
        Statement::DoBlockEnd(block) => run_block(context, block),
        Statement::WhileExprDoBlockEnd(expr, block) => {
            todo!("while expr do block")
        },
        Statement::RepeatBlockUntilExpr(block, expr) => {
            todo!("repeat block until expr")
        },
        Statement::IfBlock(if_block, elseif_block, else_block) => {
            todo!("if block")
        },
        Statement::ForEach(var_name, var_set_expr, expr, maybe_expr, block) => {
            todo!("for foo=expr, expr, expr? do block end")
        },
        Statement::ForList(name_list, expr_list, block) => {
            todo!("for namelist in exprlist do block end")
        },
        Statement::Function(name, body) => {
            todo!("function name body")
        },
        Statement::LocalFunction(name, body) => {
            todo!("local function name body")
        }
    };
    statement_res
}

fn function_call(context: &Context, call: Box<FunctionCall>) -> Result<(), RuntimeFailure> {
    match *call {
        FunctionCall::Static(static_function) => {
            let resolved_function_args = resolve_args(context, static_function.args)?;
            match static_function.prefix {
                Var::VarName(var_name) => {
                    match context.registered_functions.get(var_name.as_str()) {
                        Some(function) => {
                            match function {
                                FunctionKind::External(block) => {
                                    todo!("Run block for external function")
                                },
                                FunctionKind::Internal(internal_function) => {
                                    Ok(internal_function.run_function_one_string(context, resolved_function_args)?)
                                }
                            }
                        },
                        None => return Err(RuntimeFailure::FuncNotFound(var_name, context.current_line))
                    }
                },
                _ => todo!("Call function by nested access")
            }
        },
        FunctionCall::SelfRef(self_function) => {
            todo!("run self function")
        }
    }
}

fn run_return_statement(context: &Context, return_statement: ReturnStatement) -> Result<(), RuntimeFailure> {
    todo!("return statement");
    Ok(())
}

fn resolve_args(context: &Context, args: Args) -> Result<Vec<DataKind>, RuntimeFailure> {
    match args {
        Args::ExpressionList(maybe_expr_list) => {
            match maybe_expr_list {
                Some(expr_list) => {
                    resolve_expr_list(context, expr_list)
                },
                None => Ok(Vec::new())
            }
        }
    }
}

fn resolve_expr_list(context: &Context, expression_list: ExpressionList) -> Result<Vec<DataKind>, RuntimeFailure> {
    let mut resolved_data = Vec::new();
    for expression in expression_list.expressions {
        let resolved = resolve_expression(context, expression)?;
        resolved_data.push(resolved);
    }
    Ok(resolved_data)
}

fn resolve_expression(context: &Context, expression: Expression) -> Result<DataKind, RuntimeFailure> {
    match expression {
        Expression::Expr(expr) => {
            match expr {
                Expr::Nil => Ok(DataKind::Null),
                Expr::Boolean(bool) => Ok(DataKind::Bool(bool)),
                Expr::Numerical(number_kind) => Ok(resolve_number_kind(number_kind)),
                Expr::LiteralString(literal_string) => Ok(DataKind::String(literal_string)),
                Expr::Expansion(expansion) => todo!("expr expansion"),
                Expr::FunctionDef(function_body) => todo!("expr function def"),
                Expr::Prefix(prefix) => todo!("expr prefix"),
                Expr::Unary(unary_op, inner_expr) => todo!("expr unary"),
            }
        },
        Expression::Binary(binary_op, expr, second_expr) => {
            todo!("binary expression")
        }
    }
}

fn resolve_number_kind(number_kind: NumberKind) -> DataKind {
    match number_kind {
        NumberKind::Int(int) => DataKind::Number(data::NumberKind::Integer(int)),
        NumberKind::Float(float) => DataKind::Number(data::NumberKind::Float(float)),
    }
}
