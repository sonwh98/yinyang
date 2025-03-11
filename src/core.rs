use crate::clojure::*;
use bigdecimal::BigDecimal;

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

pub fn println_fn(args: Vec<Value>) -> Result<Value, String> {
    let strings: Vec<String> = args.iter().map(|arg| format!("{}", arg)).collect();
    println!("{}", strings.join(" "));
    Ok(Value::EDN(EDN::Nil))
}
