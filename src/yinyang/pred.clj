(ns yinyang.pred)

(defn- pred-helper [s-ex pred]
  (and (seq? s-ex)
       (let [f (first s-ex)]
         (pred f))))

(defn ns? [s-ex]
  (pred-helper s-ex #(= 'ns %)))

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
