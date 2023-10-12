(ns net.thewagner.html)

(def table-nav
  [:nav.level {:id :surfaces-table-nav}
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
            [:button.button.is-success {:id :new-row-button} "New"]]]]]])

(def table
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
       [:tbody]]])

(def cherry-raytracer "🍒 Cherry Raytracer")

(def navbar
  [:nav.navbar {:role "navigation" :aria-label "main navigation"}
    [:div.navbar-brand
      [:a.navbar-item {:href "https://browser.science"} cherry-raytracer]]])

(def index-head
  [:head
      [:meta {:charset "UTF-8"}]
      [:meta {:name "viewport" :content "width=device-width, initial-scale=1"}]
      [:meta {:http-equiv "x-ua-compatible" :content "ie=edge"}]
      [:title "🍒 Cherry Raytracer"]
      [:link {:rel "stylesheet" :href "./cherry.css"}]])

(def index-body
 [:body
   navbar
   [:section.section
     [:div.container
       [:canvas#systemModel]]
     [:div.container
       table-nav
       table]]
   [:script {:deferred true :src "./index.js"}]
   [:script {:deferred true :src "./main.js"}]])
