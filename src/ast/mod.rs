use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::err_handle::CompileError;

mod lua_program;

// TODO: Check names for reserved keyword usage
#[allow(dead_code)]
const RESERVED_KEYWORDS: [&str; 22] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LuaTokenPairs;

pub fn parse_lua_program(input: &str) -> Result<lua_program::LuaProgram, CompileError> {
    match LuaTokenPairs::parse(Rule::Main, input) {
        Ok(mut parsed) => {
            let pair = parsed.next().expect("Lua program must begin with a block");
            let block = parse_block_pair(pair)?;
            Ok(lua_program::LuaProgram { block })
        }
        Err(e) => Err(CompileError::from_pest_error(e)),
    }
}

fn parse_block_pair(block_pair: Pair<Rule>) -> Result<lua_program::Block, CompileError> {
    if block_pair.as_rule() != Rule::Block {
        panic!("Expected pair to be a block when it was not")
    }
    let inner = block_pair.into_inner();
    let mut statements: Vec<lua_program::Statement> = Vec::new();
    let mut return_statement = None;
    for next_token in inner {
        match next_token.as_rule() {
            Rule::Stat => {
                let statement = parse_statement_pair(next_token)?;
                statements.push(statement);
            }
            Rule::RetStat => {
                return_statement = Some(parse_return_statement_pair(next_token)?);
            }
            _ => panic!("Block token must only contain Stat and RetStat inner tokens"),
        }
    }
    Ok(lua_program::Block {
        statements,
        return_statement,
    })
}

