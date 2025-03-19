use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use yinyang::clojure::*;
use yinyang::core::*;
use yinyang::edn::*;

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

pub fn repl() {
    let mut env = HashMap::new();

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

    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());

    loop {
        let mut buffer = String::new();

        print!("user=> ");
        if io::stdout().flush().is_err() {
            eprintln!("Error: Failed to flush stdout");
            continue;
        }

        // Read input line-by-line until the form is complete
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok() {
            if line.is_empty() {
                println!("\nExiting REPL...");
                return; // Exit on EOF (Ctrl-D)
            }

            buffer.push_str(&line);

            // Check if the form is complete
            if is_form_complete(&buffer) {
                break;
            }

            print!("...   "); // Indicate multi-line input
            if io::stdout().flush().is_err() {
                eprintln!("Error: Failed to flush stdout");
                continue;
            }

            line.clear(); // Clear line buffer for next input
        }

        let trimmed_input = buffer.trim();
        if trimmed_input.is_empty() {
            continue;
        }

        // Check for mismatched or unclosed forms
        if !is_form_complete(trimmed_input) {
            eprintln!("Error: Unmatched parentheses/brackets/braces in input.");
            continue;
        }

        let ast = read_string(trimmed_input);
        match ast {
            Ok(ast) => match eval(ast, &mut env) {
                Ok(val) => println!("{}", val),
                Err(e) => eprintln!("Error: {}", e),
            },
            Err(e) => eprintln!("Parse error: {:?}", e),
        }
    }
}

fn main() {
    repl();
}
