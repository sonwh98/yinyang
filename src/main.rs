use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
#[allow(dead_code)]
enum EDN {
    Nil,
    Bool(bool),
    Integer(BigInt),
    Float(f64),
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
            (EDN::Float(f1), EDN::Float(f2)) => f1.to_bits() == f2.to_bits(), // Use to_bits for bitwise comparison
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
                // Hash a bitwise representation of the float
                // Not the value itself to avoid NaN issues
                state.write(&f.to_bits().to_ne_bytes());
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

#[allow(dead_code)]
#[allow(unused)]
fn main() {
    let i = BigInt::from(2147483);
    println!("i={:?}", i);

    // EDN Nil example
    let nil_example = EDN::Nil;

    // EDN Boolean example
    let bool_example = EDN::Bool(true);

    // EDN Integer example
    let int_example = EDN::Integer(BigInt::from(10));

    // EDN Float example
    let float_example = EDN::Float(3.14);

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
    println!("{:?}", float_example);
    println!("{:?}", string_example);
    println!("{:?}", symbol_example);
    println!("{:?}", keyword_example);
    println!("{:?}", list_example);
    println!("{:?}", vector_example);

    println!("{:?}", map_example);
    println!("{:?}", set_example);
}

// fn vec_char_to_string(v: &Vec<char>) -> String {
//     //v.iter().collect()
//     v.into_iter().collect()
// }

// fn helper(stack: &mut Vec<char>, chars: &mut std::str::Chars) {
//     loop {
//         match chars.next() {
//             Some('(') => {
// 		let mut new_stack = Vec::<char>::new();
//                 new_stack.push('(');
//                 println!("start sex {:?}", new_stack);
// 		helper(&mut new_stack, chars);
//             }
//             Some(')') => {
//                 stack.push(')');
//                 let sexpression = vec_char_to_string(&stack);
// 		println!("end stack {:?}", stack);
//                 println!("end sex {:?}", sexpression);
//             }
//             Some(c) => {
// 		stack.push(c);
// 		println!("stack ={:?}", stack);
// 	    }
// 	    ,
//             None => break,
//         }
//     }
// }

// fn parse(input: &str) -> Vec<char> {
//     let mut stack = Vec::<char>::new();
//     helper(&mut stack, &mut input.chars());
//     return stack;
// }

// fn main() {
//     //let mut stack: Vec<char> = Vec::new();
//     let s: SExpr = SExpr::Atom("Foo".to_string());
//     let l = SExpr::List(vec![
// 	SExpr::Atom("+".to_string()),
// 	SExpr::Atom("1".to_string()),
// 	SExpr::Atom("2".to_string()),
//     ]);

//     println!("s={:?}",s);
//     println!("l={:?}",l);

//     // let input = "(+ 10 21 (* 2 30))";
//     // println!("{:?}", parse(input));

//     // // Check the size of the stack
//     // println!("Stack size: {}", stack.len()); // Output: Stack size: 5

//     // // Peek at the top element without removing it
//     // if let Some(top) = stack.last() {
//     // 	println!("Top element: {:?}", top); // Output: Top element: Character('b')
//     // }

//     // // Pop elements off the stack
//     // while let Some(ch) = stack.pop() {
//     // 	println!("Popped other character: {}", ch);
//     // }

//     // // Check if the stack is empty
//     // println!("Stack is empty: {}", stack.is_empty()); // Output: Stack is empty: true
// }
