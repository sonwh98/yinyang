use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::ops::Add;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum EDN {
    Nil,
    Bool(bool),
    Integer(BigInt),
    Float(BigDecimal),
    String(String),
    Symbol(String),
    Keyword(String),
    List(Vec<EDN>),
    Vector(Vec<EDN>),
    Map(HashMap<EDN, EDN>),
    Set(HashSet<EDN>),
    Function(fn(EDN) -> EDN),
}

impl PartialEq for EDN {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EDN::Nil, EDN::Nil) => true,
            (EDN::Bool(b1), EDN::Bool(b2)) => b1 == b2,
            (EDN::Integer(i1), EDN::Integer(i2)) => i1 == i2,
            (EDN::Float(f1), EDN::Float(f2)) => f1 == f2,
            (EDN::String(s1), EDN::String(s2)) => s1 == s2,
            (EDN::Symbol(sym1), EDN::Symbol(sym2)) => sym1 == sym2,
            (EDN::Keyword(k1), EDN::Keyword(k2)) => k1 == k2,
            (EDN::List(l1), EDN::List(l2)) => l1 == l2,
            (EDN::Vector(v1), EDN::Vector(v2)) => v1 == v2,
            (EDN::Map(m1), EDN::Map(m2)) => m1 == m2,
            (EDN::Set(s1), EDN::Set(s2)) => s1 == s2,
            _ => false,
        }
    }
}
impl Add for EDN {
    type Output = Result<EDN, &'static str>;

    fn add(self, other: EDN) -> Self::Output {
        match (self, other) {
            (EDN::Integer(a), EDN::Integer(b)) => Ok(EDN::Integer(a + b)),
            _ => Err("Both EDN values must be Integer variants to perform addition."),
        }
    }
}

impl Eq for EDN {}

impl Hash for EDN {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            EDN::Nil => state.write_u8(0),
            EDN::Bool(b) => {
                state.write_u8(1);
                b.hash(state);
            }
            EDN::Integer(i) => {
                state.write_u8(2);
                i.hash(state);
            }
            EDN::Float(f) => {
                state.write_u8(3);
                f.hash(state);
            }
            EDN::String(s) => {
                state.write_u8(4);
                s.hash(state);
            }
            EDN::Symbol(s) => {
                state.write_u8(5);
                s.hash(state);
            }
            EDN::Keyword(s) => {
                state.write_u8(6);
                s.hash(state);
            }
            EDN::List(l) => {
                state.write_u8(7);
                for item in l {
                    item.hash(state);
                }
            }
            EDN::Vector(v) => {
                state.write_u8(8);
                for item in v {
                    item.hash(state);
                }
            }
            EDN::Map(m) => {
                state.write_u8(9);
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            EDN::Set(s) => {
                state.write_u8(10);
                for item in s {
                    item.hash(state);
                }
            }
            EDN::Function(f) => {
                state.write_u8(11);
                // Use the address of the function pointer for hashing
                (f as *const _ as usize).hash(state);
            }
        }
    }
}

fn parse_edn_value<'a, I>(iter: &mut I) -> Result<EDN, String>
where
    I: Iterator<Item = &'a str>,
{
    if let Some(token) = iter.next() {
        if token.starts_with(':') {
            Ok(EDN::Keyword(token[1..].to_string()))
        } else if let Ok(int_val) = token.parse::<BigInt>() {
            Ok(EDN::Integer(int_val))
        } else if token.starts_with('"') && token.ends_with('"') {
            Ok(EDN::String(token[1..token.len() - 1].to_string()))
        } else if token == "nil" {
            Ok(EDN::Nil)
        } else if token == "true" {
            Ok(EDN::Bool(true))
        } else if token == "false" {
            Ok(EDN::Bool(false))
        } else if token.starts_with('{') && token.ends_with('}') {
            read_string(token)
        } else {
            Ok(EDN::Symbol(token.to_string()))
        }
    } else {
        Err("Unexpected end of input".to_string())
    }
}

