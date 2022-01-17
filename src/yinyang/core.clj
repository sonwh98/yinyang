(ns yinyang.core
  (:require [clojure.pprint :as pp]
            [taoensso.timbre :as log])
  (:gen-class))

(declare eval2)

(defmacro apply2 [s-ex env]
  `(let [foo# (prn {:apply2-s-ex ~s-ex})
         s-ex# (map #(eval2 % ~env) ~s-ex)
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
  (prn {:eval2-s-ex s-ex})
  (cond
    (symbol? s-ex)              (env s-ex)
    (and (list? s-ex)
         (let [f (first s-ex)]
           (= f 'lambda)))      (let [params (second s-ex)
                                      body (drop 2 s-ex)
                                      body (conj body 'do)]
                                  (prn {:params params
                                        :body body})
                                  (fn [args]
                                    (eval2 body (fn [y]
                                                  (if (= params y)
                                                    args
                                                    (env y))))))    
    (list? s-ex) (apply2 s-ex env)
    :else s-ex))

(comment
  (require '[taoensso.timbre.appenders.core :as appenders])
  (log/merge-config! {:min-level :info
                      :middleware [(fn [data]
                                     (update data :vargs (partial mapv #(if (string? %)
                                                                          %
                                                                          (with-out-str (pp/pprint %))))))]
                      :appenders {:println {:enabled? false}
                                  :catalog (merge (appenders/spit-appender {:fname (let [log-dir (or (System/getenv "LOG_DIR") ".")]
                                                                                     (str  log-dir "/debug.log"))})
                                                  {:min-level :info
                                                   :level :info})}})
  
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
                   (prn {:x x})
                   (* x x)
                   ) 2) {'x 2
                         '* *
                         'prn prn})

  (eval2 '(* x x) {'x 2
                   '* *})

  ({'x 2} 'x)
  
  (apply2 '(inc 2))
  (take-last 2 [1 2 3 4 5])
  (conj 1 '(2 3))
  (conj '(1) 2)



  )
