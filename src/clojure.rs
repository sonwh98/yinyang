use crate::edn::*;
use crate::immutant::list;
use bigdecimal::BigDecimal;
use log::debug;
use num_bigint::BigInt;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use std::str::Chars;
use std::str::FromStr;
use std::sync::Arc;

pub struct NativeFn(pub Arc<dyn Fn(Vec<Value>) -> Result<Value, String> + Send + Sync + 'static>);

impl Clone for NativeFn {
    fn clone(&self) -> Self {
        NativeFn(self.0.clone())
    }
}

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native-fn>")
    }
}

#[derive(Debug, Clone)]
pub enum Callable {
    Lambda {
        params: Vec<Value>,
        body: EDN,
        closure: HashMap<String, Value>,
    },
    Native(NativeFn),
}

impl Callable {
    fn call(&self, args: Vec<Value>) -> Result<Value, String> {
        match self {
            Callable::Lambda {
                params,
                body,
                closure,
            } => {
                if args.len() != params.len() {
                    return Err(format!(
                        "Expected {} args, got {}",
                        params.len(),
                        args.len()
                    ));
                }

                let mut new_env = closure.clone();
                for (param, arg) in params.iter().zip(args.iter()) {
                    match param {
                        Value::EDN(EDN::Symbol(name)) => {
                            new_env.insert(name.clone(), arg.clone());
                        }
                        _ => return Err("Parameter must be a symbol".to_string()),
                    }
                }

                eval(body.clone(), &mut new_env)
            }
            Callable::Native(f) => f.0(args),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    EDN(EDN),
    Var {
        ns: String,
        name: String,
        value: Box<Value>,
    },
    Function(Callable),
    // Future additions:
    // Atom(AtomRef),
    // Class(Class),
    // etc.
}

impl fmt::Display for Value {
    #[allow(unused_variables)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::EDN(edn) => write!(f, "{}", edn),
            Value::Var { ns, name, value } => write!(f, "#'{}/{}", ns, name),
            Value::Function(_) => write!(f, "#<function>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // EDN values compare using EDN's PartialEq
            (Value::EDN(e1), Value::EDN(e2)) => e1 == e2,

            // Vars compare by namespace and name
            (
                Value::Var {
                    ns: ns1,
                    name: name1,
                    value: value1,
                },
                Value::Var {
                    ns: ns2,
                    name: name2,
                    value: value2,
                },
            ) => ns1 == ns2 && name1 == name2 && value1 == value2,

            // Different variants are never equal
            _ => false,
        }
    }
}

impl From<EDN> for Value {
    fn from(edn: EDN) -> Self {
        Value::EDN(edn)
    }
}

fn handle_nested_string(astr_iter: &mut Chars, items: &mut Vec<EDN>, buffer: &mut String) {
    //let orig_str: String = astr_iter.clone().collect();

    while let Some(ch) = astr_iter.next() {
        if ch == '"' {
            break;
        } else {
            buffer.push(ch);
        }
    }
    let val = EDN::String(buffer.to_string());
    items.push(val);
    buffer.clear();
}

fn parse_collection_helper(
    astr_iter: &mut Chars,
    mut nesting_level: i8,
    items: &mut Vec<EDN>,
    collection_type: &EDN,
) -> Result<EDN, ParseError> {
    let orig_str: String = astr_iter.clone().collect();
    let mut buffer = String::new();
    let config = EDN::collection_config(collection_type);
    let closing_char = config.closing.chars().next().unwrap();
    let mut string_start = false;

    while let Some(ch) = astr_iter.next() {
        if ch == '"' && string_start == false {
            handle_nested_string(astr_iter, items, &mut buffer);
            string_start = true;
        }

        if !matches!(ch, ' ' | '\n' | ',' | ')' | ']' | '}') {
            buffer.push(ch);
        }

        match buffer.as_str() {
            "(" => handle_nested_collection(
                &EDN::List(Box::new(list::List::new())),
                astr_iter,
                &mut nesting_level,
                items,
                &mut buffer,
            ),
            "[" => handle_nested_collection(
                &EDN::Vector(Vec::new()),
                astr_iter,
                &mut nesting_level,
                items,
                &mut buffer,
            ),
            "#{" => handle_nested_collection(
                &EDN::Set(HashSet::new()),
                astr_iter,
                &mut nesting_level,
                items,
                &mut buffer,
            ),
            "{" => handle_nested_collection(
                &EDN::Map(HashMap::new()),
                astr_iter,
                &mut nesting_level,
                items,
                &mut buffer,
            ),
            _ => {
                if ch == closing_char {
                    nesting_level -= 1;
                    parse_buffer(&mut buffer, items);
                    if nesting_level == 0 {
                        break;
                    }
                } else if matches!(ch, ' ' | '\n' | ',') {
                    parse_buffer(&mut buffer, items);
                }
            }
        }
    }

    if nesting_level != 0 {
        return Err(ParseError::NestingError(format!(
            "Unmatched delimiters {:?} buffer={:?}",
            orig_str, buffer
        )));
    }

    Ok((config.constructor)(items.to_vec()))
}

fn handle_nested_collection(
    collection_type: &EDN,
    astr_iter: &mut Chars,
    nesting_level: &mut i8,
    items: &mut Vec<EDN>,
    buffer: &mut String,
) {
    if *nesting_level > 0 {
        let nested =
            parse_collection_helper(astr_iter, 1, &mut Vec::new(), collection_type).unwrap();
        items.push(nested);
    } else {
        *nesting_level += 1;
    }
    buffer.clear();
}

fn parse_buffer(buffer: &mut String, items: &mut Vec<EDN>) {
    if !buffer.is_empty() {
        if let Ok(edn_val) = read_string(&buffer.trim()) {
            items.push(edn_val);
        }
        buffer.clear();
    }
}

fn parse_collection_with_type(astr: &str, collection_type: &EDN) -> Result<EDN, ParseError> {
    let astr = astr.trim();
    let config = EDN::collection_config(collection_type);

    if astr.starts_with(config.opening) {
        let mut items = Vec::new();
        let result = parse_collection_helper(&mut astr.chars(), 0, &mut items, collection_type)?;
        Ok(result)
    } else {
        Err(ParseError::RegularError(format!(
            "Cannot parse {}",
            config.opening
        )))
    }
}

fn parse_nil(astr: &str) -> Result<EDN, ParseError> {
    if astr == "nil" || astr.is_empty() {
        Ok(EDN::Nil)
    } else {
        Err(ParseError::RegularError(format!("No EDN::Nil: {}", astr)))
    }
}

fn parse_bool(astr: &str) -> Result<EDN, ParseError> {
    astr.parse::<bool>()
        .map(EDN::Bool)
        .map_err(|_| ParseError::RegularError(format!("No EDN::Bool: {}", astr)))
}

fn parse_int(astr: &str) -> Result<EDN, ParseError> {
    astr.parse::<BigInt>()
        .map(EDN::Integer)
        .map_err(|_| ParseError::RegularError(format!("No EDN::Integer: {}", astr)))
}

fn parse_float(astr: &str) -> Result<EDN, ParseError> {
    BigDecimal::from_str(astr)
        .map(EDN::Float)
        .map_err(|_| ParseError::RegularError(format!("No EDN::Float: {}", astr)))
}

fn parse_keyword(astr: &str) -> Result<EDN, ParseError> {
    if astr.starts_with(':') {
        Ok(EDN::Keyword(astr.to_string()))
    } else {
        Err(ParseError::RegularError(format!(
            "No EDN::Keyword: {}",
            astr
        )))
    }
}

fn parse_string(astr: &str) -> Result<EDN, ParseError> {
    if astr.starts_with('"') {
        let mut buffer = String::new();
        let mut chars = astr.chars();
        chars.next(); // Skip opening quote

        while let Some(ch) = chars.next() {
            if ch == '"' {
                return Ok(EDN::String(buffer));
            }
            buffer.push(ch);
        }
    }
    Err(ParseError::RegularError("Cannot parse string".to_string()))
}

pub fn parse_symbol(astr: &str) -> Result<EDN, ParseError> {
    if matches!(astr, "nil" | "true" | "false") {
        return Err(ParseError::RegularError(format!("Reserved name: {}", astr)));
    }

    let symbol_regex = Regex::new(
        r"^[a-zA-Z*+!_?$%&=<>'#\-][a-zA-Z0-9*+!_?$%&=<>'#\-\.]*(?:/[a-zA-Z0-9*+!_?$%&=<>'#\-\.]+)?$"
    ).unwrap();

    if symbol_regex.is_match(astr) && !astr.ends_with(':') {
        Ok(EDN::Symbol(astr.to_string()))
    } else {
        Err(ParseError::RegularError(format!(
            "Cannot parse symbol {:?}",
            astr
        )))
    }
}

fn parse_list(astr: &str) -> Result<EDN, ParseError> {
    parse_collection_with_type(astr, &EDN::List(Box::new(list::List::new())))
}

fn parse_vector(astr: &str) -> Result<EDN, ParseError> {
    parse_collection_with_type(astr, &EDN::Vector(Vec::new()))
}

fn parse_set(astr: &str) -> Result<EDN, ParseError> {
    parse_collection_with_type(astr, &EDN::Set(HashSet::new()))
}

fn parse_map(astr: &str) -> Result<EDN, ParseError> {
    parse_collection_with_type(astr, &EDN::Map(HashMap::new()))
}

fn parse_all(astr: &str) -> Result<EDN, ParseError> {
    let parsers: Vec<fn(&str) -> Result<EDN, ParseError>> = vec![
        parse_nil,
        parse_bool,
        parse_int,
        parse_float,
        parse_keyword,
        parse_string,
        parse_list,
        parse_map,
        parse_set,
        parse_vector,
    ];

    for parser in parsers {
        debug!("parser={:?}", parser);
        let ast = parser(astr);
        match ast {
            Ok(result) => return Ok(result),
            Err(ParseError::NestingError(e)) => {
                return Err(ParseError::NestingError(e));
            }
            Err(ParseError::RegularError(_s)) => continue,
        }
    }

    Err(ParseError::RegularError("No valid EDN found".to_string()))
}

fn parse_first_valid_expr(astr: &str) -> Result<EDN, ParseError> {
    let first = astr.split_whitespace().next().unwrap_or("");
    parse_all(first)
}

fn replace_quote_syntax(input: &str) -> String {
    let re = Regex::new(r"'([^\s\(\)\[\]\{\}]+|\(.*?\)|\[.*?\]|\{.*?\})").unwrap();

    let mut output = input.to_string();
    while re.is_match(&output) {
        output = re
            .replace_all(&output, |caps: &regex::Captures| {
                let quoted = &caps[1];
                format!("(quote {})", quoted)
            })
            .to_string();
    }

    output
}

pub fn read_string(astr: &str) -> Result<EDN, ParseError> {
    let astr = &replace_quote_syntax(astr.trim());
    parse_all(astr)
        .or_else(|_| parse_first_valid_expr(astr))
        .or_else(|e| match e {
            ParseError::NestingError(_) => return Err(e),
            ParseError::RegularError(_) => {
                return parse_symbol(astr);
            }
        })
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::EDN(EDN::Nil) => false,
        Value::EDN(EDN::Bool(false)) => false,
        _ => true,
    }
}

