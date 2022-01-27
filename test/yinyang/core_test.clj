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

  (testing "lambda expressions"
    (is (= 9 (eval2 '((lambda [x] (* x x)) 3)
                    {'* *})))

    (let [sq (eval2 '(lambda [x] (* x x))
                    {'* *})]
      (is (= 9 (eval2 '(square 3) {'square sq})))))
  
  )
