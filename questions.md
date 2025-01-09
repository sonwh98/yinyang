* How do I install Clojure and tools?
  - https://clojure.org/guides/getting_started 
  - IDE 
    - emacs 
    - Cursive 
    - Calva 
	
* What two important idea that people should take away about Clojure?
  - Think in terms of data values
  - Think in terms of transformations of data values instead of mutation of data
  - https://lambdakids.stigmergy.systems/2018/8/14/data-transformation.blog
  
* What is a value?
  - immutable
  - the thing that a function returns
  
* what is the basic syntax of a LISP?
  - (+ 1 2 3)
  
* What is the REPL?
  - Read Eval Print Loop
  
* What are the core data types Clojure?
  - numbers 
  - strings
  - keywords
  - collections 
    - map
    - set
    - sequences
      - list
      - vectors
    - lazy collections
  - class 
    - deftype 
    - defrecord
  - protocols

* Does Clojure have variables? 
  - not in the traditional sense
  - the closes thing to a variable is the clojure atom
  - symbol binding

* Does Clojure have loops?
  - no
  - use recursion
  - loop recur
  
* What is a function?

* What is a pure function?

* What is a closure?

* What are Clojure built-in data types?
  - maps
  - vectors
  - sets
  - numbers
  - strings

* What is an s-expression?
  - abstract syntax for data encoding 
  - nested parenthesis ()
  - Examples: XML/HTML, JSON, EDN

* What is EDN?
  - EDN is Clojure's form of s-expressions
  - Data format that is more expressive than JSON

* What is an expression?
  - Something that evaluates to a value
  
* What is the Clojure Reader?
  - Its a parser that turns strings into data EDN structures

* What is an AST?
* What is a macro?
* What is homoiconicity?
* What is Eval? 
  - It is a function that evaluates s-expressions
  - eval takes an expression and returns a value
* What is apply?
  - apply function to list of values
  - calling a function on list of values
* Does Clojure have statements?
  - No. Everything in Clojure is an expression.
  
* What is the differences between a statement and an expression?


* Does Clojure have polymorphism?
  * yes. 
  * can also dispatch on data not just on types like in Java
