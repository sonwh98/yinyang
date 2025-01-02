use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use regex::Regex;
use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::ops::Add;
use std::str::Chars;
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

impl Eq for EDN {}

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

impl fmt::Display for EDN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EDN::Nil => write!(f, "nil"),
            EDN::Bool(b) => write!(f, "{}", b),
            EDN::Integer(i) => write!(f, "{}", i),
            EDN::Float(d) => write!(f, "{}", d.to_string()),
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
                    write!(f, ":{} {}", k, v)?;
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

fn parse_nil(astr: &str) -> Result<EDN, String> {
    if astr == "nil" || astr.is_empty() {
        return Ok(EDN::Nil);
    } else {
        return Err(format!("No EDN::Nil: {}", astr));
    }
}

fn parse_bool(astr: &str) -> Result<EDN, String> {
    if let Ok(b) = astr.parse::<bool>() {
        return Ok(EDN::Bool(b));
    } else {
        return Err(format!("No EDN::Bool: {}", astr));
    }
}

fn parse_int(astr: &str) -> Result<EDN, String> {
    if let Ok(i) = astr.parse::<BigInt>() {
        return Ok(EDN::Integer(i));
    } else {
        return Err(format!("No EDN::Integer: {}", astr));
    }
}

fn parse_float(astr: &str) -> Result<EDN, String> {
    if let Ok(f) = BigDecimal::from_str(astr) {
        return Ok(EDN::Float(f));
    } else {
        return Err(format!("No EDN::Float: {}", astr));
    }
}

fn parse_keyword(astr: &str) -> Result<EDN, String> {
    if astr.starts_with(':') {
        return Ok(EDN::Keyword(astr.to_string()));
    } else {
        return Err(format!("No EDN::Keyword: {}", astr));
    }
}
fn parse_first_valid_expr(astr: &str) -> Result<EDN, String> {
    let mut token_iter = astr.split_whitespace();
    let first = token_iter.nth(0).unwrap_or("");

    return parse_nil(first)
        .or_else(|_| parse_bool(first))
        .or_else(|_| parse_int(first))
        .or_else(|_| parse_float(first))
        .or_else(|_| parse_keyword(first))
        .or_else(|_| parse_string(first))
        .or_else(|_| parse_list(first))
        .or_else(|_| parse_symbol(first));
}

fn parse_symbol(astr: &str) -> Result<EDN, String> {
    let symbol_regex = Regex::new(
        r"(?x)                      # Enable verbose mode
    [\w.!@$%^&|=<>?+/~*^-]           # Match a single character from this set
    [-a-zA-Z0-9_!@$%^&|=<>?.+/~*^-]* # Match zero or more characters from this set
    ",
    )
    .unwrap();

    if symbol_regex.is_match(astr) {
        return Ok(EDN::Symbol(astr.to_string()));
    }
    Ok(EDN::Symbol(astr.to_string()))
}

fn parse_string(astr: &str) -> Result<EDN, String> {
    if astr.starts_with('"') {
        let mut buffer = String::new();
        let mut nesting_level = 0;
        for ch in astr.chars() {
            match ch {
                '"' if nesting_level == 0 => {
                    nesting_level += 1;
                }
                '"' if nesting_level == 1 => {
                    break;
                }
                _ => {
                    buffer.push(ch);
                }
            }
        }
        return Ok(EDN::String(buffer));
    } else {
        return Err("cannot parse string".to_string());
    }
}
// (def a 1 (def b 2))
fn parse_list(astr: &str) -> Result<EDN, String> {
    let astr = astr.trim();
    if astr.starts_with('(') {
        parse_list_helper(&mut astr.chars(), 0, &mut Vec::new())
    } else {
        return Err("cannot parse list".to_string());
    }
}

fn concat(a_list: &EDN, b_list: &EDN) -> EDN {
    match (a_list, b_list) {
        (EDN::List(a), EDN::List(b)) => {
            let mut result = a.clone();
            result.extend(b.clone());
            EDN::List(result)
        }
        _ => panic!("Both arguments must be EDN::List"),
    }
}

fn into(a_list: &EDN, an_item: &EDN) -> EDN {
    match (a_list, an_item) {
        (EDN::List(a), EDN::List(b)) => {
            let mut result = a.clone();
            result.push(EDN::List(b.clone()));
            EDN::List(result)
        }
        _ => panic!("Both arguments must be EDN::List"),
    }
}

fn parse_list_helper(
    astr_iter: &mut Chars,
    mut nesting_level: i8,
    items: &mut Vec<EDN>,
) -> Result<EDN, String> {
    let mut buffer = String::new();
    let mut index = 0;

    while let Some(ch) = astr_iter.next() {
        match ch {
            '(' => {
                if nesting_level > 0 {
                    let a_list = parse_list_helper(astr_iter, 1, &mut Vec::new());
                    items.extend(a_list);
                } else {
                    nesting_level += 1;
                }
            }
            ')' => {
                nesting_level -= 1;
                if !buffer.is_empty() {
                    let edn_val = read_string(&buffer.trim()).unwrap();
                    items.push(edn_val);
                }
                buffer.clear();
                if nesting_level == 0 {
                    break;
                }
            }
            ' ' | ',' => {
                if !buffer.is_empty() {
                    let edn_val = read_string(&buffer.trim()).unwrap();
                    items.push(edn_val);
                    buffer.clear();
                }
            }
            _ => buffer.push(ch),
        }
    }

    if !buffer.is_empty() {
        let token_iterator = buffer.split_whitespace();
        let mut edn_tokens: Vec<EDN> = token_iterator
            .map(|token| {
                return read_string(token).unwrap();
            })
            .collect();
        let a_list = EDN::List(edn_tokens.clone());
        return Ok(into(&a_list, &items[0]));
    } else {
        return Ok(EDN::List(items.to_vec()));
    }
}

pub fn read_string(astr: &str) -> Result<EDN, String> {
    let astr = astr.trim();
    parse_nil(astr)
        .or_else(|_| parse_bool(astr))
        .or_else(|_| parse_int(astr))
        .or_else(|_| parse_float(astr))
        .or_else(|_| parse_keyword(astr))
        .or_else(|_| parse_string(astr))
        .or_else(|_| parse_list(astr))
        .or_else(|_| parse_first_valid_expr(astr))
        .or_else(|_| parse_symbol(astr))
}
