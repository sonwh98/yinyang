mod clojure;

use log::{debug, error, info, trace, warn};

use yinyang::clojure::repl;

fn main() {
    env_logger::init();
    repl();
}
