# yinyang

Clojure implemented in Clojure. This is a science experiment with the ultimate goal of implementing Clojure in Rust.


## Usage


    $ lein repl
 
## Options

## Rust Usage

	$ cargo run --bin clj
	
## Examples
```clojure

(eval2 '(* 3 (+ 1 2 3)) {})

(eval2 '((lambda [x y]
                 (* x y)) 5 3 )
         {'* *})

```

...

### Bugs

...

### Any Other Sections
### That You Think
### Might be Useful

## License

Copyright © 2022 FIXME

This program and the accompanying materials are made available under the
terms of the Eclipse Public License 2.0 which is available at
http://www.eclipse.org/legal/epl-2.0.

This Source Code may also be made available under the following Secondary
Licenses when the conditions for such availability set forth in the Eclipse
Public License, v. 2.0 are satisfied: GNU General Public License as published by
the Free Software Foundation, either version 2 of the License, or (at your
option) any later version, with the GNU Classpath Exception which is available
at https://www.gnu.org/software/classpath/license.html.