fn is_special_form(form_name: &str) -> bool {
    let special_forms = ["quote", "if", "def", "fn"];
    special_forms.contains(&form_name)
}

pub fn eval(ast: EDN, env: &mut HashMap<String, Value>) -> Result<Value, String> {
    match ast {
        EDN::List(list) => {
            let l = *list;
            println!("eval l={:?}", l);
            l.first()
                .ok_or("Empty list".to_string())
                .and_then(|first| match first {
                    EDN::Symbol(s) => eval_special_form(&s, &l.rest().to_vec(), env)
                        .or_else(|e| eval_function_call(&l.to_vec(), env)),
                    _ => Err("Expected a function symbol".to_string()),
                })
        }
        EDN::Symbol(ref s) => {
            println!("sonny ast={:?}", &ast);
            if is_special_form(s) {
                Ok(Value::EDN(EDN::Symbol(s.clone())))
            } else {
                env.get(s)
                    .cloned()
                    .ok_or_else(|| format!("Undefined symbol: {}", s))
            }
        }
        _ => Ok(Value::EDN(ast)),
    }
}

fn eval_special_form(
    form: &str,
    args: &[EDN],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    eval_quote(form, args)
        .or_else(|_| eval_do(form, args, env))
        .or_else(|e| {
            let foo = eval_if(form, args, env);
            println!("unknown1 special form: {:?} e={:?} foo={:?}", form, e, foo);
            return foo;
        })
        .or_else(|_| {
            println!("unknown2 special form: {:?}", form);
            eval_def(form, args, env)
        })
        .or_else(|_| eval_let(form, args, env))
        .or_else(|e| {
            println!("unknown3 special form: {:?}", form);
            eval_fn(form, args, env)
        })
        .or_else(|e| {
            println!("unknown4 special form: {:?}", form);
            Err(format!("Unknown special form: {}", form))
        })
}

