use crate::clojure::*;
use crate::core::register_native_fn;
use crate::core::*;
use crate::edn::*;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, RwLock};

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

/// Read multiple forms from a string into a vector of EDN
fn read_forms(input: &str) -> Result<Vec<EDN>, ParseError> {
    let mut forms = Vec::new();
    let mut current = String::new();
    let mut paren_count = 0;

    for ch in input.chars() {
        match ch {
            '(' => {
                paren_count += 1;
                current.push(ch);
            }
            ')' => {
                paren_count -= 1;
                current.push(ch);
                if paren_count == 0 && !current.trim().is_empty() {
                    match read_string(&current) {
                        Ok(form) => forms.push(form),
                        Err(e) => return Err(e),
                    }
                    current.clear();
                }
            }
            _ => {
                if paren_count > 0 || !ch.is_whitespace() {
                    current.push(ch);
                }
            }
        }
    }

    Ok(forms)
}

pub fn repl(environment: &Environment) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        run_script_file(&args[1], environment);
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

        let ast = read_string(trimmed_input);
        match ast {
            Ok(ast) => {
                // Pass the Environment reference directly to eval
                match eval(ast, environment) {
                    Ok(val) => println!("{}", val),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(e) => eprintln!("Parse error: {:?}", e),
        }
    }
}

fn run_script_file(filename: &str, environment: &Environment) {
    match fs::read_to_string(filename) {
        Ok(content) => match read_forms(&content) {
            Ok(forms) => {
                for form in forms {
                    match eval(form, environment) {
                        Ok(val) => println!("{}", val),
                        Err(e) => {
                            eprintln!("Evaluation error: {}", e);
                            return;
                        }
                    }
                }
            }
            Err(e) => eprintln!("Parse error: {:?}", e),
        },
        Err(e) => eprintln!("Error reading file '{}': {}", filename, e),
    }
}

pub fn create_env() -> Environment {
    let env = Arc::new(RwLock::new(HashMap::new()));
    let env_clone = env.clone();

    {
        let mut env_write = env.write().unwrap();

        let eval_wrapper = move |args: Vec<Value>| -> Result<Value, String> {
            if args.len() != 1 {
                return Err("eval requires exactly 1 argument".to_string());
            }

            let expr = match &args[0] {
                Value::EDN(edn) => edn.clone(),
                _ => return Err("eval argument must be an EDN value".to_string()),
            };

            // Pass the thread-safe environment clone
            eval(expr, &env_clone)
        };

        // Register core functions
        register_native_fn(&mut env_write, "+", add);
        register_native_fn(&mut env_write, "-", subtract);
        register_native_fn(&mut env_write, "*", multiply);
        register_native_fn(&mut env_write, "/", divide);
        register_native_fn(&mut env_write, "prn", println_fn);
        register_native_fn(&mut env_write, "print", println_fn);
        register_native_fn(&mut env_write, "println", println_fn);
        register_native_fn(&mut env_write, "read-string", read_string_wrapper);
        register_native_fn(&mut env_write, "eval", eval_wrapper);
        register_native_fn(&mut env_write, "slurp", slurp_wrapper);
        register_native_fn(&mut env_write, "=", equal);
        register_native_fn(&mut env_write, "<", less_than);
        register_native_fn(&mut env_write, "<=", less_than_equal);
        register_native_fn(&mut env_write, ">", greater_than);
        register_native_fn(&mut env_write, ">=", greater_than_equal);
    }

    env
}
