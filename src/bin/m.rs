use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::str::FromStr;
use yinyang::clojure::{eval, read_string, Value};
use yinyang::core::register_native_fn;
use yinyang::edn::*;
use yinyang::immutant::list::*;

fn main() {
    let s = "[1\n2\n3 4]";
    let v: Vec<&str> = s.split_whitespace().collect();
    println!("{:?}", v);
}
