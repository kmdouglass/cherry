(ns net.thewagner.cherry
  (:require [goog.dom :as gdom]
            [goog.string :as gstring]
            [goog.string.format]
            [reagent.dom :as rdom]
            [reagent.core :as r]))

(defn vec-remove
  "remove elem in coll"
  [coll pos]
  (into (subvec coll 0 pos) (subvec coll (inc pos))))

(defn surface-dropdown [surface change-fn]
  [:div.select
    {:class (when-not surface :is-primary)}
    [:select
      {:on-change (fn [e] (change-fn (.-value (.-target e))))}
      [:option {:disabled true
                :selected (when-not surface true)
                :value true
                :hidden true}
       "Select surface type"]
      (for [t ["glass" "air" "metal"]]
        ^{:key t}
        [:option {:value t :selected (= t (:surface-type surface))} t])]])

(defn main []
  (let [surfaces (r/atom [{:surface-type "glass" :p1 1 :p2 2}
                          {:surface-type "air"   :p1 10 :p2 20}
                          {:surface-type "metal" :p1 10 :p3 30}])]
    (fn []
      [:section.section
        [:h1.title "Cherry table demo"]
        [:nav.level
          [:div.level-left
            [:div.level-item
              [:p.subtitle "Surfaces"]]]
          [:div.level-right
            [:p.level-item
              [:a.button.is-success {:on-click #(swap! surfaces conj nil)} "New"]]]]
        [:table.table.is-fullwidth
           [:thead
             [:tr
               [:th "Surface type"]
               [:th "p1"]
               [:th "p2"]
               [:th "p3"]
               [:th "Actions"]]]
           [:tbody
             (for [[i s] (map vector (range) @surfaces)]
               ^{:key i}
               [:tr
                 [:td [surface-dropdown s #(swap! surfaces assoc-in [i :surface-type] %)]]
                 (for [p [:p1 :p2 :p3]]
                   ^{:key p}
                   [:td
                     (if-let [v (get s p)]
                       v
                       "-")])
                 [:td
                   [:a.button {:on-click #(swap! surfaces vec-remove i)} "Delete"]]])]]
       [:section.section
        [:pre
          [:code (js/JSON.stringify (clj->js @surfaces) nil 2)]]]])))

(defn mount []
  (rdom/render [main] (gdom/getElement "app")))

(defn ^:dev/after-load on-reload []
  (mount))

(defonce startup (mount))

(comment
  ; Evaluate these lines to enter into a ClojureScript REPL
  (require '[shadow.cljs.devtools.api :as shadow])
  (shadow/repl :app)
  ; Exit the CLJS session
  :cljs/quit)
