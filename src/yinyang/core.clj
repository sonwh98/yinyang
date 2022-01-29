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

(defn pred-helper [s-ex pred]
  (and (seq? s-ex)
       (let [f (first s-ex)]
         (pred f))))

(defn quote? [s-ex]
  (pred-helper s-ex #(= 'quote %)))

(defn do? [s-ex]
  (pred-helper s-ex #(= 'do %)))

(defn lambda? [s-ex]
  (pred-helper s-ex #(or (= 'lambda %)
                         (= 'fn %))))

(defn let? [s-ex]
  (pred-helper s-ex #(= 'let %)))

(defn def? [s-ex]
  (pred-helper s-ex #(= 'def %)))

(def global-env (atom {'* *
                       '+ +
                       '/ /
                       '- -}))

(defn eval2 [s-ex env]
  (log/info {:eval2-s-ex s-ex
             :env env})
  (cond
    (set? s-ex)    (set (map #(eval2 % env) s-ex))
    (vector? s-ex) (mapv #(eval2 % env) s-ex)
    (map? s-ex)    (into {} (for [[k v] s-ex]
                              [(eval2 k env)
                               (eval2 v env)]))
    (quote? s-ex) (first (rest s-ex))
    (do? s-ex)    (let [do-body (rest s-ex)
                        last-ex (last do-body)
                        ex-but-last (drop-last do-body)]
                    (map #(eval2 % env) ex-but-last)
                    (log/info {:do-body do-body
                               :ex-but-last ex-but-last
                               :last-ex last-ex})
                    (eval2 last-ex env))
    (let? s-ex)     (let [bindings (second s-ex)
                          pairs (mapv vec (partition 2 bindings))
                          env2 (into {} pairs)
                          body (drop 2 s-ex)
                          implicit-do (conj body 'do)]
                      (log/info {:bindings bindings
                                 :do implicit-do})
                      (eval2 implicit-do (merge env env2)))
    (symbol? s-ex) (let [v (or (env s-ex)
                               (@global-env s-ex))]
                     (log/info {:s s-ex
                                :v v})
                     v)
    (def? s-ex)    (let [[d s v] s-ex]
                     (swap! global-env (fn [global-env]
                                         (assoc global-env s v)))
                     v)
    (lambda? s-ex) (let [params (second s-ex)
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
  (eval2 ''(1 2 3) {}) ;;; (1 2 3)
  (eval2 '(+ 2  3) {})
  (eval2 '(+ 2 (+ 1 1 1)) {})
  (eval2 '(+ 2  3 4) {})
  (eval2 '(* 3 (+ 1 2 3)) {})
  (eval2 '(inc 2) {'inc inc})
  

  (eval2 '(lambda [x] (* x x)) {})
  (eval2 '((lambda [x] (* x x)) 4) {'* *})

  (eval2 '((lambda [x y]
                   (* x y)) 5 3 )
         {'* *})

  
  (eval2 '(* x x) {'x 2})
  
  (eval2 '((lambda [x]
                   (* x x)) 2)
         {})

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
  (eval2 '(def pi 3.141) {})
  )