pub fn read_string(input: &str) -> Result<EDN, String> {
    let input = input.trim();

    if input == "nil" {
        return Ok(EDN::Nil);
    }

    if let Ok(b) = input.parse::<bool>() {
        return Ok(EDN::Bool(b));
    }

    if let Ok(i) = input.parse::<BigInt>() {
        return Ok(EDN::Integer(i));
    }

    if let Ok(f) = BigDecimal::from_str(input) {
        return Ok(EDN::Float(f));
    }

    if input.starts_with('"') && input.ends_with('"') {
        let content = &input[1..input.len() - 1];
        return Ok(EDN::String(content.to_string()));
    }

    if input.starts_with(':') {
        return Ok(EDN::Keyword(input.to_string()));
    }

    if input.starts_with('(') && input.ends_with(')') {
        let items = &input[1..input.len() - 1];
        let parsed_items = read_string_helper(items)?;
        return Ok(EDN::List(parsed_items));
    }

    if input.starts_with('[') && input.ends_with(']') {
        let items = &input[1..input.len() - 1];
        let parsed_items = read_string_helper(items)?;
        return Ok(EDN::Vector(parsed_items));
    }

    if input.starts_with('{') && input.ends_with('}') {
        let content = &input[1..input.len() - 1];
        let mut map = HashMap::new();
        let mut key_val_iter = content.split_whitespace().peekable();

        while key_val_iter.peek().is_some() {
            let key = parse_edn_value(&mut key_val_iter)?;
            let val = parse_edn_value(&mut key_val_iter)?;
            map.insert(key, val);
        }
        return Ok(EDN::Map(map));
    }

    if input.starts_with("#{") && input.ends_with('}') {
        let items = &input[2..input.len() - 1];
        let parsed_items = read_string_helper(items)?
            .into_iter()
            .collect::<HashSet<_>>();
        return Ok(EDN::Set(parsed_items));
    }

    let symbol_regex = Regex::new(
        r"(?x)                      # Enable verbose mode
    [\w.!@$%^&|=<>?+/~*^-]           # Match a single character from this set
    [-a-zA-Z0-9_!@$%^&|=<>?.+/~*^-]* # Match zero or more characters from this set
    ",
    )
    .unwrap();

    if symbol_regex.is_match(input) {
        return Ok(EDN::Symbol(input.to_string()));
    }

    Err(format!("Unable to parse EDN: {}", input))
}

fn read_string_helper(input: &str) -> Result<Vec<EDN>, String> {
    let mut items = Vec::new();
    let mut buffer = String::new();
    let mut nesting_level = 0;
    for ch in input.chars() {
        match ch {
            '(' | '[' | '{' => {
                nesting_level += 1;
                buffer.push(ch);
            }
            ')' | ']' | '}' => {
                nesting_level -= 1;
                buffer.push(ch);
                if nesting_level == 0 {
                    items.push(read_string(&buffer.trim())?);
                    buffer.clear();
                }
            }
            ' ' if nesting_level == 0 => {
                if !buffer.is_empty() {
                    items.push(read_string(&buffer.trim())?);
                    buffer.clear();
                }
            }
            _ => buffer.push(ch),
        }
    }
    if !buffer.is_empty() {
        items.push(read_string(&buffer.trim())?);
    }
    Ok(items)
}

impl fmt::Display for EDN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EDN::Nil => write!(f, "nil"),
            EDN::Bool(b) => write!(f, "{}", b),
            EDN::Integer(i) => write!(f, "{}", i),
            EDN::Float(d) => write!(f, "{}", d),
            EDN::String(s) => write!(f, "\"{}\"", s),
            EDN::Symbol(sym) => write!(f, "{}", sym),
            EDN::Keyword(k) => write!(f, "{}", k),
            EDN::List(l) => {
                write!(f, "(")?;
                for (i, item) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            EDN::Vector(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            EDN::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "}}")
            }
            EDN::Set(s) => {
                write!(f, "#{{")?;
                for (i, item) in s.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "}}")
            }
            EDN::Function(func) => write!(f, "Function({:p})", func),
        }
    }
}

