use crate::clojure::*;
use crate::edn::*;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::sync::Arc;

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

pub fn slurp(path: &str) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Error opening file: {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Error reading file: {}", e))?;
    Ok(content)
}

pub fn slurp_wrapper(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("slurp requires exactly one argument".to_string());
    }

    match &args[0] {
        Value::EDN(EDN::String(path)) => {
            slurp(path).map(|content| Value::EDN(EDN::String(content)))
        }
        _ => Err("slurp argument must be a string representing a file path".to_string()),
    }
}

pub fn register_native_fn<F>(env: &mut HashMap<String, Value>, name: &str, f: F)
where
    F: Fn(Vec<Value>) -> Result<Value, String> + Send + Sync + 'static,
{
    env.insert(
        name.to_string(),
        Value::Function(Callable::Native(NativeFn(Arc::new(f)))),
    );
}

/// Computes a hash for a `Value`
/// Delegates to EDN where applicable
fn hash_value(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    match value {
        Value::EDN(edn) => edn.hash(&mut hasher), // Use EDN's Hash implementation
        Value::Var { ns, name, value } => {
            ns.hash(&mut hasher);
            name.hash(&mut hasher);
            hash_value(value).hash(&mut hasher);
        }
        Value::Function(_) => {
            // Functions cannot be compared for equality meaningfully
            hasher.write_u8(255);
        }
    }
    hasher.finish()
}

/// Generalized equality function using hash comparison
pub fn equal(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("equal requires at least two arguments".to_string());
    }

    let first_hash = hash_value(&args[0]);

    for arg in &args[1..] {
        if hash_value(arg) != first_hash {
            return Ok(Value::EDN(EDN::Bool(false)));
        }
    }

    Ok(Value::EDN(EDN::Bool(true)))
}

pub fn less_than(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("< requires at least two arguments".to_string());
    }

    // Convert first argument to BigDecimal
    let first = match &args[0] {
        Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
        Value::EDN(EDN::Float(f)) => f.clone(),
        _ => return Err("Arguments to < must be numbers".to_string()),
    };

    // Compare each pair of adjacent numbers
    let mut prev = first;
    for arg in &args[1..] {
        let curr = match arg {
            Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
            Value::EDN(EDN::Float(f)) => f.clone(),
            _ => return Err("Arguments to < must be numbers".to_string()),
        };

        if prev >= curr {
            return Ok(Value::EDN(EDN::Bool(false)));
        }
        prev = curr;
    }

    Ok(Value::EDN(EDN::Bool(true)))
}

pub fn less_than_equal(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("<= requires at least two arguments".to_string());
    }

    // Convert first argument to BigDecimal
    let first = match &args[0] {
        Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
        Value::EDN(EDN::Float(f)) => f.clone(),
        _ => return Err("Arguments to <= must be numbers".to_string()),
    };

    // Compare each pair of adjacent numbers
    let mut prev = first;
    for arg in &args[1..] {
        let curr = match arg {
            Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
            Value::EDN(EDN::Float(f)) => f.clone(),
            _ => return Err("Arguments to <= must be numbers".to_string()),
        };

        if prev > curr {
            return Ok(Value::EDN(EDN::Bool(false)));
        }
        prev = curr;
    }

    Ok(Value::EDN(EDN::Bool(true)))
}

pub fn greater_than(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("> requires at least two arguments".to_string());
    }

    // Convert first argument to BigDecimal
    let first = match &args[0] {
        Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
        Value::EDN(EDN::Float(f)) => f.clone(),
        _ => return Err("Arguments to > must be numbers".to_string()),
    };

    // Compare each pair of adjacent numbers
    let mut prev = first;
    for arg in &args[1..] {
        let curr = match arg {
            Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
            Value::EDN(EDN::Float(f)) => f.clone(),
            _ => return Err("Arguments to > must be numbers".to_string()),
        };

        if prev < curr {
            return Ok(Value::EDN(EDN::Bool(false)));
        }
        prev = curr;
    }

    Ok(Value::EDN(EDN::Bool(true)))
}

pub fn greater_than_equal(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(">= requires at least two arguments".to_string());
    }

    // Convert first argument to BigDecimal
    let first = match &args[0] {
        Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
        Value::EDN(EDN::Float(f)) => f.clone(),
        _ => return Err("Arguments to >= must be numbers".to_string()),
    };

    // Compare each pair of adjacent numbers
    let mut prev = first;
    for arg in &args[1..] {
        let curr = match arg {
            Value::EDN(EDN::Integer(i)) => BigDecimal::from(i.clone()),
            Value::EDN(EDN::Float(f)) => f.clone(),
            _ => return Err("Arguments to >= must be numbers".to_string()),
        };

        if prev < curr {
            return Ok(Value::EDN(EDN::Bool(false)));
        }
        prev = curr;
    }

    Ok(Value::EDN(EDN::Bool(true)))
}