fn parse_statement_pair(
    statement_pair: Pair<Rule>,
) -> Result<lua_program::Statement, CompileError> {
    if statement_pair.as_rule() != Rule::Stat {
        panic!("Expected pair to be a statement when it was not")
    }
    let mut inner = statement_pair.into_inner();
    let next = inner.next().expect("Rule::Stat must have an inner value");
    match next.as_rule() {
        Rule::EmptyStatement => Ok(lua_program::Statement::Empty),
        Rule::ListAssignment => {
            let mut list_assign_inner = next.into_inner();
            let var_list_pair = list_assign_inner
                .next()
                .expect("ListAssignment must contain two inner values");
            let exp_list_pair = list_assign_inner
                .next()
                .expect("ListAssignment must contain two inner values");
            let var_list = parse_var_list_pair(var_list_pair)?;
            let exp_list = parse_expression_list_pair(exp_list_pair)?;
            Ok(lua_program::Statement::MultipleAssignment(
                var_list, exp_list,
            ))
        }
        Rule::FunctionCall => {
            let mut function_call_inner = next.into_inner();
            let fc_name_pair = function_call_inner
                .next()
                .expect("Rule::FunctionCall must have a first inner which is a name");
            let args_pair = function_call_inner
                .next()
                .expect("Rule::FunctionCall must have a second inner which is an args");
            let args = parse_args_pair(args_pair)?;
            Ok(lua_program::Statement::FunctionCall(
                fc_name_pair.as_str().to_owned(),
                args,
            ))
        }
        Rule::Label => Ok(lua_program::Statement::Label(next.as_str().to_owned())),
        Rule::BreakStatement => Ok(lua_program::Statement::Break),
        Rule::GotoStatement => Ok(lua_program::Statement::GoTo(next.as_str().to_owned())),
        Rule::DoBlockEndStatement => {
            let mut do_block_inner = next.into_inner();
            let block_pair = do_block_inner
                .next()
                .expect("Rule::DoBlockEndStatement must contain an inner block");
            let block = parse_block_pair(block_pair)?;
            Ok(lua_program::Statement::DoBlockEnd(block))
        }
        Rule::WhileExprDoBlockStatement => {
            let mut while_expr_do_inner = next.into_inner();
            let expression_pair = while_expr_do_inner
                .next()
                .expect("Rule::WhileExprDoBlockStatement must contain an inner expression");
            let block_pair = while_expr_do_inner
                .next()
                .expect("Rule::WhileExprDoBlockStatement must contain an inner block");
            let expression = parse_expression_pair(expression_pair)?;
            let block = parse_block_pair(block_pair)?;
            Ok(lua_program::Statement::WhileExprDoBlockEnd(
                expression, block,
            ))
        }
        Rule::RepeatBlockUntilStatement => {
            let mut repeat_block_inner = next.into_inner();
            let block_pair = repeat_block_inner
                .next()
                .expect("Rule::RepeatBlockUntilStatement must contain an inner block");
            let expression_pair = repeat_block_inner
                .next()
                .expect("Rule::RepeatBlockUntilStatement must contain an inner expression");
            let block = parse_block_pair(block_pair)?;
            let expression = parse_expression_pair(expression_pair)?;
            Ok(lua_program::Statement::RepeatBlockUntilExpr(
                block, expression,
            ))
        }
        Rule::IfStatement => parse_if_statement_pair(next),
        Rule::ForEachStatement => {
            let mut for_each_inner = next.into_inner();
            let name_pair = for_each_inner
                .next()
                .expect("Rule::ForEachStatement must have an inner name");
            let name = name_pair.as_str().to_owned();
            let expression_pair = for_each_inner
                .next()
                .expect("Rule::ForEachStatement must have an inner expression");
            let first_expression = parse_expression_pair(expression_pair)?;
            let second_expression_pair = for_each_inner
                .next()
                .expect("Rule::ForEachStatement must have a second inner expression");
            let second_expression = parse_expression_pair(second_expression_pair)?;
            let optional_expression = match for_each_inner.peek().is_some()
                && for_each_inner.peek().unwrap().as_rule() == Rule::Expression
            {
                true => {
                    let optional_expression_pair = for_each_inner.next().unwrap();
                    Some(parse_expression_pair(optional_expression_pair)?)
                }
                false => None,
            };
            let block_pair = for_each_inner
                .next()
                .expect("Rule::ForEachStatement must end with a block pair");
            let block = parse_block_pair(block_pair)?;
            Ok(lua_program::Statement::ForEach(
                name,
                first_expression,
                second_expression,
                optional_expression,
                block,
            ))
        }
        Rule::ForListStatement => {
            let mut for_list_inner = next.into_inner();
            let name_list_pair = for_list_inner
                .next()
                .expect("Rule::ForListStatement must have a name list pair");
            let name_list = parse_name_list_pair(name_list_pair)?;
            let expr_list_pair = for_list_inner
                .next()
                .expect("Rule::ForListStatement must have an expression list pair");
            let expr_list = parse_expression_list_pair(expr_list_pair)?;
            let block_pair = for_list_inner
                .next()
                .expect("Rule::ForListStatement must have a block");
            let block = parse_block_pair(block_pair)?;
            Ok(lua_program::Statement::ForList(name_list, expr_list, block))
        }
        Rule::FunctionStatement => {
            let mut function_inner = next.into_inner();
            let function_name_pair = function_inner
                .next()
                .expect("Rule::FunctionStatement must have a FunctionName pair");
            let function_name = parse_function_name_pair(function_name_pair)?;
            let function_body_pair = function_inner
                .next()
                .expect("Rule::FunctionStatement must have a FunctionBody pair");
            let function_body = parse_function_body(function_body_pair)?;
            Ok(lua_program::Statement::Function(
                function_name,
                function_body,
            ))
        }
        Rule::LocalFunctionStatement => {
            let mut local_function_inner = next.into_inner();
            let function_name_pair = local_function_inner
                .next()
                .expect("Rule::LocalFunctionStatement must have a function name");
            let function_body_pair = local_function_inner
                .next()
                .expect("Rule::LocalFunctionStatement must have a function body");
            let function_body = parse_function_body(function_body_pair)?;
            Ok(lua_program::Statement::LocalFunction(
                function_name_pair.as_str().to_owned(),
                function_body,
            ))
        }
        Rule::LocalAttributeNameListStatement => todo!(),
        _ => panic!("Matched on an undefined Stat inner"),
    }
}

