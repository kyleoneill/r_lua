#![allow(dead_code)]
#![allow(unused_variables)]
// TODO: Remove this once everything is being used

mod variable_map;
mod data;
mod function;
mod lib;
mod function_map;

use crate::ast::lua_program::{Block, Expression, ExpressionList, FunctionCall, LuaProgram, ReturnStatement, Statement, Var, Args, Expr, NumberKind, PrefixExpression, BinaryOperator, Parameters, NameList, FunctionName};
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

pub fn enter_program(input: LuaProgram) -> Result<DataKind, RuntimeFailure> {
    let mut variable_map = VariableMap::new();
    let mut registered_functions = FunctionMap::new();
    lib::register_std_lib(&mut registered_functions);

    // Should ownership of var_map be passed here instead of a mut ref?
    let mut context = Context{ variable_map: &mut variable_map, registered_functions: &mut registered_functions, current_line: 0 };
    run_block(&mut context, input.block)
}

pub fn run_block(context: &mut Context, block: Block) -> Result<DataKind, RuntimeFailure> {
    for statement in block.statements {
        run_statement(context, statement)?;
    }
    if block.return_statement.is_some() {
        Ok(run_return_statement(context, block.return_statement.unwrap())?)
    }
    else {
        Ok(DataKind::Null)
    }
}

fn run_statement(context: &mut Context, statement: Statement) -> Result<DataKind, RuntimeFailure> {
    let statement_res = match statement {
        Statement::Empty => Ok(DataKind::Null),
        Statement::MultipleAssignment(var_list, expr_list) => {
            // TODO: Need to support multiple vars and multiple exprs
            let var_name = match &var_list.vars[0] {
                Var::NestedAccess(_) => todo!("nested access"),
                Var::VarName(var_name) => var_name.to_owned(),
                Var::TableAccess(_, _) => todo!("table access")
            };
            let resolved = resolve_expression(context, expr_list.expressions[0].clone())?;
            let new_data = Data::new(resolved);
            context.variable_map.insert(var_name, new_data);
            Ok(DataKind::Null)
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
            let resolved = resolve_expression(context, if_block.0)?;
            if resolved.is_true() {
                return run_block(context, if_block.1)
            }
            for elseif in elseif_block {
                let resolved = resolve_expression(context, elseif.0)?;
                if resolved.is_true() {
                    return run_block(context, elseif.1)
                }
            }
            match else_block {
                Some(some_else_block) => {
                    return run_block(context, some_else_block)
                },
                None => Ok(DataKind::Null)
            }
        },
        Statement::ForEach(var_name, var_set_expr, expr, maybe_expr, block) => {
            todo!("for foo=expr, expr, expr? do block end")
        },
        Statement::ForList(name_list, expr_list, block) => {
            todo!("for namelist in exprlist do block end")
        },
        Statement::Function(name, body) => {
            // TODO: Need to implement the rest of name
            let function_name = name.outer_name.clone();
            let function = FunctionKind::External(body);
            context.registered_functions.register_function(function_name, function)?;
            Ok(DataKind::Null)
        },
        Statement::LocalFunction(name, body) => {
            todo!("local function name body")
        }
    };
    statement_res
}

