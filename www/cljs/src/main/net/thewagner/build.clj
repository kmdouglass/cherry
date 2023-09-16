(ns net.thewagner.build
  (:require [hiccup.core :refer [html]]
            [net.thewagner.html :as pages]))

(defn hook
  {:shadow.build/stage :flush}
  [{:keys [shadow.build/config] :as build-state} & _args]
  (spit (str (:output-dir config) "/index.html") (html pages/index))
  build-state)