fn parse_if_statement_pair(
    if_statement_pair: Pair<Rule>,
) -> Result<lua_program::Statement, CompileError> {
    if if_statement_pair.as_rule() != Rule::IfStatement {
        panic!("Expected pair to be an if statement when it was not")
    }
    // if
    let mut if_inner = if_statement_pair.into_inner();
    let expr_pair = if_inner
        .next()
        .expect("Rule::IfStatement must have an expression pair");
    let expr = parse_expression_pair(expr_pair)?;
    let block_pair = if_inner
        .next()
        .expect("Rule::IfStatement must have a block pair");
    let block = parse_block_pair(block_pair)?;

    // elseif
    let mut elseif: Vec<(lua_program::Expression, lua_program::Block)> = Vec::new();
    while if_inner.peek().is_some() && if_inner.peek().unwrap().as_rule() == Rule::Expression {
        let elseif_expr_pair = if_inner.next().unwrap();
        let elseif_block_pair = if_inner
            .next()
            .expect("Rule::IfStatement must always have a block after an optional expression");
        let elseif_expr = parse_expression_pair(elseif_expr_pair)?;
        let elseif_block = parse_block_pair(elseif_block_pair)?;
        elseif.push((elseif_expr, elseif_block));
    }

    // else
    let else_block = match if_inner.next() {
        Some(else_block_pair) => {
            let block = parse_block_pair(else_block_pair)?;
            Some(block)
        }
        None => None,
    };
    Ok(lua_program::Statement::IfBlock(
        (expr, block),
        elseif,
        else_block,
    ))
}

fn parse_return_statement_pair(
    statement_pair: Pair<Rule>,
) -> Result<lua_program::ReturnStatement, CompileError> {
    if statement_pair.as_rule() != Rule::RetStat {
        panic!("Expected pair to be a return statement when it was not")
    }
    let mut inner = statement_pair.into_inner();
    let expression_list = match inner.next() {
        Some(inner_val) => match parse_expression_list_pair(inner_val) {
            Ok(exp_list) => Some(exp_list),
            Err(e) => return Err(e),
        },
        None => None,
    };
    Ok(lua_program::ReturnStatement { expression_list })
}

fn parse_function_name_pair(pair: Pair<Rule>) -> Result<lua_program::FunctionName, CompileError> {
    if pair.as_rule() != Rule::FunctionName {
        panic!("Expected pair to be a function name when it was not")
    }
    let mut inner = pair.into_inner();
    let outer_name_pair = inner
        .next()
        .expect("Rule::FunctionName must have a name inner");
    let mut accessors: Vec<String> = Vec::new();
    while inner.peek().is_some() && inner.peek().unwrap().as_rule() == Rule::FunctionNameAccessor {
        let next = inner.next().unwrap();
        accessors.push(next.as_str().to_owned());
    }
    let pass_self = inner
        .next()
        .map(|final_pair| final_pair.as_str().to_owned());
    Ok(lua_program::FunctionName {
        outer_name: outer_name_pair.as_str().to_owned(),
        accessors,
        pass_self,
    })
}

fn parse_expression_list_pair(
    expr_list_pair: Pair<Rule>,
) -> Result<lua_program::ExpressionList, CompileError> {
    if expr_list_pair.as_rule() != Rule::ExpList {
        panic!("Expected pair to be an expression list when it was not")
    }
    let mut expressions: Vec<lua_program::Expression> = Vec::new();
    let inner = expr_list_pair.into_inner();
    for val in inner {
        let expression = parse_expression_pair(val)?;
        expressions.push(expression);
    }
    Ok(lua_program::ExpressionList { expressions })
}

