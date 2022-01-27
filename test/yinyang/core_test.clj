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
           (eval2 '{:name "foo" :age 1} {}))))

  (testing "arithmatic expressions"
    (is (= 5 (eval2 '(+ 2  3)
                    {'+ +})))
    (is (= 8 (eval2 '(+ 2 (* 2 3))
                    {'+ +
                     '* *}))))
  
  )
