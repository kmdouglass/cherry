; Based on https://github.com/clojure/clojurescript/blob/master/samples/twitterbuzz/src/twitterbuzz/dom-helpers.cljs
(ns science.browser.cherry.dom
  (:require [clojure.string :as string]
            [goog.dom :as dom]))

(defn get-element
  "Return the element with the passed id."
  [id]
  (dom/getElement (name id)))

(defn append
  "Append all children to parent."
  [parent & children]
  (doseq [child children]
    (dom/appendChild parent child))
  parent)

(defn set-text
  "Set the text content for the passed element returning the
  element. If a keyword is passed in the place of e, the element with
  that id will be used and returned."
  [e s]
  (let [e (if (keyword? e) (get-element e) e)]
    (doto e (dom/setTextContent s))))

(defn normalize-args [tag args]
  (let [parts (string/split (name tag) #"(\.|#)")
        [tag attrs] [(first parts)
                     (apply hash-map (map #(cond (= % ".") :class
                                                 (= % "#") :id
                                                 :else %)
                                          (rest parts)))]]
    (if (map? (first args))
      [tag (merge attrs (first args)) (rest args)]
      [tag attrs args])))

(defn element
  "Create a dom element using a keyword for the element name and a map
  for the attributes. Append all children to parent. If the first
  child is a string then the string will be set as the text content of
  the parent and all remaining children will be appended."
  [tag & args]
  (let [[tag attrs children] (normalize-args tag args)
        parent (dom/createDom (name tag) (clj->js attrs))
        [parent children] (if (string? (first children))
                            [(set-text (element tag attrs) (first children))
                             (rest children)]
                            [parent children])]
    (apply append parent children)))

(defn text [t]
  (dom/createTextNode t))

(defn remove-children
  "Remove all children from the element."
  [parent]
  (dom/removeChildren parent)
  parent)

(defn- element-arg? [x]
  (or (keyword? x)
      (map? x)
      (string? x)))

(defn build
  "Build up a dom element from nested vectors."
  [x]
  (if (vector? x)
    (let [[parent children] (if (keyword? (first x))
                              [(apply element (take-while element-arg? x))
                               (drop-while element-arg? x)]
                              [(first x) (rest x)])
          children (map build children)]
      (apply append parent children))
    x))