fn parse_var_list_pair(var_list_pair: Pair<Rule>) -> Result<lua_program::VarList, CompileError> {
    if var_list_pair.as_rule() != Rule::VarList {
        panic!("Expected pair to be a var list when it was not")
    }
    let mut vars: Vec<lua_program::Var> = Vec::new();
    let inner = var_list_pair.into_inner();
    for val in inner {
        let var = parse_var_pair(val)?;
        vars.push(var);
    }
    Ok(lua_program::VarList { vars })
}

fn parse_var_pair(var_pair: Pair<Rule>) -> Result<lua_program::Var, CompileError> {
    if var_pair.as_rule() != Rule::Var {
        panic!("Expected pair to be a var when it was not")
    }
    let mut inner = var_pair.into_inner();
    let next = inner.next().expect("Rule::Var must contain an inner value");
    match next.as_rule() {
        Rule::VarNestedAccess => {
            let mut var_names: Vec<String> = Vec::new();
            let nested_access_inner = next.into_inner();
            for var_name in nested_access_inner {
                var_names.push(var_name.as_str().to_owned())
            }
            Ok(lua_program::Var::NestedAccess(var_names))
        }
        Rule::VarName => Ok(lua_program::Var::VarName(next.as_str().to_owned())),
        Rule::VarTableAccess => todo!(),
        _ => panic!("Matched on an undefined var"),
    }
}

fn parse_args_pair(args_pair: Pair<Rule>) -> Result<lua_program::Args, CompileError> {
    if args_pair.as_rule() != Rule::Args {
        panic!("Expected pair to be args when it was not")
    }
    let mut inner = args_pair.into_inner();
    match inner.next() {
        Some(next) => {
            match next.as_rule() {
                Rule::ExpList => {
                    let expression_list = parse_expression_list_pair(next)?;
                    Ok(lua_program::Args::ExpressionList(Some(expression_list)))
                }
                // TODO: TableConstructor
                // TODO: LiteralString
                _ => panic!("Matched on an undefined arg"),
            }
        }
        None => Ok(lua_program::Args::ExpressionList(None)),
    }
}

fn parse_expression_pair(
    statement_pair: Pair<Rule>,
) -> Result<lua_program::Expression, CompileError> {
    if statement_pair.as_rule() != Rule::Expression {
        panic!("Expected pair to be an expression when it was not")
    }
    let mut inner = statement_pair.into_inner();
    let next = inner.next().expect("Expression must have an inner value");
    match next.as_rule() {
        Rule::ExpressionInner => {
            let expr_inner = parse_expression_inner_pair(next)?;
            Ok(lua_program::Expression::Normal(expr_inner))
        }
        Rule::OperatorExpression => {
            let mut operator_exp_inner = next.into_inner();
            let first = operator_exp_inner
                .next()
                .expect("OperatorExpression must have at least one inner");
            match first.as_rule() {
                Rule::ExpressionInner => {
                    let first_expr = parse_expression_inner_pair(first)?;
                    let operation = parse_binary_operator_pair(operator_exp_inner.next().unwrap());
                    let second_expr =
                        parse_expression_inner_pair(operator_exp_inner.next().unwrap())?;
                    Ok(lua_program::Expression::Binary(
                        operation,
                        first_expr,
                        second_expr,
                    ))
                }
                Rule::UnaryOperator => {
                    let operation = parse_unary_operator_pair(first);
                    let expr = parse_expression_inner_pair(operator_exp_inner.next().unwrap())?;
                    Ok(lua_program::Expression::Unary(operation, expr))
                }
                _ => panic!("Matched on an undefined OperatorExpression"),
            }
        }
        _ => panic!("Matched on an undefined Expression"),
    }
}

