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

fn parse_edn_primitives(astr: &str) -> Result<EDN, String> {
    let astr = astr.trim();
    println!("astr = {:?}", astr);

    if astr == "nil" || astr.is_empty() {
        return Ok(EDN::Nil);
    }

    if let Ok(b) = astr.parse::<bool>() {
        return Ok(EDN::Bool(b));
    }

    if let Ok(i) = astr.parse::<BigInt>() {
        return Ok(EDN::Integer(i));
    }

    if let Ok(f) = BigDecimal::from_str(astr) {
        return Ok(EDN::Float(f));
    }

    if astr.starts_with(':') {
        return Ok(EDN::Keyword(astr.to_string()));
    }

    Err(format!("No primitives: {}", astr))
}

pub fn read_string(input: &str) -> Result<EDN, String> {
    let edn = parse_edn_primitives(input);
    return edn;
}
