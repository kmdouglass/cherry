(ns net.thewagner.html)

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
      [:a.navbar-item {:href "https://browser.science"} cherry-raytracer]
      [:a.navbar-burger {:role "button" :aria-label "menu" :aria-expanded "false"
                         :data-target :navMenu}
        [:span {:aria-hidden true}]
        [:span {:aria-hidden true}]
        [:span {:aria-hidden true}]]]
    [:div#navMenu.navbar-menu
      [:div.navbar-start
        [:div.navbar-item.has-dropdown.is-hoverable
          [:a.navbar-link "File"]
          [:div.navbar-dropdown
            [:a#file-save.navbar-item "Save"]]]
        [:div.navbar-item.has-dropdown.is-hoverable
          [:a.navbar-link "Examples"]
          [:div.navbar-dropdown
            [:a#preset-planoconvex.navbar-item "Planoconvex lens"]
            [:a#preset-petzval.navbar-item "Petzval objective"]]]]]])

(defn tabs-nav [active]
  (letfn [(current [t] (if (= t active) {:class :is-active} {}))]
    [:ul
      [:li (current :surfaces) [:a#surfaces "Surfaces"]]
      [:li (current :fields) [:a#fields "Fields"]]
      [:li (current :aperture) [:a#aperture "Aperture"]]]))

(defn tabs-body [active & {:keys [default-value] :or {default-value 10}}]
  (case active
    :surfaces [:div.container table]
    :aperture [:div.container
                [:div.field.is-horizontal
                  [:div.field-label.is-normal
                    [:label.label "Entrance pupil diameter"]]
                  [:div.field-body
                    [:div.field
                      [:p.control
                        [:input#aperture-input.input {:type "text" :value (str default-value)}]]]]]]
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
     [:div#systemRendering.container]
     [:div#system-parameters.tabs.is-centered
       (tabs-nav :surfaces)]
     [:div#tab-body
       (tabs-body :surfaces)]]
   [:script {:deferred true :src "./index.js"}]
   [:script {:deferred true :src "./main.js"}]])
