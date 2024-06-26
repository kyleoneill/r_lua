use pest::iterators::{Pairs, Pair};
use pest::Parser;
use pest_derive::Parser;

use crate::lua_program::{LuaProgram, Block};
use crate::err_handle::CompileError;

const RESERVED_KEYWORDS: [&str; 22] = ["and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while"];

#[derive(Parser, Debug)]
#[grammar = "grammar.pest"]
pub struct LuaTokenPairs;

pub fn parse_lua_program(input: &str) -> Result<LuaProgram, CompileError> {
    match LuaTokenPairs::parse(Rule::Main, input) {
        Ok(mut parsed) => {
            let pair = parsed.next().expect("Lua program must begin with a block");
            // TODO: REMOVE ME
            println!("{:?}", pair);

            let block = parse_block_pair(pair)?;
            Ok(LuaProgram { block })
        },
        Err(e) => Err(CompileError::from_pest_error(e))
    }
}

fn parse_block_pair(block_pair: Pair<Rule>) -> Result<Block, CompileError> {
    if block_pair.as_rule() != Rule::Block {
        panic!("Expected pair to be a block when it was not")
    }
    Ok(Block{})
}


// TODO: Invariants to check for
// 1. Name must not be equal to any value in RESERVED_KEYWORDS
// 2. Statement::MultipleAssignment must have an equal number of vars and expressions
// 3. BinaryOperator::Concat -> Both ExprInner of whatever is calling the binary operation
//    must resolve to either a String or something that implements ToString
