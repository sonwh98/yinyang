(ns yinyang.core
  (:require [clojure.pprint :as pp :refer :all]
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
                       'first first
                       'second second
                       'last last
                       'list list}))

(declare eval2)

(defmacro apply2 [s-ex env]
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

(defn ns-parts [a-ns]
  (let [ns-parts (string/split (str a-ns) #"\.")]
    (mapv symbol ns-parts)))

(defn eval2 [s-ex env]
  (log/debug {:eval2-s-ex s-ex
              :env env
              :global-env @global-env})
  (cond
    (symbol? s-ex)     (or (env s-ex)
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
    (set? s-ex)        (set (map #(eval2 % env) s-ex))
    (vector? s-ex)     (mapv #(eval2 % env) s-ex)
    (map? s-ex)        (into {} (for [[k v] s-ex]
                                  [(eval2 k env)
                                   (eval2 v env)]))
    (p/lambda? s-ex)   (let [params (second s-ex)
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
    (p/quote? s-ex)    (first (rest s-ex))
    (p/do? s-ex)       (let [do-body (rest s-ex)
                             last-ex (last do-body)
                             ex-but-last (drop-last do-body)]
                         (doseq [ex ex-but-last]
                           (eval2 ex env))
                         (log/debug {:do-body do-body
                                     :ex-but-last ex-but-last
                                     :last-ex last-ex})
                         (eval2 last-ex env))
    (p/let? s-ex)      (let [bindings (second s-ex)
                             pairs (mapv vec (partition 2 bindings))
                             env2 (into {} pairs)
                             body (drop 2 s-ex)
                             implicit-do (conj body 'do)]
                         (log/debug {:bindings bindings
                                     :do implicit-do})
                         (eval2 implicit-do (merge env env2)))
    (p/defn? s-ex)     (let [[_ fn-name fn-param & body] s-ex
                             lambda (concat '(lambda)
                                            [fn-param]
                                            body)
                             def-lambda (concat '(def)
                                                [fn-name]
                                                [lambda])]
                         (log/debug {:def-lambda def-lambda})
                         (eval2 def-lambda env))
    (p/def? s-ex)      (let [[_ s v] s-ex
                             this-ns (@global-env '*ns*)
                             v (eval2 v env)]
                         (if this-ns
                           (let [ns-path (ns-parts this-ns)]
                             (swap! global-env update-in ns-path
                                    (fn [current-ns]
                                      (assoc current-ns s v))))
                           (swap! global-env assoc s v))
                         v)

    (p/if? s-ex)       (let [[_ test branch1 branch2] s-ex]
                         (log/debug {:test test
                                     :b1 branch1
                                     :b2 branch2})
                         (if (eval2 test env)
                           (eval2 branch1 env)
                           (eval2 branch2 env)))
    (p/defmacro? s-ex) (let [[_ macro-name macro-params & macro-body] s-ex
                             l (concat '(lambda) [macro-params] macro-body)
                             l (with-meta (eval2 l env) {:macro true})
                             l (concat '(def) [macro-name l] )]
                         (eval2 l env))
    (p/ns? s-ex)       (let [a-ns (second s-ex)
                             ns-parts (ns-parts a-ns)
                             ns-val (get-in @global-env ns-parts)]
                         (when-not ns-val
                           (swap! global-env assoc-in ns-parts {}))

                         (swap! global-env assoc '*ns* a-ns)
                         (log/debug {:a-ns a-ns
                                     :ns-val ns-val
                                     :ns-parts ns-parts}))
    (seq? s-ex)        (let [f (first s-ex)
                             f-val (eval2 f env)
                             f-val-meta (meta f-val)
                             macro? (:macro f-val-meta)
                             s-ex (if macro?
                                    (let [param (rest s-ex)
                                          param (concat '(quote) param)]
                                      (concat '() [f]  (list param)))
                                    s-ex)]
                         (if macro?
                           (let [s-ex2 (apply2 s-ex env)]
                             (eval2 s-ex2 env))
                           (apply2 s-ex env)))
    :else              s-ex))

(defn text->forms
  "parse text into clojure s-expressions. returns a vector of clojure s-expression forms"
  [txt]
  (loop [char-seq  (seq txt) #_(let [char-seq (seq txt)
                             last-char (last char-seq)]
                         (if (= last-char \n)
                           char-seq
                           (concat char-seq [\n])))
         level 0
         buffer nil
         forms []
         reader-macro? false]
    (let [c (first char-seq)]
      (log/info {:txt txt
                 :char-seq char-seq
                 :c c
                 :level level
                 :buffer buffer
                 :forms forms})
      (cond
        (= c \#)            (recur (rest char-seq)
                                   level
                                   buffer
                                   forms
                                   true)
        
        (= c \()            (recur (rest char-seq)
                                   (inc level)
                                   (str buffer c)
                                   forms
                                   reader-macro?)
        (= c \))            (recur (rest char-seq)
                                   (dec level)
                                   (str buffer c)
                                   forms
                                   reader-macro?)
        (nil? c)        (conj forms (read-string buffer))
        (and (zero? level)
             (nil? buffer)) (recur (rest char-seq)
                                   level
                                   buffer
                                   forms
                                   reader-macro?)
        (zero? level)       (let [form (read-string buffer)]
                              (recur (rest char-seq)
                                       level
                                       nil
                                       (conj forms form)
                                       reader-macro?))

        :else               (recur (rest char-seq)
                                   level
                                   (str buffer c)
                                   forms
                                   reader-macro?)))))
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

(defn clj->wat [forms]
  (mapcat (fn [form]
            (cond
              (p/defn? form) (let [f-name# (nth form 1)
                                   f-name-meta# (meta f-name#)
                                   _ (prn :meta f-name-meta#)
                                   $f-name# (str "$" f-name#)
                                   $f-name-sym# (symbol $f-name#)
                                   params# (nth form 2)
                                   params# (map (fn [p]
                                                  (concat '(param) [(symbol (str "$" p))] ['i32]))
                                                params#)
                                   body (drop 3 form)
                                   _ (prn "body" body)
                                   exports (if (:export f-name-meta#)
                                             (concat ['export (str f-name#)]
                                                     [(list 'func $f-name-sym#)]))
                                   result (concat '(result) [(or (:tag f-name-meta#) 'i32)])]
                               (remove nil? (list (concat '(func)
                                                          `( ~$f-name-sym# ~@params#)
                                                          [result]
                                                          body)
                                                  exports))
                               )
              )
            ) forms)
  )

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
                   (* x y)) 5 3)
         {'* *})

  (eval2 '(* x x) {'x 2})

  (eval2 '((lambda [x]
                   (* x x)) 2)
         {})

  (eval2 '(prn {:x1 x}) {'x 3
                         'prn prn})
  (eval2 '(do (* x x)
              (* 2 x))
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
  (def forms (-> "example/math.clj" slurp text->forms))
  (def forms (-> "example/sex.clj" slurp text->forms))

  (def forms (-> "(+ 1 1)" text->forms))
  (concat (seq "(comment ") (seq "+ 1 2)"))
  (def forms (text->forms "(+ 1 1) #_(+ 1)"))
  (text->forms "(comment (comment (+ 1)))")
  (text->forms "(comment (+ 1))")
  
  (def w (clj->wat (:forms forms)))
  (pprint w)
  (-> w first (nth 2) meta)
  (read-string "(defn  ^:export 1)")
  (clojure.edn/read-string "(1 #^ )")
  (def foo (read-string "#^String x"))
  (def foo (read-string "(defn #^String x)"))
  (def foo (read-string "(defn #^:export x)"))
  (def foo (read-string "(defn ^:export ^Integer ^String ^:bar add [a b] (+ a b))"))
  (-> foo (nth 1) meta)

  (into ['(1 2 3)] [[4]])

  [
   (
    (func $add (param $a i32) (param $b i32))
    
    (export "add" (func "$add"))
    )
   ]

  [
   ((func $add (param $a i32) (param $b i32))
    (export "add" (func $add)))
   
   ((func $inc (param $a i32)))]

  (defn ^:export ^Integer add [^String a ^String b]
    1)

  (defn ^Integer ^:export  add [^String a ^String b]
    1)
  )
