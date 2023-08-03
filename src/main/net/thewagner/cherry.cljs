(ns net.thewagner.cherry
  (:require [goog.dom :as gdom]
            [reagent.dom :as rdom]
            [reagent.core :as r]
            [cljs.spec.alpha :as s]
            [clojure.test.check.generators :as gen]))

(s/def ::surfaces (s/coll-of ::surface))
(s/def ::surface (s/or ::ObjectOrImagePlane (s/keys {:req-un [:ObjectOrImagePlane/surface-type ::diam]})
                       ::RefractingCircularConic (s/keys {:req-un [:RefractingCircularConic/surface-type ::diam ::n ::roc ::k]})
                       ::RefractingCircularFlat (s/keys {:req-un [:RefractingCircularFlat/surface-type ::diam ::n]})))

(s/def :ObjectOrImagePlane/surface-type #{:ObjectOrImagePlane})
(s/def :RefractingCircularConic/surface-type #{:RefractingCircularConic})
(s/def :RefractingCircularFlat/surface-type #{:RefractingCircularFlat})

(s/def ::diam number?)
(s/def ::n number?)
(s/def ::roc number?)
(s/def ::k number?)

(def surface-types
  {:ObjectOrImagePlane {:display-name "Object/Image plane"}
   :RefractingCircularConic {:display-name "Conic"}
   :RefractingCircularFlat {:display-name "Flat"}})

(def parameters [:diam :n :roc :k])

(comment
  (map (partial ->externally-tagged :surface-type)
    (gen/sample (s/gen ::surface) 5))
  (gen/sample (s/gen number?) 5))

(defn vec-remove
  "remove elem in coll"
  [coll pos]
  (into (subvec coll 0 pos) (subvec coll (inc pos))))

(defn ->externally-tagged [tag v]
  (println tag v)
  {(get v tag) (dissoc v tag)})

(defn surface-dropdown [surface change-fn]
  [:div.select
    {:class (when-not surface :is-primary)}
    [:select
      {:on-change (fn [e] (change-fn (keyword (.-value (.-target e)))))}
      [:option {:disabled true
                :selected (when-not surface true)
                :value true
                :hidden true}
       "Select surface type"]
      (for [t (keys surface-types)]
        ^{:key t}
        [:option
          {:value t
           :selected (= t (:surface-type surface))}
          (get-in surface-types [t :display-name])])]])

(defn param-input [value change-fn]
  [:input.input
   {:value value
    :on-change #((when-let [value (parse-double (.-value (.-target %)))]
                   (change-fn value)))}])

(defn main []
  (let [surfaces (r/atom (into [] (gen/sample (s/gen ::surface) 3)))]
    (fn []
      [:<>
        [:section.section
          [:h1.title "Cherry table demo"]
          [:nav.level
            [:div.level-left
              [:div.level-item
                [:p.subtitle "Surfaces"]]]
            [:div.level-right
              [:p.level-item
                [:a.button {:on-click #(reset! surfaces (into [] (gen/sample (s/gen ::surface) 3)))} "I'm Feeling Lucky"]]
              [:p.level-item
                [:a.button.is-success {:on-click #(swap! surfaces conj nil)} "New"]]]]
          [:table.table.is-fullwidth
             [:thead
               [:tr
                 [:th "Surface type"]
                 (for [p parameters]
                   ^{:key p} [:th p])
                 [:th "Actions"]]]
             [:tbody
               (for [[i s] (map vector (range) @surfaces)]
                 ^{:key i}
                 [:tr
                   [:td [surface-dropdown s #(swap! surfaces assoc-in [i :surface-type] %)]]
                   (for [p parameters]
                     ^{:key p}
                     [:td
                       (if-let [v (get s p)]
                         [param-input v #(swap! surfaces assoc-in [i p] %)]
                         "-")])
                   [:td
                     [:a.button {:on-click #(swap! surfaces vec-remove i)} "Delete"]]])]]]
        [:section.section
          [:p "ClojureScript data"]
          [:pre
            [:code (with-out-str (cljs.pprint/pprint @surfaces))]]]
        [:section.section
          [:p [:a {:href "https://serde.rs/enum-representations.html#internally-tagged"} "Internally tagged"] " JSON representation"]
          [:pre
            [:code (-> @surfaces
                       (clj->js)
                       (js/JSON.stringify nil 2))]]]
        [:section.section
          [:p [:a {:href "https://serde.rs/enum-representations.html#externally-tagged"} "Externally tagged"] " JSON representation"]
          [:pre
            [:code (-> (map #(->externally-tagged :surface-type %) @surfaces)
                       (clj->js)
                       (js/JSON.stringify nil 2))]]]])))

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
