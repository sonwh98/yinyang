use std::collections::HashMap;
use std::io::{self, Write};
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

/// Reads multiple lines until two consecutive newlines are entered or Ctrl+D is pressed.
fn read_multiline_input() -> Option<String> {
    let mut buffer = String::new();
    let mut empty_line_count = 0; // Tracks consecutive empty lines

    loop {
        print!("user=> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\nGoodbye!"); // Handle Ctrl+D (EOF)
                return None; 
            }
            Ok(_) => {
                // Check if the input is `:clj/quit`
                if input.trim() == ":clj/quit" {
                    println!("Goodbye!");
                    return None;
                }

                // Check if the input is an empty line
                if input.trim().is_empty() {
                    empty_line_count += 1;
                    if empty_line_count >= 2 {
                        break; // Stop reading, but do not exit the REPL
                    }
                } else {
                    empty_line_count = 0; // Reset if a non-empty line is entered
                }

                buffer.push_str(&input);
            }
            Err(_) => {
                eprintln!("Error reading input.");
                continue;
            }
        }
    }

    Some(buffer)
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

    loop {
        match read_multiline_input() {
            Some(input) if !input.trim().is_empty() => {
                match read_string(&input) {
                    Ok(ast) => match eval(ast, &mut env) {
                        Ok(val) => println!("{}", val),
                        Err(e) => eprintln!("Error: {}", e),
                    },
                    Err(e) => eprintln!("Parse error: {:?}", e),
                }
            }
            Some(_) => continue, // Empty input, restart REPL
            None => break, // Exit REPL on Ctrl+D or `:clj/quit`
        }
    }
}

fn main() {
    repl();
}
