(ns yinyang.core-test
  (:require [clojure.test :refer :all]
            [yinyang.core :refer :all]))

(deftest a-test
  (testing "eval data structures"
    (is (= '(1 2 3)
           (eval2 ''(1 2 3) {})))
    (is (= [1 2 3]
           (eval2 '[1 2 3] {})))
    (is (= {:name "foo" :age 1}
           (eval2 '{:name "foo" :age 1} {})))
    (is (= #{1 :a "b"}
           (eval2 '#{1 :a "b"} {}))))

  (testing "arithmatic expressions"
    (is (= 5 (eval2 '(+ 2  3)
                    {'+ +})))
    (is (= 8 (eval2 '(+ 2 (* 2 3))
                    {'+ +
                     '* *}))))

  (testing "lambda expressions"
    (is (= 9 (eval2 '((lambda [x] (* x x)) 3)
                    {'* *})))

    (let [sq (eval2 '(fn [x] (* x x))
                    {'* *})]
      (is (= 9 (eval2 '(square 3) {'square sq})))))

  (testing "let"
    (is (= 7
           (eval2 '(let [x 2
                         y 3]
                     (* x y)
                     (+ x y 2))
                  {'* *
                   '+ +}))))

  (testing "def"
    (let [pi (eval2 '(def pi 3.14) {})]
      (is (= 3.14 pi))
      (is (= 3.14 (@global-env 'pi))))

    (let [sq (eval2 '(def sq (lambda [x] (* x x)))
                    {})]
      (is (= 4 (eval2 '(sq 2) {})))))

  (testing "defn"
    (let [fib (eval2 '(defn fib [x]
                        (if (= x 0)
                          0
                          (if (= x 1)
                            1
                            (+ (fib (- x 1))
                               (fib (- x 2)))))) {})]
      (is (fn? fib))
      (is (= 0 (eval2 '(fib 0) {})))
      (is (= 1 (eval2 '(fib 1) {})))
      (is (= 34 (eval2 '(fib 9) {})))))

  #_(testing "text->forms"
    (let [sexp "(+ 1 1)"
          comment "#_(+ 1 1)"]
      (is (= '{:forms [(+ 1 1)], :reader-macro-forms [(+ 1 1)]}
             (text->forms (str sexp "\n" comment))))))
  
  (testing "defmacro"
    (let [dm '(defmacro infix
                 [infixed]
                (list (second infixed) (first infixed) (last infixed)))]
      (is (fn? (eval2 dm {})))
      (is  (= 5
              (eval2 '(infix (2 + 3)) {})))))
  )
