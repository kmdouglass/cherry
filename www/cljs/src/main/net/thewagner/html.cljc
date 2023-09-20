(ns net.thewagner.html
  (:require [hiccup.page :refer [html5]]))

(def index
  (html5 {:lang "en"}
    [:head
      [:meta {:charset "UTF-8"}]
      [:meta {:name "viewport" :content "width=device-width, initial-scale=1"}]
      [:meta {:http-equiv "x-ua-compatible" :content "ie=edge"}]
      [:title "🍒 Cherry Raytracer"]
      [:link {:rel "stylesheet" :href "css/cherry.css"}]]
    [:body
      [:div#app]
      [:script {:src "./index.js"}]
      [:script {:src "./main.js"}]]))

(comment
  (spit "build/index.html" index))
