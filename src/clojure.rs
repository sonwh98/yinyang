use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
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

// Collection parsing infrastructure
#[derive(Debug)]
enum CollectionType {
    List,
    Vector,
    Set,
    Map,
}

#[derive(Debug)]
struct CollectionConfig {
    opening: &'static str,
    closing: char,
    constructor: fn(Vec<EDN>) -> EDN,
}

impl CollectionType {
    fn config(&self) -> CollectionConfig {
        match self {
            CollectionType::List => CollectionConfig {
                opening: "(",
                closing: ')',
                constructor: |items| EDN::List(items),
            },
            CollectionType::Vector => CollectionConfig {
                opening: "[",
                closing: ']',
                constructor: |items| EDN::Vector(items),
            },
            CollectionType::Set => CollectionConfig {
                opening: "#{",
                closing: '}',
                constructor: vec_to_set,
            },
            CollectionType::Map => CollectionConfig {
                opening: "{",
                closing: '}',
                constructor: |items| EDN::Map(vec_to_map(items)),
            },
        }
    }
}

// Basic type parsers
fn parse_nil(astr: &str) -> Result<EDN, String> {
    if astr == "nil" || astr.is_empty() {
        Ok(EDN::Nil)
    } else {
        Err(format!("No EDN::Nil: {}", astr))
    }
}

fn parse_bool(astr: &str) -> Result<EDN, String> {
    astr.parse::<bool>()
        .map(EDN::Bool)
        .map_err(|_| format!("No EDN::Bool: {}", astr))
}

fn parse_int(astr: &str) -> Result<EDN, String> {
    astr.parse::<BigInt>()
        .map(EDN::Integer)
        .map_err(|_| format!("No EDN::Integer: {}", astr))
}

fn parse_float(astr: &str) -> Result<EDN, String> {
    BigDecimal::from_str(astr)
        .map(EDN::Float)
        .map_err(|_| format!("No EDN::Float: {}", astr))
}

fn parse_keyword(astr: &str) -> Result<EDN, String> {
    if astr.starts_with(':') {
        Ok(EDN::Keyword(astr.to_string()))
    } else {
        Err(format!("No EDN::Keyword: {}", astr))
    }
}

fn parse_string(astr: &str) -> Result<EDN, String> {
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
    Err("Cannot parse string".to_string())
}

fn parse_symbol(astr: &str) -> Result<EDN, String> {
    let symbol_regex = Regex::new(
        r"(?x)
        [\w.!@$%^&|=<>?+/~*^-]
        [-a-zA-Z0-9_!@$%^&|=<>?.+/~*^-]*
        ",
    )
    .unwrap();

    if symbol_regex.is_match(astr) {
        Ok(EDN::Symbol(astr.to_string()))
    } else {
        Err(format!("Cannot parse symbol {:?}", astr))
    }
}

// Collection parsing helpers
fn parse_collection_helper(
    astr_iter: &mut Chars,
    mut nesting_level: i8,
    items: &mut Vec<EDN>,
) -> Result<EDN, String> {
    let mut buffer = String::new();
    
    while let Some(ch) = astr_iter.next() {
        if !matches!(ch, ' ' | ',' | ')' | ']' | '}') {
            buffer.push(ch);
        }
	println!("buffer={:?} ch={:?} level={:?}", buffer, ch, nesting_level);
        match buffer.as_str() {
            "(" => {
                handle_nested_collection(
                    CollectionType::List,
                    astr_iter,
                    &mut nesting_level,
                    items,
                    &mut buffer,
                )?;
            }
            "[" => {
                handle_nested_collection(
                    CollectionType::Vector,
                    astr_iter,
                    &mut nesting_level,
                    items,
                    &mut buffer,
                )?;
            }
            "#{" => {
                handle_nested_collection(
                    CollectionType::Set,
                    astr_iter,
                    &mut nesting_level,
                    items,
                    &mut buffer,
                )?;
            }
            "{" => {
                handle_nested_collection(
                    CollectionType::Map,
                    astr_iter,
                    &mut nesting_level,
                    items,
                    &mut buffer,
                )?;
            }
            _ => {
                if matches!(ch, ')' | ']' | '}') {
                    nesting_level -= 1;
                    handle_buffer(&mut buffer, items)?;
                    if nesting_level == 0 {
                        break;
                    }
                } else if matches!(ch, ' ' | ',') {
                    handle_buffer(&mut buffer, items)?;
                }
            }
        }
    }

    if nesting_level != 0 {
        return Err("Unmatched delimiters".to_string());
    }

    Ok((CollectionType::List.config().constructor)(items.to_vec()))
}

