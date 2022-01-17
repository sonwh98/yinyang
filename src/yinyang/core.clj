(ns yinyang.core
  (:require [clojure.pprint :as pp])
  (:gen-class))

(declare eval2)

(defmacro apply2 [s-ex env]
  (prn {:s-ex s-ex})
  `(let [s-ex# (map #(eval2 % ~env) ~s-ex)
         f# (first s-ex#)
         args# (rest s-ex#)
         args-count# (count args#)]

     (case args-count#
       1 (f# (nth args# 0))
       2 (f# (nth args# 0)
             (nth args# 1))
       3 (f# (nth args# 0)
             (nth args# 1)
             (nth args# 2))
       
       4 (f# (nth args# 0)
             (nth args# 1)
             (nth args# 2)
             (nth args# 3))
       (.applyTo f# args#))))

(defn self-eval? [s-ex]
  (or (number? s-ex)
      (and
       (list? s-ex)
       (not (-> s-ex first symbol?)))))

(defn eval2 [s-ex env]

  (cond
    (symbol? s-ex)              (env s-ex)
    (and (list? s-ex)
         (let [f (first s-ex)]
           (= f 'lambda)))      (let [[x] (second s-ex)
                                      body (last s-ex)]
                                  (prn {
                                        :body body})
                                  (fn [args]
                                    
                                    (eval2 body (fn [y]
                                                  (if (= x y)
                                                    args
                                                    (env y))))))    
    (list? s-ex) (do
                   (prn {:s-ex s-ex})
                   (apply2 s-ex env))
    :else (do
            (prn {:bar s-ex})
            s-ex)))

(comment
  (self-eval? '(1 2 3))
  (self-eval? '(+ 2 (+ 1 1 1)))
  
  (eval2 '(1 2 3))
  (eval2 '(+ 2  3))
  (eval2 '(+ 2 (+ 1 1 1)))
  (eval2 '(+ 2  3 4))
  (eval2 '(* 3 (+ 1 2 3)) {})
  (eval2 '(inc 2) {})
  

  (eval2 '(lambda [x] (* x x)) {})
  (eval2 '((lambda [x]
                   (* x x)
                   ) 2) {'x 2
                         '* *})

  (eval2 '(* x x) {'x 2
                   '* *})

  ({'x 2} 'x)
  
  (apply2 '(inc 2))
  )
