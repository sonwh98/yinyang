mod clojure;

use std::collections::{HashMap, HashSet};

fn main() {
    //read_string_example();
    //eval_examples();
    //let v = clojure::read_string("(:a :b :c)");
    //let v = clojure::read_string("(1 [2 3])");
    //let v = clojure::read_string("(1 (2 3))");
    //let v = clojure::read_string("[1 (2 3 [4 (5)] (6 7))]");
    //let v = clojure::read_string("#{1 2 [3 4] (5 6)}");
    //let v = clojure::read_string("#{1 2 #{3 4} (5 6)}");
    //let v = clojure::read_string("[1 2 [3 4] #{5 6}]");
    //let v = clojure::read_string("(1 2 [3 4] #{5 6 (7 8)} [9 10]})");
    let v = clojure::read_string("{:a 1 :c [2 3 (4 (5)) ]}");
    println!("v={:?}", v);

    // let vec = vec![1, 2, 3, 4];
    // let map = clojure::vec_to_map(vec);

    // for (key, value) in &map {
    //     println!("{}: {}", key, value);
    // }

    //let input = "(def a \"1 2 3\")";
    //read_string(input);
}