type Context = HashMap<String, EDN>;

fn eval(ctx: &mut Context, edn: &EDN) -> Result<EDN, String> {
    match edn {
        EDN::List(l) => {
            let f = l.first();
            if let Some(EDN::Symbol(sym)) = f {
                if sym.as_str() == "def" {
                    let var = &l[1];
                    let var_val = &l[2];
                    if let EDN::Symbol(v) = var {
                        ctx.insert(
                            v.clone(),
                            eval(&mut ctx.clone(), var_val).unwrap_or(EDN::Nil),
                        );
                    }
                    return Ok(var_val.clone());
                } else {
                    let sym_val = ctx.get(sym).unwrap_or(&EDN::Nil);
                    match sym_val {
                        EDN::Function(fun) => {
                            let args = l[1..].to_vec();
                            let eval_args: Vec<EDN> = args
                                .iter()
                                .map(|a| eval(&mut ctx.clone(), a).unwrap_or(EDN::Nil))
                                .collect();
                            let args_edn = EDN::Vector(eval_args);
                            return Ok(fun(args_edn));
                        }
                        _ => {
                            return Err("don't know how to eval".to_string());
                        }
                    }
                }
            }
            return Ok(EDN::Nil);
        }
        EDN::Symbol(s) => {
            let sym_val = ctx.get(s);
            if let Some(v) = sym_val {
                return Ok(v.clone());
            } else {
                println!("Unable to resolve {:?}", edn);
                return Ok(EDN::Nil);
            }
        }
        EDN::Bool(_)
        | EDN::Integer(_)
        | EDN::Float(_)
        | EDN::String(_)
        | EDN::Keyword(_)
        | EDN::Vector(_)
        | EDN::Map(_)
        | EDN::Set(_) => {
            return Ok(edn.clone());
        }
        _ => {
            println!("Error");
            return Ok(EDN::Nil);
        }
    }
}

pub fn sum(edn: EDN) -> EDN {
    match edn {
        EDN::Vector(vec) => {
            let a = vec.iter().fold(BigInt::from(0), |acc, item| {
                if let EDN::Integer(ref num) = item {
                    acc + num
                } else {
                    acc
                }
            });
            return EDN::Integer(a);
        }
        _ => EDN::Integer(BigInt::from(0)),
    }
}

pub fn mul(edn: EDN) -> EDN {
    match edn {
        EDN::Vector(vec) => {
            let a = vec.iter().fold(BigInt::from(1), |acc, item| {
                if let EDN::Integer(ref num) = item {
                    acc * num
                } else {
                    acc
                }
            });
            return EDN::Integer(a);
        }
        _ => EDN::Integer(BigInt::from(0)),
    }
}

fn read_string_example() {
    let examples = vec![
        "(1 (2 3) 4)",
        "[1 2 [3 4] 5]",
        "{:a 1 :b 2 :c 3}",
        "{1 2 2 4}",
        "{\"first-name\" \"Sonny\" \"last-name\" \"Su\"}",
        "#{1 2}",
        "#{{1 2} {3 4}}",
        "#{{:a :b} {:c :d}}",
        "nil",
        "true",
        "false",
        "123",
        "45.67",
        "\"a string\"",
        ":keyword",
        "symbol",
        "+",
        "-",
        "*",
        "/",
        "_",
        "$",
        "?",
        "<",
        ">",
        "!",
        "|",
        "%",
        "@",
        ".",
        "..",
        "(+ 1 1)",
        "(defn sum [a b] (+ a b))",
        "(-> a b c)",
        "(->> foo bar)",
        "(. Foo bar 1 2 3)",
        "(.. Foo (bar 1 2 3))",
    ];

    for example in examples {
        match read_string(example) {
            Ok(edn) => println!("Parsed: {} -> {:?}", example, edn),
            Err(e) => println!("Error: {} -> {}", example, e),
        }
    }
}