fn eval_quote(form: &str, args: &[EDN]) -> Result<Value, String> {
    if form != "quote" {
        return Err("Not a quote form".to_string());
    }

    if args.len() == 1 {
        Ok(Value::EDN(args[0].clone()))
    } else {
        Err("Incorrect number of arguments for 'quote'".to_string())
    }
}

fn eval_do(form: &str, args: &[EDN], env: &mut HashMap<String, Value>) -> Result<Value, String> {
    if form != "do" {
        return Err("Not a do form".to_string());
    }

    args.iter()
        .try_fold(Value::EDN(EDN::Nil), |_, expr| eval(expr.clone(), env))
}

fn eval_if(form: &str, args: &[EDN], env: &mut HashMap<String, Value>) -> Result<Value, String> {
    if form != "if" {
        return Err("Not an if form".to_string());
    }
    println!("eval_if1 args={:?}", args);
    if args.len() < 2 || args.len() > 3 {
        return Err("'if' requires 2 or 3 arguments".to_string());
    }
    println!("eval_if2 args={:?}", args);

    eval(args[0].clone(), env).and_then(|condition| {
        println!("condition={:?}", condition);
        if is_truthy(&condition) {
            eval(args[1].clone(), env)
        } else if args.len() == 3 {
            eval(args[2].clone(), env)
        } else {
            Ok(Value::EDN(EDN::Nil))
        }
    })
}