fn handle_nested_collection(
    collection_type: CollectionType,
    astr_iter: &mut Chars,
    nesting_level: &mut i8,
    items: &mut Vec<EDN>,
    buffer: &mut String,
) -> Result<(), String> {
    if *nesting_level > 0 {
        let nested = parse_collection_helper(astr_iter, 1, &mut Vec::new())?;
        items.push(nested);
    } else {
        *nesting_level += 1;
    }
    buffer.clear();
    Ok(())
}

fn handle_buffer(buffer: &mut String, items: &mut Vec<EDN>) -> Result<(), String> {
    if !buffer.is_empty() {
        if let Ok(edn_val) = read_string(&buffer.trim()) {
            items.push(edn_val);
        }
        buffer.clear();
    }
    Ok(())
}

// Collection parsers
fn parse_list(astr: &str) -> Result<EDN, String> {
    parse_collection_with_type(astr, CollectionType::List)
}

fn parse_vector(astr: &str) -> Result<EDN, String> {
    parse_collection_with_type(astr, CollectionType::Vector)
}

fn parse_set(astr: &str) -> Result<EDN, String> {
    parse_collection_with_type(astr, CollectionType::Set)
}

fn parse_map(astr: &str) -> Result<EDN, String> {
    parse_collection_with_type(astr, CollectionType::Map)
}

fn parse_collection_with_type(astr: &str, collection_type: CollectionType) -> Result<EDN, String> {
    let astr = astr.trim();
    let config = collection_type.config();
    println!("astr={:?} config={:?}",astr, config);

    if astr.starts_with(config.opening) {
        let mut items = Vec::new();
        let result = parse_collection_helper(&mut astr.chars(), 0, &mut items)?;
        Ok((config.constructor)(match result {
            EDN::List(items) => items,
            _ => unreachable!(),
        }))
    } else {
        Err(format!("Cannot parse {}", config.opening))
    }
}

// Utility functions
fn vec_to_set(items: Vec<EDN>) -> EDN {
    let mut set = HashSet::new();
    for item in items {
        set.insert(item);
    }
    EDN::Set(set)
}

fn vec_to_map<V>(v: Vec<V>) -> HashMap<V, V>
where
    V: Eq + Hash + Clone,
{
    let mut map = HashMap::new();
    for pair in v.chunks(2) {
        if let [key, value] = pair {
            map.insert(key.clone(), value.clone());
        }
    }
    map
}

// Parser entry points
fn parse_all(astr: &str) -> Result<EDN, String> {
    parse_nil(astr)
        .or_else(|_| parse_bool(astr))
        .or_else(|_| parse_int(astr))
        .or_else(|_| parse_float(astr))
        .or_else(|_| parse_keyword(astr))
        .or_else(|_| parse_string(astr))
        .or_else(|_| parse_map(astr))
        .or_else(|_| parse_vector(astr))
        .or_else(|_| parse_list(astr))
}

fn parse_first_valid_expr(astr: &str) -> Result<EDN, String> {
    let first = astr.split_whitespace().next().unwrap_or("");
    parse_all(first).or_else(|_| parse_symbol(first))
}

pub fn read_string(astr: &str) -> Result<EDN, String> {
    let astr = astr.trim();
    parse_all(astr)
        .or_else(|_| parse_first_valid_expr(astr))
        .or_else(|_| parse_symbol(astr))
}