fn parse_expression_inner_pair(pair: Pair<Rule>) -> Result<lua_program::ExprInner, CompileError> {
    if pair.as_rule() != Rule::ExpressionInner {
        panic!("Expected pair to be an expression inner when it was not")
    }
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .expect("Rule::ExpressionInner must contain an inner pair");
    let inner_str = first.as_str();
    if inner_str == "nil" {
        return Ok(lua_program::ExprInner::Nil);
    };
    if inner_str == "false" {
        return Ok(lua_program::ExprInner::Boolean(false));
    };
    if inner_str == "true" {
        return Ok(lua_program::ExprInner::Boolean(true));
    };
    match first.as_rule() {
        Rule::Numerical => {
            let number = parse_numerical_pair(first);
            Ok(lua_program::ExprInner::Numerical(number))
        }
        Rule::LiteralString => {
            let mut remove_quotes_inner = first.into_inner();
            let str_value = remove_quotes_inner.next().expect(
                "Rule::LiteralString must have an inner value used to strip quotation marks",
            );
            Ok(lua_program::ExprInner::LiteralString(
                str_value.as_str().to_owned(),
            ))
        }
        Rule::Expansion => Ok(lua_program::ExprInner::Expansion(lua_program::Expansion)),
        Rule::FunctionDef => {
            let function_body = parse_function_def_pair(first)?;
            Ok(lua_program::ExprInner::FunctionDef(function_body))
        }
        Rule::TableConstructor => todo!(),
        _ => panic!("Matched on an undefined ExpressionInner "),
    }
}

fn parse_binary_operator_pair(pair: Pair<Rule>) -> lua_program::BinaryOperator {
    if pair.as_rule() != Rule::BinaryOperator {
        panic!("Expected pair to be a binary operator")
    }
    let inner = pair.into_inner();
    match inner.as_str() {
        "+" => lua_program::BinaryOperator::MathOperator(lua_program::MathOperator::Plus),
        "-" => lua_program::BinaryOperator::MathOperator(lua_program::MathOperator::Minus),
        "*" => lua_program::BinaryOperator::MathOperator(lua_program::MathOperator::Multiply),
        "/" => lua_program::BinaryOperator::MathOperator(lua_program::MathOperator::FloatDivision),
        "//" => lua_program::BinaryOperator::MathOperator(lua_program::MathOperator::FloorDivision),
        "^" => lua_program::BinaryOperator::MathOperator(lua_program::MathOperator::Exponent),
        "%" => lua_program::BinaryOperator::MathOperator(lua_program::MathOperator::Mod),
        "&" => lua_program::BinaryOperator::BitwiseOperator(lua_program::BitwiseOperator::And),
        "~" => {
            lua_program::BinaryOperator::BitwiseOperator(lua_program::BitwiseOperator::ExclusiveOr)
        }
        "|" => lua_program::BinaryOperator::BitwiseOperator(lua_program::BitwiseOperator::Or),
        ">>" => {
            lua_program::BinaryOperator::BitwiseOperator(lua_program::BitwiseOperator::RightShift)
        }
        "<<" => {
            lua_program::BinaryOperator::BitwiseOperator(lua_program::BitwiseOperator::LeftShift)
        }
        ".." => lua_program::BinaryOperator::Concat,
        "<" => lua_program::BinaryOperator::BooleanOperator(lua_program::BooleanOperator::LessThan),
        "<=" => lua_program::BinaryOperator::BooleanOperator(
            lua_program::BooleanOperator::LessThanEqualTo,
        ),
        ">" => {
            lua_program::BinaryOperator::BooleanOperator(lua_program::BooleanOperator::GreaterThan)
        }
        ">=" => lua_program::BinaryOperator::BooleanOperator(
            lua_program::BooleanOperator::GreaterThanEqualTo,
        ),
        "==" => lua_program::BinaryOperator::BooleanOperator(lua_program::BooleanOperator::Equal),
        "~=" => lua_program::BinaryOperator::BooleanOperator(lua_program::BooleanOperator::Unequal),
        "and" => lua_program::BinaryOperator::BooleanOperator(lua_program::BooleanOperator::And),
        "or" => lua_program::BinaryOperator::BooleanOperator(lua_program::BooleanOperator::Or),
        _ => panic!("Matched on an undefined binary operator"),
    }
}

