use crate::clojure::*;
use crate::core::*;
use crate::edn::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

fn read_string_wrapper(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("read-string requires exactly 1 argument".to_string());
    }

    let s = match &args[0] {
        Value::EDN(EDN::String(s)) => s,
        _ => return Err("read-string argument must be a string".to_string()),
    };

    let v = read_string(s).unwrap();
    Ok(Value::EDN(v))
}

fn eval_wrapper(args: Vec<Value>) -> Result<Value, String> {
    let mut env = HashMap::new();
    register_native_fn(&mut env, "+", add);

    if args.len() != 1 {
        return Err("eval requires exactly 1 argument".to_string());
    }

    let expr = match &args[0] {
        Value::EDN(edn) => edn.clone(),
        _ => return Err("eval argument must be an EDN value".to_string()),
    };
    eval(expr, &mut env)
}

/// Function to check if parentheses, brackets, and braces are balanced
fn is_form_complete(input: &str) -> bool {
    let mut stack = Vec::new();

    for c in input.chars() {
        match c {
            '(' | '[' | '{' => stack.push(c),
            ')' => {
                if stack.pop() != Some('(') {
                    return false;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return false;
                }
            }
            '}' => {
                if stack.pop() != Some('{') {
                    return false;
                }
            }
            _ => {}
        }
    }

    stack.is_empty()
}

pub fn repl(env: &mut HashMap<String, Value>) {
    // Check for script file argument first
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // Run a script file if provided
        let filename = &args[1];
        match fs::read_to_string(filename) {
            Ok(content) => match read_string(&content) {
                Ok(ast) => match eval(ast, env) {
                    Ok(val) => println!("{}", val),
                    Err(e) => eprintln!("Error: {}", e),
                },
                Err(e) => eprintln!("Parse error: {:?}", e),
            },
            Err(e) => eprintln!("Error reading file '{}': {}", filename, e),
        }
        return;
    }

    // Start interactive REPL
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());

    loop {
        let mut buffer = String::new();

        print!("user=> ");
        if io::stdout().flush().is_err() {
            eprintln!("Error: Failed to flush stdout");
            continue;
        }

        let mut line = String::new();
        while reader.read_line(&mut line).is_ok() {
            if line.is_empty() {
                println!("\nExiting REPL...");
                return;
            }

            buffer.push_str(&line);

            if is_form_complete(&buffer) {
                break;
            }

            print!("...   ");
            if io::stdout().flush().is_err() {
                eprintln!("Error: Failed to flush stdout");
                continue;
            }

            line.clear();
        }

        let trimmed_input = buffer.trim();
        if trimmed_input.is_empty() {
            continue;
        }

        if !is_form_complete(trimmed_input) {
            eprintln!("Error: Unmatched parentheses/brackets/braces in input.");
            continue;
        }

        let ast = read_string(trimmed_input);
        match ast {
            Ok(ast) => match eval(ast, env) {
                Ok(val) => println!("{}", val),
                Err(e) => eprintln!("Error: {}", e),
            },
            Err(e) => eprintln!("Parse error: {:?}", e),
        }
    }
}

pub fn create_env() -> HashMap<String, Value> {
    let mut env = HashMap::new();

    // Register core functions
    register_native_fn(&mut env, "+", add);
    register_native_fn(&mut env, "-", subtract);
    register_native_fn(&mut env, "*", multiply);
    register_native_fn(&mut env, "/", divide);
    register_native_fn(&mut env, "prn", println_fn);
    register_native_fn(&mut env, "print", println_fn);
    register_native_fn(&mut env, "println", println_fn);
    register_native_fn(&mut env, "read-string", read_string_wrapper);
    register_native_fn(&mut env, "eval", eval_wrapper);
    register_native_fn(&mut env, "slurp", slurp);
    register_native_fn(&mut env, "=", equal);
    register_native_fn(&mut env, "<", less_than);
    register_native_fn(&mut env, "<=", less_than_equal);
    register_native_fn(&mut env, ">", greater_than_equal);
    register_native_fn(&mut env, ">=", greater_than_equal);

    env
}