fn eval_examples() {
    let mut ctx = HashMap::new();

    ctx.insert("+".to_string(), EDN::Function(sum));
    ctx.insert("-".to_string(), EDN::Function(sum));
    ctx.insert("*".to_string(), EDN::Function(mul));
    let add = read_string("(+ 1 2 3 4 5 6)").unwrap();
    let mul = read_string("(* 1 2 3 4 5 6)").unwrap();
    let sub = read_string("(- 1 2 3 4 5 6)").unwrap();
    let a = eval(&mut ctx, &add).unwrap();
    let m = eval(&mut ctx, &mul).unwrap();
    let s = eval(&mut ctx, &sub).unwrap();
    println!("a= {:?} m={:?} s={:?}", a, m, s);

    read_string("(defn add [a b] (+ a b))").unwrap();
}

fn repl() {
    let mut ctx = HashMap::new();
    ctx.insert("+".to_string(), EDN::Function(sum));

    loop {
        print!("repl> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        let bytes_read = io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        let ctrl_d = 0;
        if bytes_read == ctrl_d || input == ":clj/quit" {
            break;
        }

        let parsed_edn = read_string(input);
        match parsed_edn {
            Ok(edn) => {
                let result = eval(&mut ctx, &edn);
                match result {
                    Ok(edn) => {
                        println!("{}", edn);
                    }
                    Err(err) => {
                        println!("cannot eval {:?}", edn);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading input: {:?}", err);
            }
        }
    }
}

fn main() {
    //read_string_example();
    //eval_examples();
    repl();
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn test_read_string_simple_addition() {
        let input = "(+ 1 1)";
        let expected = EDN::List(
            (vec![
                EDN::Symbol("+".to_string()),
                EDN::Integer(BigInt::from(1)),
                EDN::Integer(BigInt::from(1)),
            ]),
        );

        match read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }

    #[test]
    fn test_read_string_nested_list() {
        let input = "(1 (2 3) 4)";
        let expected = EDN::List(
            (vec![
                EDN::Integer(BigInt::from(1)),
                EDN::List((vec![EDN::Integer(BigInt::from(2)), EDN::Integer(BigInt::from(3))])),
                EDN::Integer(BigInt::from(4)),
            ]),
        );

        match read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }

    #[test]
    fn test_read_string_vector() {
        let input = "[1 2 [3 4] 5]";
        let expected = EDN::Vector(vec![
            EDN::Integer(BigInt::from(1)),
            EDN::Integer(BigInt::from(2)),
            EDN::Vector(vec![
                EDN::Integer(BigInt::from(3)),
                EDN::Integer(BigInt::from(4)),
            ]),
            EDN::Integer(BigInt::from(5)),
        ]);

        match read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }

    #[test]
    fn test_read_string_map() {
        let input = "{:a 1 :b 2}";
        let mut expected_map = HashMap::new();
        expected_map.insert(EDN::Keyword("a".to_string()), EDN::Integer(BigInt::from(1)));
        expected_map.insert(EDN::Keyword("b".to_string()), EDN::Integer(BigInt::from(2)));
        let expected = EDN::Map(expected_map);
        println!("expected={:?}", expected);

        match read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }

    #[test]
    fn test_read_string_set() {
        let input = "#{{1 2} {3 4}}";

        let mut map1 = HashMap::new();
        map1.insert(EDN::Integer(BigInt::from(1)), EDN::Integer(BigInt::from(2)));

        let mut map2 = HashMap::new();
        map2.insert(EDN::Integer(BigInt::from(3)), EDN::Integer(BigInt::from(4)));

        let mut expected_set = HashSet::new();
        expected_set.insert(EDN::Map(map1));
        expected_set.insert(EDN::Map(map2));

        let expected = EDN::Set(expected_set);

        match read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }
}
