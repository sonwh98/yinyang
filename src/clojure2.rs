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

fn parse_collection(astr: &str) -> Result<EDN,String>{
    if astr.starts_with('"'){
	let mut buffer = String::new();
	let mut nesting_level = 0;
	for ch in astr.chars(){
	    match ch{
		'"' if nesting_level ==0 => {
		    nesting_level+=1;
		}
		'"' if nesting_level == 1 =>{
		    break;
		}
		_ => {
		    buffer.push(ch);
		}
	    }
	}
	return Ok(EDN::String(buffer));
    }
    Err("cannot parse collection".to_string())
}

pub fn read_string(astr: &str) -> Result<EDN, String> {
    let edn = parse_edn_primitives(astr);
    match edn{
	Ok(_) => { return edn;}
	Err(msg) => { return parse_collection(astr);}
    }
}
