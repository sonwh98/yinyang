use yinyang::clojure::*;
use yinyang::core::*;

use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

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

pub fn repl() {
    let mut env = HashMap::new();
    let eval_fn = |args: Vec<Value>| -> Result<Value, String> {
        if args.len() != 1 {
            return Err("eval requires exactly 1 argument".to_string());
        }

        let expr = match &args[0] {
            Value::EDN(edn) => edn.clone(),
            _ => return Err("eval argument must be an EDN value".to_string()),
        };
        eval(expr, &mut env)
    };

    register_native_fn(&mut env, "+", add);
    register_native_fn(&mut env, "prn", println_fn);
    register_native_fn(&mut env, "print", println_fn);
    register_native_fn(&mut env, "println", println_fn);
    register_native_fn(&mut env, "read-string", read_string_wrapper);

    loop {
        print!("user=> ");
        if io::stdout().flush().is_err() {
            eprintln!("Error: Failed to flush stdout");
            continue;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                if input.trim().is_empty() {
                    continue;
                }

                match read_string(&input) {
                    Ok(ast) => match eval(ast, &mut env) {
                        Ok(val) => println!("{}", val),
                        Err(e) => eprintln!("Error: {}", e),
                    },
                    Err(e) => eprintln!("Parse error: {:?}", e),
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }
    }
}

fn main() {
    repl();
}
