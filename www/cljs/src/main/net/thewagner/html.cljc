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

(defn tabs-nav [active]
  (letfn [(current [t] (if (= t active) {:class :is-active} {}))]
    [:ul
      [:li (current :surfaces) [:a#surfaces "Surfaces"]]
      [:li (current :fields) [:a#fields "Fields"]]
      [:li (current :aperture) [:a#aperture "Aperture"]]]))

(defn tabs-body [active]
  (case active
    :surfaces [:div.container table-nav table]
    :aperture [:div.container
                [:div.field.is-horizontal
                  [:div.field-label.is-normal
                    [:label.label "Entrance pupil diameter"]]
                  [:div.field-body
                    [:div.field
                      [:p.control
                        [:input.input {:disabled true :type "text" :value "10"}]]]]]]
    :fields   [:div.container
                [:div.columns.is-centered
                  [:div.column.is-half
                    [:table.table.is-fullwidth
                      [:tr
                        [:th "#"]
                        [:th "Angle"]]
                      [:tr
                        [:th "1"]
                        [:td "0"]]
                      [:tr
                        [:th "2"]
                        [:td "5"]]]]]]))

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
     [:div#system-parameters.tabs.is-centered
       (tabs-nav :surfaces)]
     [:div#tab-body
       (tabs-body :surfaces)]]
   [:script {:deferred true :src "./index.js"}]
   [:script {:deferred true :src "./main.js"}]])
