(ns net.thewagner.build
  (:require [net.thewagner.html :as pages]))

(defn hook
  {:shadow.build/stage :flush}
  [{:keys [shadow.build/config] :as build-state} & _args]
  (spit (str (:output-dir config) "/../index.html") pages/index)
  build-state)
