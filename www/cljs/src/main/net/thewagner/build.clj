(ns net.thewagner.build
  (:require [hiccup.page :refer [html5]]
            [net.thewagner.html :as pages]))

(defn hook
  {:shadow.build/stage :flush}
  [{:keys [shadow.build/config] :as build-state} & _args]
  (spit (str (:output-dir config) "/../index.html") (html5 pages/index-head pages/index-body))
  build-state)
