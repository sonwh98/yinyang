use yinyang::clojure::read_string;
use yinyang::core::*;

fn main() {
    let s = slurp("add.clj").unwrap();
    let ast = read_string(&s).unwrap();
    println!("{:?}", ast);
}
