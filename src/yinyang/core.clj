(ns yinyang.core
  (:require [clojure.pprint :as pp]
            [taoensso.timbre :as log])
  (:gen-class))

(declare eval2)

(defmacro apply2 [s-ex env]
  `(let [;;foo# (log/info {:apply2-s-ex ~s-ex})
         s-ex# (map #(eval2 % ~env) ~s-ex)
         f# (first s-ex#)
         args# (rest s-ex#)
         args-count# (count args#)]
     (log/info {:apply2-s-ex s-ex#
                :apply2-args args#})
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

(defn eval2 [s-ex env]
  (log/info {:eval2-s-ex s-ex
             :env env})
  (cond
    (symbol? s-ex)              (let [v (env s-ex)]
                                  (log/info {:s s-ex
                                             :v v})
                                  v)
    (vector? s-ex)              (mapv #(eval2 % env) s-ex)
    (map? s-ex)                 (into {} (for [[k v] s-ex]
                                           [(eval2 k env)
                                            (eval2 v env)]))
    (and (seq? s-ex)
         (let [f (first s-ex)]
           (= f 'do)))          (let [do-body (rest s-ex)
                                      last-ex (last do-body)
                                      ex-but-last (drop-last do-body)]
                                  (map #(eval2 % env) ex-but-last)
                                  (log/info {:do-body do-body
                                             :ex-but-last ex-but-last
                                             :last-ex last-ex})
                                  (eval2 last-ex env))
    (and (seq? s-ex)
         (let [f (first s-ex)]
           (or (= f 'lambda)
               (= f 'fn))))      (let [params (second s-ex)
                                      body (drop 2 s-ex)
                                      body (conj body 'do)]
                                  (fn [& args]
                                    (eval2 body (fn [a-symbol]
                                                  (let [v (env a-symbol)]
                                                    (log/info {:params params
                                                               :a-symbol a-symbol
                                                               :a-symbol-v v
                                                               :args args})
                                                    (if v
                                                      v
                                                      (let [pairs (mapv vec (partition 2 (interleave params args)))
                                                            env2 (into {} pairs)]
                                                        (env2 a-symbol))))))))
    (seq? s-ex) (apply2 s-ex env)
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
  
  (eval2 '(1 2 3) {}) ;;error
  (eval2 '(+ 2  3) {'+ +})
  (eval2 '(+ 2 (+ 1 1 1)))
  (eval2 '(+ 2  3 4))
  (eval2 '(* 3 (+ 1 2 3)) {})
  (eval2 '(inc 2) {})
  

  (eval2 '(lambda [x] (* x x)) {})
  (eval2 '((lambda [x] (* x x)) 4) {'* *})

  (eval2 '((lambda [x y]
                   (* x y)) 5 3 )
         {'* *})

  
  (eval2 '(* x x) {'x 2
                   '* *})
  
  (eval2 '((lambda [x]
                   (* x x)) 2)
         {
          'prn prn})

  (eval2 '(prn {:x1 x}) {'x 3
                         'prn prn})
  (eval2 '(do (* x x)
              (* 2 x)
              )
         {'x 4
          '* *})
  (eval2 '(do 2) {})
  
  (eval2 '{:x x} {'x 1})
  (eval2 '[x] {'x 2})

  (seq? [1])
  
  ({'x 2} 'x)
  
  (apply2 '(inc 2))
  (take-last 2 [1 2 3 4 5])
  (conj 1 '(2 3))
  (conj '(1) 2)



  )
