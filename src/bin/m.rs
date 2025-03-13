use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::str::FromStr;
use yinyang::clojure::{eval, read_string, Value};
use yinyang::core::register_native_fn;
use yinyang::edn::*;
use yinyang::immutant::list::*;

fn main() {
    //let mut env = HashMap::new();

    let a_sexp = List::singleton(EDN::Symbol("def".to_string()))
        .append(EDN::Symbol("pi".to_string()))
        .append(EDN::Float(BigDecimal::from_str("3.14").unwrap()));
    //println!("a={:?}",a_sexp);
    //println!("a2={:?}",a_sexp.reverse());

    // let def_expr = EDN::List(Box::new(a_sexp));

    // let a_var = eval(def_expr, &mut env).unwrap();
}
