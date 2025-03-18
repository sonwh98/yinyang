use crate::clojure::*;
use crate::edn::*;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::fs;
use std::io::Read;

pub fn add(args: Vec<Value>) -> Result<Value, String> {
    let mut sum = BigDecimal::from(0);

    for arg in args {
        match arg {
            Value::EDN(EDN::Integer(i)) => {
                sum += BigDecimal::from(i);
            }
            Value::EDN(EDN::Float(f)) => {
                sum += f;
            }
            _ => return Err("Arguments to + must be numbers".to_string()),
        }
    }
    Ok(Value::EDN(EDN::Float(sum)))
}

pub fn subtract(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Subtract requires at least one argument".to_string());
    }

    let mut iter = args.into_iter();
    let first = match iter.next().unwrap() {
        Value::EDN(EDN::Integer(i)) => BigDecimal::from(i),
        Value::EDN(EDN::Float(f)) => f,
        _ => return Err("Arguments to - must be numbers".to_string()),
    };

    let result = iter.fold(first, |acc, arg| match arg {
        Value::EDN(EDN::Integer(i)) => acc - BigDecimal::from(i),
        Value::EDN(EDN::Float(f)) => acc - f,
        _ => acc,
    });

    Ok(Value::EDN(EDN::Float(result)))
}

pub fn multiply(args: Vec<Value>) -> Result<Value, String> {
    let mut product = BigDecimal::from(1);

    for arg in args {
        match arg {
            Value::EDN(EDN::Integer(i)) => {
                product *= BigDecimal::from(i);
            }
            Value::EDN(EDN::Float(f)) => {
                product *= f;
            }
            _ => return Err("Arguments to * must be numbers".to_string()),
        }
    }
    Ok(Value::EDN(EDN::Float(product)))
}

pub fn divide(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Divide requires at least one argument".to_string());
    }

    let mut iter = args.into_iter();
    let first = match iter.next().unwrap() {
        Value::EDN(EDN::Integer(i)) => BigDecimal::from(BigInt::from(i)), // Convert to BigInt first
        Value::EDN(EDN::Float(f)) => f,
        _ => return Err("Arguments to / must be numbers".to_string()),
    };

    let result = iter.try_fold(first, |acc, arg| match arg {
        Value::EDN(EDN::Integer(i)) => {
            let divisor = BigDecimal::from(BigInt::from(i)); // Convert to BigInt before division
            if divisor == BigDecimal::from(0) {
                Err("Division by zero".to_string())
            } else {
                Ok(acc / divisor)
            }
        }
        Value::EDN(EDN::Float(f)) => {
            if f == BigDecimal::from(0) {
                Err("Division by zero".to_string())
            } else {
                Ok(acc / f)
            }
        }
        _ => Err("Arguments to / must be numbers".to_string()),
    })?;

    Ok(Value::EDN(EDN::Float(result)))
}

pub fn println_fn(args: Vec<Value>) -> Result<Value, String> {
    let strings: Vec<String> = args.iter().map(|arg| format!("{}", arg)).collect();
    println!("{}", strings.join(" "));
    Ok(Value::EDN(EDN::Nil))
}

pub fn slurp(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("slurp requires exactly one argument".to_string());
    }

    match &args[0] {
        Value::EDN(EDN::String(path)) => {
            let mut file =
                fs::File::open(path).map_err(|e| format!("Error opening file: {}", e))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Error reading file: {}", e))?;
            Ok(Value::EDN(EDN::String(content)))
        }
        _ => Err("slurp argument must be a string representing a file path".to_string()),
    }
}

pub fn register_native_fn(
    env: &mut HashMap<String, Value>,
    name: &str,
    f: fn(Vec<Value>) -> Result<Value, String>,
) {
    env.insert(name.to_string(), Value::Function(Callable::Native(f)));
}
