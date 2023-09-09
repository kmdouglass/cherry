(ns net.thewagner.html)

(def index
  [:html
    [:head
      [:meta {:charset "UTF-8"}]
      [:meta {:name "viewport" :content "width=device-width, initial-scale=1"}]
      [:meta {:http-equiv "x-ua-compatible" :content "ie=edge"}]
      [:title "🍒 Cherry Raytracer"]
      [:link {:rel "stylesheet" :href "css/cherry.css"}]]
    [:body
      [:div#app]
      [:script {:src "./index.js"}]
      [:script {:src "./main.js"}]]])

(comment
  (require '[hiccup.core :refer [html]])
  (spit "build/index.html" (html index)))
