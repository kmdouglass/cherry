(ns net.thewagner.cherry
  (:require [goog.dom :as gdom]
            [reagent.dom.client :as rclient]
            [reagent.core :as r]
            [cljs.pprint :refer [pprint]]
            [cljs.spec.alpha :as s]
            [cljs.core.async :refer [take! go]]
            [cljs.core.async.interop :refer-macros [<p!]]
            [clojure.test.check.generators :as gen]
            [cherry :as cherry-async]
            [kmdouglass.cherry :as cherry-spec]))

(defmulti row-type :surface-type)
(defmethod row-type :RefractingCircularFlat [_]
  (s/merge ::cherry-spec/RefractingCircularFlat ::cherry-spec/gap))

(defmethod row-type :RefractingCircularConic [_]
  (s/merge ::cherry-spec/RefractingCircularConic ::cherry-spec/gap))

(s/def ::row (s/multi-spec row-type :surface-type))

(def surface-types
  {:RefractingCircularConic {:display-name "Conic"
                             :default #::cherry-spec{:n 1 :thickness 1 :diam 1 :roc 1 :k 1}}
   :RefractingCircularFlat {:display-name "Flat"
                             :default #::cherry-spec{:n 1 :thickness 1 :diam 1}}})

(def parameters [::cherry-spec/n ::cherry-spec/thickness ::cherry-spec/diam ::cherry-spec/roc ::cherry-spec/k])

(comment
  (gen/sample (s/gen number?) 5)
  (gen/sample (s/gen ::row) 5))

(defn vec-remove
  "remove elem in coll"
  [coll pos]
  (into (subvec coll 0 pos) (subvec coll (inc pos))))

(defn parameters-as-numbers [row]
  (update-vals row (fn [v] (if (string? v) (parse-double v) v))))

(defn row->surface-and-gap [row]
 (vector
   {(:surface-type row) (dissoc row :surface-type ::cherry-spec/thickness ::cherry-spec/n)}
   (select-keys row [::cherry-spec/thickness ::cherry-spec/n])))

(s/fdef row->surface-and-gap
  :args (s/cat :row ::row)
  :ret ::cherry-spec/surface-and-gap)

(comment
  (s/exercise-fn `row->surface-and-gap) 100)

(defn wasm-system-model [constructor surfaces-and-gaps]
  (let [m (new constructor)]
    (doseq [[i [s g]] (map vector (range) surfaces-and-gaps)]
       (.insertSurfaceAndGap m (inc i) (clj->js s) (clj->js g)))
    m))

(set! *warn-on-infer* false)
(defn compute-results [surfaces-and-gaps]
  (go
    (let [cherry (<p! cherry-async)
          model (wasm-system-model (.-WasmSystemModel cherry) surfaces-and-gaps)]
      {:surfaces (js->clj (.surfaces model))
       :gaps (js->clj (.gaps model))})))

(defn prefill-row [old selected]
  (let [kw (keyword selected)
        default (get-in surface-types [kw :default])]
   (merge
     default
     (select-keys old (keys default))
     {:surface-type kw})))

(defn surface-dropdown [surface change-fn]
  [:div.select
    {:class (when-not surface :is-primary)}
    [:select
      {:value (or (:surface-type surface) ::default)
       :on-change #(change-fn (.. % -target -value))}
      [:option {:disabled true :value ::default :hidden true}
               "Select surface type"]
      (for [t (keys surface-types)]
        ^{:key t}
        [:option {:value t}
                 (get-in surface-types [t :display-name])])]])

(defn param-input [spec value change-fn]
  (let [valid? (s/valid? spec (parse-double (str @value)))]
    [(if valid? :input.input :input.input.is-warning)
     {:value @value
      :on-change #(change-fn (.. % -target -value))}]))

(defn data-viewer-component [surfaces]
  [:div.columns
    [:div.column
      [:h2.subtitle "Table as ClojureScript data"]
      [:pre
        [:code (with-out-str (pprint @surfaces))]]]
    [:div.column
      [:h2.subtitle "Surfaces and gaps"]
      [:pre
        [:code (-> (map row->surface-and-gap @surfaces)
                   (clj->js)
                   (js/JSON.stringify nil 2))]]]])

(defn results-viewer-component [rows]
  (r/with-let [result (r/atom [])
               watch (r/track!
                       (fn [] (let [surfaces-and-gaps (map (comp row->surface-and-gap
                                                                 parameters-as-numbers)
                                                           @rows)]
                                (when (s/valid? (s/coll-of ::cherry-spec/surface-and-gap)
                                                surfaces-and-gaps)
                                  (take!
                                    (compute-results surfaces-and-gaps)
                                    #(reset! result %))))))]
    [:<>
      [:pre
        [:code (with-out-str (pprint @result))]]]
    (finally
      (r/dispose! watch))))

(defn surfaces-table [surfaces]
  [:<>
    [:nav.level
      [:div.level-left
         [:div.level-item
           [:p.subtitle "Surfaces"]]]
      [:div.level-right
        [:p.level-item
          [:a.button {:on-click #(reset! surfaces (into [] (gen/sample (s/gen ::row) 3)))} "I'm Feeling Lucky"]]
        [:p.level-item
          [:a.button.is-success {:on-click #(swap! surfaces conj nil)} "New"]]]]
    [:table.table.is-fullwidth
       [:thead
         [:tr
           [:th "Surface type"]
           (for [p parameters]
             ^{:key p} [:th {:style {:width "12%"}} p])
           [:th "Actions"]]]
       [:tbody
         (for [[i s] (map vector (range) @surfaces)]
           ^{:key i}
           [:tr
             [:td [surface-dropdown s (fn [selected-type]
                                        (swap! surfaces update i #(prefill-row % selected-type)))]]
             (for [p parameters]
               ^{:key p}
               [:td
                 (when (get s p)
                   [param-input p
                                (r/track #(get-in @surfaces [i p]))
                                #(swap! surfaces assoc-in [i p] %)])])
             [:td
               [:a.button {:on-click #(swap! surfaces vec-remove i)} "Delete"]]])]]])

(defn main []
  (r/with-let [surfaces (r/atom (vec (gen/sample (s/gen ::row) 3)))]
    [:<>
       [:section.section
         [:h1.title "Cherry table demo"]
         [surfaces-table surfaces]]
       [:section.section
         [:h2.subtitle "Surfaces and gaps from WasmSystemModel"]
         [results-viewer-component surfaces]]
       [:section.section
         [data-viewer-component surfaces]]]))

(defonce dom-root
   (rclient/create-root (gdom/getElement "app")))

; https://code.thheller.com/blog/shadow-cljs/2019/08/25/hot-reload-in-clojurescript.html
(defn ^:dev/after-load start []
  (rclient/render dom-root [main]))

(defn init []
  (start))

(comment
  ; Evaluate these lines to enter into a ClojureScript REPL
  (require '[shadow.cljs.devtools.api :as shadow])
  (shadow/repl :app)
  ; Exit the CLJS session
  :cljs/quit)
