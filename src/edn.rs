use crate::immutant::list;
use bigdecimal::BigDecimal;
use log::debug;
use num_bigint::BigInt;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;
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
    List(Box<list::List<EDN>>),
    Vector(Vec<EDN>),
    Map(HashMap<EDN, EDN>),
    Set(HashSet<EDN>),
}

#[derive(Debug)]
pub struct CollectionConfig {
    pub opening: &'static str,
    pub closing: &'static str,
    pub constructor: fn(Vec<EDN>) -> EDN,
}

#[derive(Debug)]
pub enum ParseError {
    NestingError(String),
    RegularError(String),
}

impl EDN {
    pub fn collection_config(collection_type: &EDN) -> CollectionConfig {
        match collection_type {
            EDN::List(_) => CollectionConfig {
                opening: "(",
                closing: ")",
                constructor: |items| EDN::List(Box::new(list::List::from_vec(items))),
            },
            EDN::Vector(_) => CollectionConfig {
                opening: "[",
                closing: "]",
                constructor: |items| EDN::Vector(items),
            },
            EDN::Set(_) => CollectionConfig {
                opening: "#{",
                closing: "}",
                constructor: vec_to_set,
            },
            EDN::Map(_) => CollectionConfig {
                opening: "{",
                closing: "}",
                constructor: |items| EDN::Map(vec_to_map(items)),
            },
            _ => panic!("Not a collection type"),
        }
    }
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
                for item in l.iter() {
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
        }
    }
}

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