fn eval_def(form: &str, args: &[EDN], env: &mut HashMap<String, Value>) -> Result<Value, String> {
    if form != "def" {
        return Err("Not a def form".to_string());
    }

    if args.len() != 2 {
        return Err("'def' requires exactly 2 arguments".to_string());
    }

    let symbol = match &args[0] {
        EDN::Symbol(name) => Ok(name.clone()),
        _ => Err("First argument to 'def' must be a symbol".to_string()),
    }?;

    let value = eval(args[1].clone(), env)?;
    env.insert(symbol.clone(), value.clone());

    Ok(Value::Var {
        ns: "user".to_string(),
        name: symbol,
        value: Box::new(value),
    })
}

fn eval_let(form: &str, args: &[EDN], env: &mut HashMap<String, Value>) -> Result<Value, String> {
    if form != "let" {
        return Err("Not a let form".to_string());
    }

    if args.len() != 2 {
        return Err("'let' requires exactly 2 arguments".to_string());
    }

    let bindings = match &args[0] {
        EDN::Vector(bindings) => bindings,
        _ => return Err("First argument to 'let' must be a vector".to_string()),
    };

    if bindings.len() % 2 != 0 {
        return Err("Binding vector requires an even number of forms".to_string());
    }

    let mut new_env = env.clone();

    // Process bindings in pairs
    for chunk in bindings.chunks(2) {
        let sym = match &chunk[0] {
            EDN::Symbol(name) => name.clone(),
            _ => return Err("Binding target must be a symbol".to_string()),
        };

        let val = eval(chunk[1].clone(), &mut new_env)?;
        new_env.insert(sym, val);
    }

    // Evaluate body in new environment
    eval(args[1].clone(), &mut new_env)
}

fn eval_fn(form: &str, args: &[EDN], env: &mut HashMap<String, Value>) -> Result<Value, String> {
    if form != "fn" {
        return Err("Not a fn form".to_string());
    }

    if args.len() != 2 {
        return Err("'fn' requires exactly 2 arguments".to_string());
    }

    let params = match &args[0] {
        EDN::Vector(param_list) => param_list
            .iter()
            .map(|param| match param {
                EDN::Symbol(name) => Ok(Value::EDN(EDN::Symbol(name.clone()))),
                _ => Err("Parameters must be symbols".to_string()),
            })
            .collect::<Result<Vec<Value>, String>>()?,
        _ => return Err("First argument to 'fn' must be a vector".to_string()),
    };

    Ok(Value::Function(Callable::Lambda {
        params,
        body: args[1].clone(),
        closure: env.clone(),
    }))
}

fn eval_function_call(list: &[EDN], env: &mut HashMap<String, Value>) -> Result<Value, String> {
    // Evaluate first element to get the function
    let func = eval(list[0].clone(), env)?;

    match func {
        Value::Function(f) => {
            // Evaluate all arguments
            let args: Result<Vec<Value>, String> =
                list[1..].iter().map(|arg| eval(arg.clone(), env)).collect();
            let args = args?;
            println!("call {:?} args={:?}", f, args);
            f.call(args)
        }
        _ => {
            println!("sonny list={:?}", list);
            Err("First element is not a function".to_string())
        }
    }
}
