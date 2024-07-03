use bigdecimal::BigDecimal;
use imbl::{HashMap, HashSet, Vector};
use num_bigint::BigInt;
use regex::Regex;
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
    List(Vector<EDN>),
    Vector(Vector<EDN>),
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
        }
    }
}

impl EDN {
    fn read_string(input: &str) -> Result<EDN, String> {
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
            let parsed_items = Self::parse_items(items)?;
            return Ok(EDN::List(parsed_items));
        }

        if input.starts_with('[') && input.ends_with(']') {
            let items = &input[1..input.len() - 1];
            let parsed_items = Self::parse_items(items)?;
            return Ok(EDN::Vector(parsed_items));
        }

        if input.starts_with('{') && input.ends_with('}') {
            let items = &input[1..input.len() - 1];
            let mut map = HashMap::new();
            let pairs = Self::split_items(items)?;
            for pair in pairs {
                let mut kv = pair.splitn(2, ' ');
                let k = kv.next().ok_or("Missing key")?;
                let v = kv.next().ok_or("Missing value")?;
                map.insert(EDN::read_string(k)?, EDN::read_string(v)?);
            }
            return Ok(EDN::Map(map));
        }

        if input.starts_with('#') && input.ends_with('}') {
            let items = &input[2..input.len() - 1];
            let parsed_items = Self::parse_items(items)?
                .into_iter()
                .collect::<HashSet<_>>();
            return Ok(EDN::Set(parsed_items));
        }

        let symbol_regex =
            Regex::new(r"[\.\w*!@$%^&|=<>?+/~-][-a-zA-Z0-9_*!@$%^&|=<>?.+/~-]*").unwrap();
        if symbol_regex.is_match(input) {
            return Ok(EDN::Symbol(input.to_string()));
        }

        Err(format!("Unable to parse EDN: {}", input))
    }

    fn parse_items(input: &str) -> Result<Vector<EDN>, String> {
        let mut items = Vector::new();
        let mut buffer = String::new();
        let mut in_nested = 0;
        for ch in input.chars() {
            match ch {
                '(' | '[' | '{' => {
                    in_nested += 1;
                    buffer.push(ch);
                }
                ')' | ']' | '}' => {
                    in_nested -= 1;
                    buffer.push(ch);
                    if in_nested == 0 {
                        items.push_back(EDN::read_string(&buffer.trim())?);
                        buffer.clear();
                    }
                }
                ' ' if in_nested == 0 => {
                    if !buffer.is_empty() {
                        items.push_back(EDN::read_string(&buffer.trim())?);
                        buffer.clear();
                    }
                }
                _ => buffer.push(ch),
            }
        }
        if !buffer.is_empty() {
            items.push_back(EDN::read_string(&buffer.trim())?);
        }
        Ok(items)
    }

    fn split_items(input: &str) -> Result<Vector<String>, String> {
        let mut items = Vector::new();
        let mut buffer = String::new();
        let mut in_nested = 0;
        for ch in input.chars() {
            match ch {
                '(' | '[' | '{' => {
                    in_nested += 1;
                    buffer.push(ch);
                }
                ')' | ']' | '}' => {
                    in_nested -= 1;
                    buffer.push(ch);
                }
                ',' if in_nested == 0 => {
                    items.push_back(buffer.trim().to_string());
                    buffer.clear();
                }
                _ => buffer.push(ch),
            }
        }
        if !buffer.is_empty() {
            items.push_back(buffer.trim().to_string());
        }
        Ok(items)
    }
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
        }
    }
}

fn main() {
    let examples = vec![
        "(1 (2 3) 4)",
        "[1 2 [3 4] 5]",
        "{:a 1 :b 2}",
        "#{{1 2} {3 4}}",
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
        "(defn sum [a b] (+ a b))",
        "(-> a b c)",
        "(. Foo bar 1 2 3)",
        "(.. Foo (bar 1 2 3))",
    ];

    for example in examples {
        match EDN::read_string(example) {
            Ok(edn) => println!("Parsed: {} -> {:?}", example, edn),
            Err(e) => println!("Error: {} -> {}", example, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use imbl::Vector;

    #[test]
    fn test_read_string_simple_addition() {
        let input = "(+ 1 1)";
        let expected = EDN::List(Vector::from(vec![
            EDN::Symbol("+".to_string()),
            EDN::Integer(BigInt::from(1)),
            EDN::Integer(BigInt::from(1)),
        ]));
        
        match EDN::read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }

    #[test]
    fn test_read_string_nested_list() {
        let input = "(1 (2 3) 4)";
        let expected = EDN::List(Vector::from(vec![
            EDN::Integer(BigInt::from(1)),
            EDN::List(Vector::from(vec![
                EDN::Integer(BigInt::from(2)),
                EDN::Integer(BigInt::from(3)),
            ])),
            EDN::Integer(BigInt::from(4)),
        ]));
        
        match EDN::read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }

    #[test]
    fn test_read_string_vector() {
        let input = "[1 2 [3 4] 5]";
        let expected = EDN::Vector(Vector::from(vec![
            EDN::Integer(BigInt::from(1)),
            EDN::Integer(BigInt::from(2)),
            EDN::Vector(Vector::from(vec![
                EDN::Integer(BigInt::from(3)),
                EDN::Integer(BigInt::from(4)),
            ])),
            EDN::Integer(BigInt::from(5)),
        ]));
        
        match EDN::read_string(input) {
            Ok(parsed) => assert_eq!(parsed, expected),
            Err(e) => panic!("Failed to parse '{}': {}", input, e),
        }
    }

    // #[test]
    // fn test_read_string_map() {
    //     let input = "{:a 1 :b 2}";
    //     let mut expected_map = HashMap::new();
    //     expected_map.insert(EDN::Keyword("a".to_string()), EDN::Integer(BigInt::from(1)));
    //     expected_map.insert(EDN::Keyword("b".to_string()), EDN::Integer(BigInt::from(2)));
    //     let expected = EDN::Map(expected_map);
        
    //     match EDN::read_string(input) {
    //         Ok(parsed) => assert_eq!(parsed, expected),
    //         Err(e) => panic!("Failed to parse '{}': {}", input, e),
    //     }
    // }

    // #[test]
    // fn test_read_string_set() {
    //     let input = "#{{1 2} {3 4}}";
    //     let mut set1 = HashSet::new();
    //     set1.insert(EDN::Integer(BigInt::from(1)));
    //     set1.insert(EDN::Integer(BigInt::from(2)));

    //     let mut set2 = HashSet::new();
    //     set2.insert(EDN::Integer(BigInt::from(3)));
    //     set2.insert(EDN::Integer(BigInt::from(4)));

    //     let mut expected_set = HashSet::new();
    //     expected_set.insert(EDN::Set(set1));
    //     expected_set.insert(EDN::Set(set2));

    //     let expected = EDN::Set(expected_set);
        
    //     match EDN::read_string(input) {
    //         Ok(parsed) => assert_eq!(parsed, expected),
    //         Err(e) => panic!("Failed to parse '{}': {}", input, e),
    //     }
    // }
}
