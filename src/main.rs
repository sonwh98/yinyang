mod clojure;

fn main() {
    //read_string_example();
    //eval_examples();
    let v = clojure::read_string("(:a :b :c)");
    println!("v={:?}", v);
    //let input = "(def a \"1 2 3\")";
    //read_string(input);
}
