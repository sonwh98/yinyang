use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use num_bigint::BigInt;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
//use std::str::FromStr;

#[derive(Debug)]
enum EDN {
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
                l.hash(state);
            }
            EDN::Vector(v) => {
                state.write_u8(8);
                v.hash(state);
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
                for v in s {
                    v.hash(state);
                }
            }
        }
    }
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn main() {
    // EDN Nil example
    let nil_example = EDN::Nil;

    // EDN Boolean example
    let bool_example = EDN::Bool(true);

    // EDN Integer example
    let b = BigInt::from(i64::MAX) +1;
    let int_example = EDN::Integer(b);

    // EDN Float example
    //let f = BigDecimal::from(f64::MAX)+1;
    //let dec = BigDecimal::from_str(&input).unwrap();
    let dec = BigDecimal::from_f64(f64::MAX).unwrap();
    let float_example = EDN::Float(dec);

    // EDN String example
    let string_example = EDN::String("Hello, EDN!".to_string());

    // EDN Symbol example
    let symbol_example = EDN::Symbol("my-symbol".to_string());

    // EDN Keyword example
    let keyword_example = EDN::Keyword(":my-keyword".to_string());

    // EDN List example
    let list_example = EDN::List(vec![
        EDN::Integer(BigInt::from(1)),
        EDN::Integer(BigInt::from(2)),
        EDN::Integer(BigInt::from(2147483648i64)),
    ]);

    // EDN Vector example
    let vector_example = EDN::Vector(vec![EDN::Bool(false), EDN::String("vector".to_string())]);

    // EDN Map example
    let mut map = HashMap::new();
    map.insert(
        EDN::Keyword(":key".to_string()),
        EDN::String("value".to_string()),
    );
    let map_example = EDN::Map(map);

    // EDN Set example
    let mut set = HashSet::new();
    set.insert(EDN::Integer(BigInt::from(1)));
    set.insert(EDN::Integer(BigInt::from(2)));
    let set_example = EDN::Set(set);

    // Printing examples
    println!("{:?}", nil_example);
    println!("{:?}", bool_example);
    println!("{:?}", int_example);
    println!("{:?} {:?}", float_example, float_example);
    println!("{:?}", string_example);
    println!("{:?}", symbol_example);
    println!("{:?}", keyword_example);
    println!("{:?}", list_example);
    println!("{:?}", vector_example);

    println!("{:?}", map_example);
    println!("{:?}", set_example);
[}
