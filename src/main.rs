use bigdecimal::BigDecimal;
use regex::Regex;
use num_bigint::BigInt;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Debug, Clone)]
enum EDN {
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
                l.hash(state);
            }
            EDN::Vector(v) => {
                state.write_u8(8);
                v.hash(state);
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
                for v in s {
                    v.hash(state);
                }
            }
        }
    }
}

impl fmt::Display for EDN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EDN::Nil => write!(f, "nil"),
            EDN::Bool(b) => write!(f, "{}", b),
            EDN::Integer(i) => write!(f, "{}", i),
            EDN::Float(dec) => write!(f, "{}", dec),
            EDN::String(s) => write!(f, "\"{}\"", s),
            EDN::Symbol(sym) => write!(f, "{}", sym),
            EDN::Keyword(kw) => write!(f, "{}", kw),
            EDN::List(list) => {
                write!(f, "(")?;
                for (i, item) in list.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i < list.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")
            }
            EDN::Vector(vec) => {
                write!(f, "[")?;
                for (i, item) in vec.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i < vec.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, "]")
            }
            EDN::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    write!(f, "{} {}", k, v)?;
                    if i < map.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            EDN::Set(set) => {
                write!(f, "#{{")?;
                for (i, item) in set.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i < set.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

impl EDN {
    fn parse(input: &str) -> Result<EDN, String> {
        // For simplicity, let's start with basic parsing
        if input.trim() == "nil" {
            return Ok(EDN::Nil);
        }

        if let Ok(boolean) = input.trim().parse::<bool>() {
            return Ok(EDN::Bool(boolean));
        }

        if let Some(integer) = BigInt::parse_bytes(input.trim().as_bytes(), 10) {
            return Ok(EDN::Integer(integer));
        }

        if let Ok(float) = BigDecimal::from_str(input.trim()) {
            return Ok(EDN::Float(float));
        }

        if input.starts_with("\"") && input.ends_with("\"") {
            let s = &input[1..input.len() - 1];
            return Ok(EDN::String(s.to_string()));
        }

        if input.starts_with(':') {
            return Ok(EDN::Keyword(input.to_string()));
        }

        if input.starts_with('(') && input.ends_with(')') {
            let items = &input[1..input.len() - 1];
            let parsed_items = items
                .split_whitespace()
                .map(|item| EDN::parse(item))
                .collect::<Result<Vec<EDN>, String>>()?;
            return Ok(EDN::List(parsed_items));
        }

        if input.starts_with('[') && input.ends_with(']') {
            let items = &input[1..input.len() - 1];
            let parsed_items = items
                .split_whitespace()
                .map(|item| EDN::parse(item))
                .collect::<Result<Vec<EDN>, String>>()?;
            return Ok(EDN::Vector(parsed_items));
        }

        if input.starts_with('{') && input.ends_with('}') {
            let items = &input[1..input.len() - 1];
            let mut map = HashMap::new();
            let pairs = items
                .split(',')
                .map(|pair| {
                    let mut kv = pair.split_whitespace();
                    let k = kv.next().ok_or("Missing key")?;
                    let v = kv.next().ok_or("Missing value")?;
                    Ok((EDN::parse(k)?, EDN::parse(v)?))
                })
                .collect::<Result<Vec<(EDN, EDN)>, String>>()?;
            for (k, v) in pairs {
                map.insert(k, v);
            }
            return Ok(EDN::Map(map));
        }

        if input.starts_with('#') && input.ends_with('}') {
            let items = &input[2..input.len() - 1];
            let parsed_items = items
                .split_whitespace()
                .map(|item| EDN::parse(item))
                .collect::<Result<HashSet<EDN>, String>>()?;
            return Ok(EDN::Set(parsed_items));
        }

        let symbol_regex =
            Regex::new(r"[a-zA-Z_*!@$%^&|=<>?][a-zA-Z0-9_*!@$%^&|=<>?.+-]*").unwrap();

        if symbol_regex.is_match(input) {
            return Ok(EDN::Symbol(input.to_string()));
        }

        Err(format!("Unable to parse EDN: {}", input))
    }
}

fn main() {
    let examples = vec![
        "nil",
        "true",
        "false",
        "42",
        "3.14",
        "\"Hello, EDN!\"",
        ":my-keyword",
        "(1 2 3)",
        "[true \"vector\"]",
        "{:key \"value\"}",
        "#{1 2 3}",
        "foobar",
	"((+ 2 3 ))"
    ];

    for example in examples {
        match EDN::parse(example) {
            Ok(edn) => println!("Parsed: {} -> {:?}", example, edn),
            Err(e) => println!("Error: {} -> {}", example, e),
        }
    }
}
