mod ast;
mod err_handle;
mod frontend;

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Path to a Lua file
    #[arg()]
    path: String,
}

fn main() {
    let args = Args::parse();

    let path = PathBuf::from_str(args.path.as_str()).expect("infallible");
    if !path.exists() {
        panic!("Provided path is not valid")
    }
    let extension = path.extension();
    if !path.is_file() | extension.is_none() || extension.unwrap() != "lua" {
        panic!("Provided path does not correspond to a Lua file")
    }

    let file_contents =
        fs::read_to_string(path.as_path()).unwrap_or_else(|_| panic!("Failed to read Lua file"));

    match ast::parse_lua_program(file_contents.as_str()) {
        Ok(parsed_lua_program) => {
            match frontend::enter_program(parsed_lua_program) {
                Ok(_) => (),
                Err(e) => e.print_error()
            }
        }
        Err(e) => e.print_error(),
    }
}