fn parse_unary_operator_pair(pair: Pair<Rule>) -> lua_program::UnaryOperator {
    if pair.as_rule() != Rule::UnaryOperator {
        panic!("Expected pair to be a unary operator")
    }
    let inner = pair.into_inner();
    match inner.as_str() {
        "-" => lua_program::UnaryOperator::UnaryMinus,
        "not" => lua_program::UnaryOperator::Not,
        "#" => lua_program::UnaryOperator::Length,
        "~" => lua_program::UnaryOperator::BitwiseUnaryNot,
        _ => panic!("Matched on an undefined unary operator"),
    }
}

fn parse_numerical_pair(pair: Pair<Rule>) -> lua_program::NumberKind {
    if pair.as_rule() != Rule::Numerical {
        panic!("Expected pair to be a numerical")
    }
    let mut inner = pair.into_inner();
    let next = inner
        .next()
        .expect("Rule::Numerical must contain an inner value");
    match next.as_rule() {
        Rule::Float => match next.as_str().parse::<f64>() {
            Ok(float) => lua_program::NumberKind::Float(float),
            Err(_) => panic!("Failed to parse a Rule::Float into an f64"),
        },
        Rule::Integer => match next.as_str().parse::<i64>() {
            Ok(int) => lua_program::NumberKind::Int(int),
            Err(_) => panic!("Failed to parse a Rule::Integer into an i64"),
        },
        _ => panic!("Matched on an undefined numerical"),
    }
}

fn parse_function_def_pair(pair: Pair<Rule>) -> Result<lua_program::FunctionBody, CompileError> {
    if pair.as_rule() != Rule::FunctionDef {
        panic!("Expected pair to be a FunctionDef")
    }
    let mut inner = pair.into_inner();
    let function_body_pair = inner
        .next()
        .expect("Rule::FunctionDef must contain one inner");
    parse_function_body(function_body_pair)
}

fn parse_function_body(pair: Pair<Rule>) -> Result<lua_program::FunctionBody, CompileError> {
    if pair.as_rule() != Rule::FunctionBody {
        panic!("Expected pair to be a FunctionBody")
    }
    let mut function_body_inner = pair.into_inner();
    let parameters = match function_body_inner.peek().is_some()
        && function_body_inner.peek().unwrap().as_rule() == Rule::ParList
    {
        true => Some(parse_params_pair(function_body_inner.next().unwrap())?),
        false => None,
    };
    let block = parse_block_pair(function_body_inner.next().unwrap())?;
    Ok(lua_program::FunctionBody { parameters, block })
}

fn parse_params_pair(pair: Pair<Rule>) -> Result<lua_program::Parameters, CompileError> {
    if pair.as_rule() != Rule::ParList {
        panic!("Expected pair to be a ParList")
    }
    let mut inner = pair.into_inner();
    let next = inner
        .next()
        .expect("Rule::ParList must have at least one inner value");
    match next.as_rule() {
        Rule::NameList => {
            let name_list = parse_name_list_pair(next)?;
            let expansion = inner.next().map(|_| lua_program::Expansion);
            Ok(lua_program::Parameters::Normal(name_list, expansion))
        }
        Rule::Expansion => Ok(lua_program::Parameters::Expanded(lua_program::Expansion)),
        _ => panic!("Matched on an undefined ParList inner"),
    }
}

fn parse_name_list_pair(pair: Pair<Rule>) -> Result<lua_program::NameList, CompileError> {
    if pair.as_rule() != Rule::NameList {
        panic!("Expected pair to be a NameList")
    }
    let mut names: Vec<String> = Vec::new();
    let inner = pair.into_inner();
    for inner_val in inner {
        names.push(inner_val.as_str().to_owned())
    }
    Ok(lua_program::NameList { names })
}
