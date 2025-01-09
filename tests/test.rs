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
use yinyang::clojure::read_string;
use yinyang::clojure::EDN;

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;

    #[test]
    fn test_read_string() {
        // Test for Nil
        assert_eq!(read_string("nil").unwrap(), EDN::Nil);

        // Test for Boolean
        assert_eq!(read_string("true").unwrap(), EDN::Bool(true));
        assert_eq!(read_string("false").unwrap(), EDN::Bool(false));

        // Test for Integer
        assert_eq!(read_string("42").unwrap(), EDN::Integer(BigInt::from(42)));

        // Test for Float
        assert_eq!(
            read_string("3.14").unwrap(),
            EDN::Float(BigDecimal::from_str("3.14").unwrap())
        );

        // Test for Keyword
        assert_eq!(
            read_string(":keyword").unwrap(),
            EDN::Keyword(":keyword".to_string())
        );

        // Test for String
        assert_eq!(
            read_string("\"hello\"").unwrap(),
            EDN::String("hello".to_string())
        );

        // Test for Symbol
        // assert_eq!(
        //     read_string("symbol").unwrap(),
        //     EDN::Symbol("symbol".to_string())
        // );
    }

    // #[test]
    // fn test_collection_parsing() {
    //     // Test for List
    //     assert_eq!(
    //         read_string("(1 2 3)").unwrap(),
    //         EDN::List(vec![
    //             EDN::Integer(BigInt::from(1)),
    //             EDN::Integer(BigInt::from(2)),
    //             EDN::Integer(BigInt::from(3)),
    //         ])
    //     );

    //     // Test for Vector
    //     assert_eq!(
    //         read_string("[1 2 3]").unwrap(),
    //         EDN::Vector(vec![
    //             EDN::Integer(BigInt::from(1)),
    //             EDN::Integer(BigInt::from(2)),
    //             EDN::Integer(BigInt::from(3)),
    //         ])
    //     );

    //     // Test for Map
    //     let mut expected_map = HashMap::new();
    //     expected_map.insert(
    //         EDN::Keyword(":key".to_string()),
    //         EDN::String("value".to_string()),
    //     );
    //     assert_eq!(
    //         read_string("{:key \"value\"}").unwrap(),
    //         EDN::Map(expected_map)
    //     );
    // }

    // #[test]
    // fn test_set_parsing() {
    //     let input = "#{1 (2 [3 4] 5)}";

    //     let result = read_string(input);
    //     assert!(result.is_ok());

    //     if let Ok(EDN::Set(set)) = result {
    //         assert_eq!(set.len(), 2);

    //         let mut inner_vec = Vec::new();
    //         inner_vec.push(EDN::Integer(3.into()));
    //         inner_vec.push(EDN::Integer(4.into()));

    //         let mut inner_list = Vec::new();
    //         inner_list.push(EDN::Integer(2.into()));
    //         inner_list.push(EDN::Vector(inner_vec));
    //         inner_list.push(EDN::Integer(5.into()));

    //         let mut hset = HashSet::new();
    //         hset.insert(EDN::Integer(1.into()));
    //         hset.insert(EDN::List(inner_list));

    //         assert_eq!(hset, set);
    //     } else {
    //         panic!("Expected Set");
    //     }
    // }
}
