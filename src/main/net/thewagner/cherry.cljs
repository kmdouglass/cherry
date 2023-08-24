(ns net.thewagner.cherry
  (:require [goog.dom :as gdom]
            [reagent.core :as r]
            [reagent.dom.client :as rclient]
            [cljs.pprint :refer [pprint]]
            [cljs.spec.alpha :as s]
            [cljs.core.async :refer [take! go]]
            [cljs.core.async.interop :refer-macros [<p!]]
            [clojure.test.check.generators :as gen]
            [rendering]
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
          model (wasm-system-model (.-WasmSystemModel cherry) surfaces-and-gaps)
          surfaces (.surfaces model)
          gaps (.gaps model)
          results (.rayTrace model)
          surfaces-samples (let [num-samples 20]
                             (for [i (range (count surfaces))]
                               {:samples (js->clj (.sampleSurfYZ model i num-samples))}))

          valid-surfaces? (s/valid? ::cherry-spec/surface-samples surfaces-samples)
          valid-results? (s/valid? ::cherry-spec/raytrace-results
                                   (js->clj results {:keywordize-keys true}))]

      (cond->
        {:surfaces (js->clj surfaces)
         :gaps (js->clj gaps)}

        valid-surfaces?
        (assoc :surface-samples surfaces-samples)

        valid-results?
        (assoc :ray-samples (js->clj (rendering/resultsToRayPaths results)))

        (not valid-surfaces?)
        (assoc ::surface-sample-problems
               (::s/problems (s/explain-data ::cherry-spec/surface-samples surfaces-samples)))

        (not valid-results?)
        (assoc ::raytrace-results-problems
               (::s/problems (s/explain-data ::cherry-spec/raytrace-results
                                             (js->clj results {:keywordize-keys true}))))))))



(defn render [canvas surface-samples ray-samples]
  (when surface-samples
    (let [ctx (.getContext canvas "2d")
          w (.-width canvas)
          h (.-height canvas)
          _ (.clearRect ctx 0 0 w h)
          sf (rendering/scaleFactor (clj->js surface-samples) w h 0.8)
          comSamples (rendering/centerOfMass (clj->js surface-samples))
          canvasCenterCoords [(/ w 2.) (/ h 2.)]
          canvasSurfs (rendering/toCanvasCoordinates (clj->js surface-samples)
                                                     (clj->js comSamples)
                                                     (clj->js canvasCenterCoords)
                                                     sf)]
      (rendering/draw canvasSurfs ctx "black" 1.0)
      (when ray-samples
        (let [canvasRays (rendering/toCanvasCoordinates (clj->js ray-samples)
                                                        (clj->js comSamples)
                                                        (clj->js canvasCenterCoords)
                                                        sf)]
          (rendering/draw canvasRays ctx "red" 1.0))))))

(defn window-size []
  (r/with-let [size (r/atom nil)
               handler #(swap! size assoc
                               :innerWidth (.-innerWidth js/window)
                               :innerHeight (.-innerHeight js/window))
               _ (.addEventListener js/window "resize" handler false)]
    @size
    (finally
      (.removeEventListener js/window "resize" handler))))

(defn canvas-component [results]
  (let [canvas (atom nil)
        div (atom nil)
        did-mount? (atom false)]
    (r/create-class
      {:display-name `canvas-component
       :component-did-update
       (fn [_this]
         (when (and @did-mount? @results)
           (render @canvas (:surface-samples @results) (:ray-samples @results))))

       :component-did-mount
       (fn [_this]
         (reset! did-mount? true))

       :reagent-render
       (fn [results]
         @(r/track window-size)
         [:div.container
           {:ref #(reset! div %)}
           (when (and (:surface-samples @results) (not (:ray-samples @results)))
             [:div.notification.is-danger "No rays."])
           [:canvas#systemModel (cond-> {:ref #(reset! canvas %)}

                                        @div
                                        (assoc :width (.-clientWidth @div)
                                               :height (let [aspect 3]
                                                          (/ (.-clientWidth @div) aspect))))]])})))

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

(defn data-viewer-component [surfaces results]
  [:<>
    [:h2.subtitle "Surfaces and gaps from WasmSystemModel"]
    [:pre
      [:code (with-out-str (pprint @results))]]
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
                     (js/JSON.stringify nil 2))]]]]])

(defn results-viewer-component [rows results]
  (r/with-let [watch (r/track!
                       (fn [] (let [surfaces-and-gaps (map (comp row->surface-and-gap
                                                                 parameters-as-numbers)
                                                           @rows)]
                                (when (s/valid? (s/coll-of ::cherry-spec/surface-and-gap)
                                                surfaces-and-gaps)
                                  (take!
                                    (compute-results surfaces-and-gaps)
                                    #(reset! results %))))))]
    [canvas-component results]
    (finally
      (r/dispose! watch))))

(def planoconvex
  [(merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:n 1.515 :thickness 5.3 :diam 25.0 :roc 25.8 :k 0.0})
   (merge {:surface-type :RefractingCircularFlat}
          #::cherry-spec{:n 1.0 :thickness 46.6 :diam 25.0})])

(def petzval
  [(merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:n 1.5168, :thickness 13.0, :diam 56.956, :roc 99.56266, :k 0.0})
   (merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:diam 52.552, :roc -86.84002, :k 0.0 :n 1.6645, :thickness 4.0})
   (merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:n 1.0, :thickness 40.0 :diam 42.04, :roc -1187.63858, :k 0.0})
   (merge {:surface-type :RefractingCircularFlat}
          #::cherry-spec{:n 1.0, :thickness 40.0, :diam 33.262})
   (merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:n 1.6074, :thickness 12.0  :diam 41.086, :roc 57.47491, :k 0.0})
   (merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:n 1.6727, :thickness 3.0 :diam 40.148, :roc -54.61685, :k 0.0})
   (merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:n 1.0, :thickness 46.82210 :diam 32.984, :roc -614.68633, :k 0.0})
   (merge {:surface-type :RefractingCircularConic}
          #::cherry-spec{:n 1.6727, :thickness 2.0 :diam 34.594, :roc -38.17110, :k 0.0})
   (merge {:surface-type :RefractingCircularFlat}
          #::cherry-spec{:n 1.0, :thickness 1.87179 :diam 37.88})])

(defn surfaces-table [surfaces]
  [:<>
    [:nav.level
      [:div.level-left
         [:div.level-item
           [:p.subtitle "Surfaces"]]]
      [:div.level-right
        [:p.level-item
          [:button.button {:on-click #(reset! surfaces planoconvex)} "Planoconvex"]]
        [:p.level-item
          [:button.button {:on-click #(reset! surfaces petzval)} "Petzval"]]
        [:p.level-item
          [:button.button {:on-click #(reset! surfaces (into [] (gen/sample
                                                                  (gen/fmap
                                                                    (fn [r] (update r ::cherry-spec/diam (partial * 1)))
                                                                    (s/gen ::row)) 5)))}
           "I'm Feeling Lucky"]]
        [:p.level-item
          [:button.button.is-success {:on-click #(swap! surfaces conj nil)} "New"]]]]
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
               [:button.button {:on-click #(swap! surfaces vec-remove i)} "Delete"]]])]]])

(defonce surfaces (r/atom (vec (gen/sample (s/gen ::row) 3))))

(defn main []
  (r/with-let [results (r/atom nil)]
    [:<>
       [:section.section
         [:h1.title "Cherry table demo"]
         [results-viewer-component surfaces results]
         [surfaces-table surfaces]]
       [:section.section
         [data-viewer-component surfaces results]]]))

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