fn function_call(context: &mut Context, call: Box<FunctionCall>) -> Result<DataKind, RuntimeFailure> {
    match *call {
        FunctionCall::Static(static_function) => {
            let resolved_function_args = resolve_args(context, static_function.args)?;
            match static_function.prefix {
                Var::VarName(var_name) => {
                    match context.registered_functions.get(var_name.as_str()) {
                        Some(function) => {
                            match function {
                                FunctionKind::External(ref function_body) => {
                                    if let Some(params) = &function_body.parameters {
                                        let mut names: Vec<String> = Vec::new();
                                        match params {
                                            Parameters::Normal(name_list, exp) => {
                                                names = name_list.names.clone();
                                            }
                                            Parameters::Expanded(_) => todo!("expanded params")
                                        }
                                        if names.len() != resolved_function_args.len() {
                                            return Err(RuntimeFailure::InvalidArgs("Number of args passed are invalid".to_string(), context.current_line))
                                        }
                                        // This is not good, but I am out of time
                                        // Insert passed args into var map
                                        let mut counter = 0;
                                        for data in resolved_function_args {
                                            let to_insert = Data::new(data);
                                            context.variable_map.insert(names[counter].clone(), to_insert);
                                            counter += 1;
                                        }
                                        // Run the function
                                        let res = run_block(context, function_body.block.clone())?;
                                        // pop injected vars from var map
                                        for var_name in names {
                                            context.variable_map.remove(var_name.as_str())
                                        }
                                        Ok(res)
                                    }
                                    else {
                                        Ok(run_block(context, function_body.block.clone())?)
                                    }
                                },
                                FunctionKind::Internal(internal_function) => {
                                    // This is a hack, right now the only internal function is 'print'
                                    internal_function.run_function_one_string(context, resolved_function_args)?;
                                    Ok(DataKind::Null)
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
        },
        FunctionCall::PassedExpression(expression) => {
            Ok(resolve_expression(context, expression)?)
        }
    }
}

fn run_return_statement(context: &mut Context, return_statement: ReturnStatement) -> Result<DataKind, RuntimeFailure> {
    match return_statement.expression_list {
        Some(expr_list) => {
            let mut expressions = expr_list.expressions;
            if expressions.len() == 0 {
                return Ok(DataKind::Null)
            }
            // TODO: Need to support returning a list of expressions
            if expressions.len() != 1 {
                return Err(RuntimeFailure::InternalError("resolving return statement".to_string()))
            }
            Ok(resolve_expression(context, expressions.pop().unwrap())?)
        },
        None => Ok(DataKind::Null)
    }
}

fn resolve_args(context: &mut Context, args: Args) -> Result<Vec<DataKind>, RuntimeFailure> {
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

fn resolve_expr_list(context: &mut Context, expression_list: ExpressionList) -> Result<Vec<DataKind>, RuntimeFailure> {
    let mut resolved_data = Vec::new();
    for expression in expression_list.expressions {
        let resolved = resolve_expression(context, expression)?;
        resolved_data.push(resolved);
    }
    Ok(resolved_data)
}

fn resolve_expression(context: &mut Context, expression: Expression) -> Result<DataKind, RuntimeFailure> {
    match expression {
        Expression::Expr(expr) => resolve_expr(context, expr),
        Expression::Binary(binary_op, expr, second_expr) => {
            let lhs = resolve_expr(context, expr)?;
            let rhs = resolve_expr(context, second_expr)?;
            match binary_op {
                BinaryOperator::MathOperator(math_op) => Ok(DataKind::Number(lhs.math_binary_op(&rhs, math_op, context)?)),
                BinaryOperator::BitwiseOperator(bitwise_op) => todo!("bitwise binary op"),
                BinaryOperator::Concat => {
                    let concat = format!("{}{}", lhs.to_string(), rhs.to_string());
                    Ok(DataKind::String(concat))
                },
                BinaryOperator::BooleanOperator(boolean_op) => Ok(DataKind::Bool(lhs.boolean_binary_op(&rhs, boolean_op, context)?))
            }
        }
    }
}

fn resolve_expr(context: &mut Context, expr: Expr) -> Result<DataKind, RuntimeFailure> {
    match expr {
        Expr::Nil => Ok(DataKind::Null),
        Expr::Boolean(bool) => Ok(DataKind::Bool(bool)),
        Expr::Numerical(number_kind) => Ok(resolve_number_kind(number_kind)),
        Expr::LiteralString(literal_string) => Ok(DataKind::String(literal_string)),
        Expr::Expansion(expansion) => todo!("expr expansion"),
        Expr::FunctionDef(function_body) => todo!("expr anonymous function def"),
        Expr::Prefix(prefix) => resolve_prefix(context, prefix),
        Expr::Unary(unary_op, inner_expr) => todo!("expr unary"),
    }
}

fn resolve_prefix(context: &mut Context, prefix: Box<PrefixExpression>) -> Result<DataKind, RuntimeFailure> {
    match *prefix {
        PrefixExpression::Var(var) => resolve_var(context, var),
        PrefixExpression::FunctionCall(fc) => function_call(context, fc),
        PrefixExpression::Expression(expr) => resolve_expression(context, expr)
    }
}

fn resolve_var(context: &Context, var: Var) -> Result<DataKind, RuntimeFailure> {
    match var {
        Var::VarName(var_name) => {
            let binding = context.variable_map.get(context, var_name.as_str())?.borrow(context)?;
            Ok(binding.clone())
        },
        Var::NestedAccess(nested_accessors) => {
            todo!("Nested var resolution")
        },
        Var::TableAccess(table_name, expression) => {
            todo!("Table access")
        }
    }
}

fn resolve_number_kind(number_kind: NumberKind) -> DataKind {
    match number_kind {
        NumberKind::Int(int) => DataKind::Number(data::NumberKind::Integer(int)),
        NumberKind::Float(float) => DataKind::Number(data::NumberKind::Float(float)),
    }
}
