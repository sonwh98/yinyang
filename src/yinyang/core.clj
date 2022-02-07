(ns yinyang.core
  (:require [clojure.pprint :as pp]
            [clojure.string :as string]
            [taoensso.timbre :as log]
            [taoensso.timbre.appenders.core :as appenders]
            [yinyang.pred :as p])
  (:gen-class))

(def global-env (atom {'* *
                       '+ +
                       '/ /
                       '- -
                       'prn prn
                       '= =
                       }))

(declare eval2)

(defmacro apply2 [s-ex env]
  `(let [s-ex# (map #(eval2 % ~env) ~s-ex)
         f# (first s-ex#)
         args# (rest s-ex#)
         args-count# (count args#)]
     (log/debug {:apply2-s-ex s-ex#
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

(defn ns-parts [a-ns]
  (let [ns-parts (string/split (str a-ns) #"\.")]
    (mapv symbol ns-parts)))

(defn eval2 [s-ex env]
  (log/debug {:eval2-s-ex s-ex
              :env env
              :global-env @global-env})
  (cond
    (p/lambda? s-ex) (let [params (second s-ex)
                           body (drop 2 s-ex)
                           body (conj body 'do)]
                       (fn [& args]
                         (eval2 body (fn [a-symbol]
                                       (let [v (env a-symbol)]
                                         (log/debug {:params params
                                                     :a-symbol a-symbol
                                                     :a-symbol-v v
                                                     :args args})
                                         (if v
                                           v
                                           (let [pairs (mapv vec (partition 2 (interleave params args)))
                                                 env2 (into {} pairs)]
                                             (env2 a-symbol))))))))
    (p/ns? s-ex)     (let [a-ns (second s-ex)
                           ns-parts (ns-parts a-ns)
                           ns-val (get-in @global-env ns-parts)]
                       (when-not ns-val
                         (swap! global-env assoc-in ns-parts {})
                         )
                       
                       (swap! global-env assoc '*ns* a-ns)
                       (log/debug {:a-ns a-ns
                                   :ns-val ns-val
                                   :ns-parts ns-parts }))
    (set? s-ex)      (set (map #(eval2 % env) s-ex))
    (vector? s-ex)   (mapv #(eval2 % env) s-ex)
    (map? s-ex)      (into {} (for [[k v] s-ex]
                                [(eval2 k env)
                                 (eval2 v env)]))
    (p/quote? s-ex)  (first (rest s-ex))
    (p/do? s-ex)     (let [do-body (rest s-ex)
                           last-ex (last do-body)
                           ex-but-last (drop-last do-body)]
                       (doseq [ex ex-but-last]
                         (eval2 ex env))
                       (log/debug {:do-body do-body
                                   :ex-but-last ex-but-last
                                   :last-ex last-ex})
                       (eval2 last-ex env))
    (p/let? s-ex)    (let [bindings (second s-ex)
                           pairs (mapv vec (partition 2 bindings))
                           env2 (into {} pairs)
                           body (drop 2 s-ex)
                           implicit-do (conj body 'do)]
                       (log/debug {:bindings bindings
                                   :do implicit-do})
                       (eval2 implicit-do (merge env env2)))
    (symbol? s-ex)   (or (env s-ex)
                         (let [current-ns (@global-env '*ns*)
                               ns-path (ns-parts current-ns)
                               s-path (conj ns-path s-ex)
                               s-val (or (get-in @global-env s-path)
                                         (get-in @global-env [s-ex]))]
                           (log/debug {:ns-path ns-path
                                       :s-path s-path
                                       :s-val s-val
                                       :global-env global-env})
                           
                           s-val))
    (p/def? s-ex)    (let [[_ s v] s-ex
                           this-ns (@global-env '*ns*)
                           v (eval2 v env)]
                       (if this-ns
                         (let [ns-path (ns-parts this-ns)]
                           (swap! global-env update-in ns-path
                                  (fn [current-ns]
                                    (assoc current-ns s v))))
                         (swap! global-env assoc s v))
                       v)
    (p/defn? s-ex)    (let [[_ fn-name fn-param & body] s-ex
                            lambda (concat '(lambda)
                                           [fn-param]
                                           body)
                            def-lambda (concat '(def)
                                               [fn-name]
                                               [lambda])]
                        (log/debug {:def-lambda def-lambda})
                        (eval2 def-lambda env))
    (p/if? s-ex)     (let [[_ test branch1 branch2] s-ex]
                       (log/debug {:test test
                                   :b1 branch1
                                   :b2 branch2})
                       (if (eval2 test env)
                         (eval2 branch1 env)
                         (eval2 branch2 env)))
    (p/defmacro? s-ex)  (let [[_ macro-name macro-params & macro-body] s-ex
                              mac (concat '(defn) [macro-name macro-params] macro-body)]
                          ;;macro-params are not evaluated

                          (log/info {:macro-name macro-name
                                     :params macro-params
                                     :mac mac})
                          
                          )
    (seq? s-ex)      (apply2 s-ex env)
    :else            s-ex))

(defn text->forms
  "parse text into clojure s-expressions. returns a vector of clojure s-expression forms"
  [txt]
  (loop [char-seq (seq txt)
         level 0
         buffer nil
         forms []]
    (let [c (first char-seq)]
      (cond
        (= c \()            (recur (rest char-seq)
                                   (inc level)
                                   (str buffer c)
                                   forms)
        (= c \))            (recur (rest char-seq)
                                   (dec level)
                                   (str buffer c)
                                   forms)
        (nil? c)            forms
        (and (zero? level)
             (nil? buffer)) (recur (rest char-seq)
                                   level
                                   buffer
                                   forms)
        (zero? level)       (let [form (read-string buffer)]
                              (recur (rest char-seq)
                                     level
                                     nil
                                     (conj forms form)))

        :else               (recur (rest char-seq)
                                   level
                                   (str buffer c)
                                   forms)))))
(defn load-file2 [file-name]
  (let [forms (-> file-name slurp text->forms)]
    (doseq [f forms]
      (eval2 f {}))))

(defn config-log [level]
  (log/merge-config! {:min-level level
                      :middleware [(fn [data]
                                     (update data :vargs
                                             (partial mapv #(if (string? %)
                                                              %
                                                              (with-out-str (pp/pprint %))))))]
                      :appenders {:println {:enabled? false}
                                  :catalog (merge (appenders/spit-appender
                                                   {:fname (let [log-dir (or (System/getenv "LOG_DIR") ".")]
                                                             (str  log-dir "/debug.log"))})
                                                  {:min-level level
                                                   :level level})}}))

(comment
  (config-log :info)
  (config-log :debug)
  (log/set-level! :info)
  (log/spy :info (* 2 2))
  (load-file2 "src/yinyang/fib.clj")

  (eval2 '(sq 2) {})
  (eval2 '(cube 2) {})

  (eval2 '(if false 1 0) {})

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
  (type `(+ 1 1))

  (first `(+ 1 1))
  (type '(+ 1 1))
  (type `(fn []
           (list + 1 1)))
  (type (cons 1 '(2)))
  (cons '(1) 2)
  (cons 1 2)
  (eval2 '(defmacro infix [s-ex] (bar 1 2 3)) {})
  )
