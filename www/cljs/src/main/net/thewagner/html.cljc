(ns net.thewagner.html
  (:require [hiccup.core :refer [html]]
            [hiccup.page :refer [html5]]))

(def table
  (html
    [:nav.level
      [:div.level-left
        [:div.level-item
          [:p.subtitle "Surfaces"]]]
      [:div.level-right
        [:div.level-item
          [:div.field.is-grouped
            [:p.control
              [:button.button {:id :preset-planoconvex-button} "Planoconvex"]]
            [:p.control
              [:button.button {:id :preset-petzval-button} "Petzval"]]
            [:p.control
              [:button.button {:id :preset-random-button} "I'm Feeling Lucky"]]
            [:p.control
              [:button.button.is-success {:id :new-row-button} "New"]]]]]]
    [:div.table-container
      [:table.table {:id "surfaces-table"}
         [:thead
           [:tr
             [:th {:style "width: 14%"} "Surface type"]
             [:th {:style "width: 14%"} "n"]
             [:th {:style "width: 14%"} "thickness"]
             [:th {:style "width: 14%"} "diam"]
             [:th {:style "width: 14%"} "roc"]
             [:th {:style "width: 14%"} "k"]
             [:th {:style "width: 14%"} "Actions"]]]
         [:tbody]]]))

(def index
  (html5 {:lang "en"}
    [:head
      [:meta {:charset "UTF-8"}]
      [:meta {:name "viewport" :content "width=device-width, initial-scale=1"}]
      [:meta {:http-equiv "x-ua-compatible" :content "ie=edge"}]
      [:title "🍒 Cherry Raytracer"]
      [:link {:rel "stylesheet" :href "./cherry.css"}]]
    [:body
      [:section.section
        [:h1.title "Cherry Raytracer"]
        [:div.container
          [:canvas#systemModel]]
        [:div.container
          table]]
      [:script {:deferred true :src "./index.js"}]
      [:script {:deferred true :src "./main.js"}]]))

(comment
  (spit "build/index.html" index))
