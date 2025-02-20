use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use yinyang::clojure::{eval, read_string, register_native_fn, Value, EDN};

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;

    #[test]
    fn test_single_value_parsing() {
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

        //Test for Symbol
        assert_eq!(
            read_string("symbol").unwrap(),
            EDN::Symbol("symbol".to_string())
        );
    }

    #[test]
    fn test_list_parsing() {
        assert_eq!(
            read_string("(1 2 3)").unwrap(),
            EDN::List(vec![
                EDN::Integer(BigInt::from(1)),
                EDN::Integer(BigInt::from(2)),
                EDN::Integer(BigInt::from(3)),
            ])
        );
    }

    #[test]
    fn test_vector_parsing() {
        assert_eq!(
            read_string("[1 2 3]").unwrap(),
            EDN::Vector(vec![
                EDN::Integer(BigInt::from(1)),
                EDN::Integer(BigInt::from(2)),
                EDN::Integer(BigInt::from(3)),
            ])
        );
    }

    #[test]
    fn test_map_parsing() {
        // Test for Map
        let mut expected_map = HashMap::new();
        expected_map.insert(
            EDN::Keyword(":key".to_string()),
            EDN::String("value".to_string()),
        );
        assert_eq!(
            read_string("{:key \"value\"}").unwrap(),
            EDN::Map(expected_map)
        );
    }

    #[test]
    fn test_set_parsing() {
        let input = "#{1 (2 [3 4] 5)}";

        let result = read_string(input);
        assert!(result.is_ok());

        if let Ok(EDN::Set(set)) = result {
            assert_eq!(set.len(), 2);

            let mut inner_vec = Vec::new();
            inner_vec.push(EDN::Integer(3.into()));
            inner_vec.push(EDN::Integer(4.into()));

            let mut inner_list = Vec::new();
            inner_list.push(EDN::Integer(2.into()));
            inner_list.push(EDN::Vector(inner_vec));
            inner_list.push(EDN::Integer(5.into()));

            let mut hset = HashSet::new();
            hset.insert(EDN::Integer(1.into()));
            hset.insert(EDN::List(inner_list));

            assert_eq!(hset, set);
        } else {
            panic!("Expected Set");
        }
    }
    #[test]
    fn test_nested_string_in_collection() {
        let rs = read_string("(\"[1]\")");
        let v = EDN::List(vec![EDN::String("[1]".to_string())]);
        assert_eq!(v, rs.unwrap());
    }

    #[test]
    fn test_special_form_quote() {
        let mut env = HashMap::new();
        let ast = read_string("(quote a)").unwrap();
        let a = eval(ast, &mut env).unwrap();
        assert_eq!(Value::EDN(EDN::Symbol("a".to_string())), a);

        let ast2 = read_string("'a").unwrap();
        let a2 = eval(ast2, &mut env).unwrap();
        assert_eq!(Value::EDN(EDN::Symbol("a".to_string())), a2);
    }

    #[test]
    fn test_special_form_do() {
        let mut env = HashMap::new();
        let ast = read_string("(do 1 2 3)").unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(Value::EDN(EDN::Integer(BigInt::from(3))), result);
    }

    #[test]
    fn test_special_form_if() {
        let mut env = HashMap::new();
        let ast = read_string("(if true 1 2)").unwrap();
        let result = eval(ast, &mut env).unwrap();
        assert_eq!(Value::EDN(EDN::Integer(BigInt::from(1))), result);
    }

    #[test]
    fn test_special_form_def() {
        let mut env = HashMap::new();

        let def_expr = EDN::List(vec![
            EDN::Symbol("def".to_string()),
            EDN::Symbol("pi".to_string()),
            EDN::Float(BigDecimal::from_str("3.14").unwrap()),
        ]);

        let a_var = eval(def_expr, &mut env).unwrap();

        assert!(matches!(
            a_var,
            Value::Var {
                ns: _,
                name: _,
                value: _
            }
        ));

        if let Value::Var {
            ref ns,
            ref name,
            ref value,
        } = a_var
        {
            assert_eq!(ns, "user");
            assert_eq!(name, "pi");
            if let Value::EDN(ref edn) = **value {
                if let EDN::Float(pi) = edn {
                    assert_eq!(*pi, BigDecimal::from_str("3.14").unwrap());
                }
            }

            assert!(env.contains_key("pi"));
            if let Some(Value::EDN(EDN::Float(val))) = env.get("pi") {
                assert_eq!(val, &BigDecimal::from_str("3.14").unwrap());
            } else {
                panic!("Expected pi to be bound to float 3.14");
            }
        }
    }

    #[test]
    fn test_special_form_let() {
        let mut env = HashMap::new();

        let let_expr = read_string("(let [pi 3.14] pi)").unwrap();
        let result = eval(let_expr, &mut env).unwrap();

        assert_eq!(
            result,
            Value::EDN(EDN::Float(BigDecimal::from_str("3.14").unwrap()))
        );

        // Verify binding was local
        assert!(env.get("pi").is_none());
    }

    #[test]
    #[allow(unused_variables)]
    fn test_call_lambda() {
        let mut env = HashMap::new();
        let ast = read_string("(def one (fn [] 1))").unwrap();
        let one_fn = eval(ast, &mut env).unwrap();
        let call_one = read_string("(one)").unwrap();
        let r = eval(call_one, &mut env).unwrap();
        if let Value::EDN(edn) = r {
            match edn {
                EDN::Integer(i) => {
                    assert_eq!(BigInt::from(1), i);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_call_native() {
        let mut env = HashMap::new();

        let echo: fn(Vec<Value>) -> Result<Value, String> = |args: Vec<Value>| {
            if args.len() != 1 {
                return Err("Expected exactly one argument".to_string());
            }
            Ok(args[0].clone())
        };

        register_native_fn(&mut env, "echo", echo);

        let ast = read_string("(echo 123)").unwrap();
        let result = eval(ast, &mut env).unwrap();

        if let Value::EDN(EDN::Integer(i)) = result {
            assert_eq!(BigInt::from(123), i);
        } else {
            panic!("Expected result to be integer 123");
        }
    }
}
