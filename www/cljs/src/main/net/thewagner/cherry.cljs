(ns net.thewagner.cherry
  (:require [goog.dom :as gdom]
            [reagent.core :as r]
            [reagent.dom.client :as rclient]
            [cljs.pprint :refer [pprint]]
            [cljs.spec.alpha :as s]
            [cljs.spec.gen.alpha :as gen]
            [cljs.core.async :refer [take! go]]
            [cljs.core.async.interop :refer-macros [<p!]]
            [cljs.reader :refer [read-string]]
            [rendering]
            [cherry :as cherry-async]
            [kmdouglass.cherry :as cherry-spec]
            [clojure.test.check.generators]))

(defmulti row-type :surface-type)
(defmethod row-type ::cherry-spec/RefractingCircularFlat [_]
  (s/merge ::cherry-spec/RefractingCircularFlat ::cherry-spec/gap))

(defmethod row-type ::cherry-spec/RefractingCircularConic [_]
  (s/merge ::cherry-spec/RefractingCircularConic ::cherry-spec/gap))

(defmethod row-type ::cherry-spec/Stop [_]
  (s/merge ::cherry-spec/Stop ::cherry-spec/gap))

(s/def ::row (s/multi-spec row-type :surface-type))

(def surface-types
  {::cherry-spec/RefractingCircularConic
   {:display-name "Conic"
    :default #::cherry-spec{:n 1 :thickness 1 :diam 1 :roc 1 :k 1}}

   ::cherry-spec/RefractingCircularFlat
   {:display-name "Flat"
    :default #::cherry-spec{:n 1 :thickness 1 :diam 1}}

   ::cherry-spec/Stop
   {:display-name "Stop"
    :default #::cherry-spec{:n 1 :thickness 1 :diam 1}}})

(def parameters [::cherry-spec/n ::cherry-spec/thickness ::cherry-spec/diam ::cherry-spec/roc ::cherry-spec/k])

(defn vec-remove
  "remove elem in coll"
  [coll pos]
  (into (subvec coll 0 pos) (subvec coll (inc pos))))

(defn parameters-as-numbers [row]
  (update-vals row (fn [v] (if (string? v) (parse-double v) v))))

(defn rows->surfaces-and-gaps [rows]
 (for [row rows]
   (vector
    {(:surface-type row) (dissoc row :surface-type ::cherry-spec/thickness ::cherry-spec/n)}
    (select-keys row [::cherry-spec/thickness ::cherry-spec/n]))))

(defn entrance-pupil-diameter->aperture [d]
  {::cherry-spec/EntrancePupilDiameter {::cherry-spec/diam d}})

(defn surfaces-and-gaps->rows [surfaces-and-gaps]
  (mapv
    (fn [[surface gap]]
      (let [surface-type (first (keys surface))]
        (merge
          {:surface-type surface-type}
          (get surface surface-type)
          gap)))
    (partition 2 2 surfaces-and-gaps)))

(defn wasm-system-model [constructor {:keys [aperture surfaces-and-gaps]}]
  (let [^js/WasmSystemModel m (new constructor)]
    (doseq [[i [s g]] (map vector (range) surfaces-and-gaps)]
       (.insertSurfaceAndGap m (inc i) (clj->js s) (clj->js g)))
    (.setAperture m (clj->js aperture))
    m))

(set! *warn-on-infer* true)
(defn compute-results [raytrace-input]
  (go
    (let [cherry (<p! cherry-async)
          model (wasm-system-model (.-WasmSystemModel cherry) raytrace-input)
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
          sf (rendering/scaleFactor (clj->js surface-samples) w h 0.8)
          comSamples (rendering/centerOfMass (clj->js surface-samples))
          canvasCenterCoords [(/ w 2.) (/ h 2.)]
          canvasSurfs (rendering/toCanvasCoordinates (clj->js surface-samples)
                                                     (clj->js comSamples)
                                                     (clj->js canvasCenterCoords)
                                                     sf)]
      (.clearRect ctx 0 0 w h)
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
  (let [default (get-in surface-types [selected :default])]
   (merge
     default
     (select-keys old (keys default))
     {:surface-type selected})))

(comment
  (prefill-row {::cherry-spec/n 2} ::cherry-spec/Stop))

(defn surface-dropdown [surface change-fn]
  [:div.select
    {:class (when-not surface :is-primary)}
    [:select
      {:value (str (or (:surface-type surface) ::default))
       :on-change #(change-fn (read-string (.. % -target -value)))}
      [:option {:disabled true :value (str ::default) :hidden true}
               "Select surface type"]
      (for [t (keys surface-types)]
        ^{:key (str t)}
        [:option {:value (str t)}
                 (get-in surface-types [t :display-name])])]])

(defn param-input [spec value change-fn]
  (let [valid? (r/atom (s/valid? spec (parse-double (str @value))))]
    [(if @valid? :input.input :input.input.is-warning)
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
          [:code (-> (rows->surfaces-and-gaps @surfaces)
                     (clj->js)
                     (js/JSON.stringify nil 2))]]]]])

(defn results-viewer-component [rows aperture results]
  (r/with-let [watch (r/track!
                       (fn [] (let [inputs {:aperture (entrance-pupil-diameter->aperture (parse-double (str @aperture)))
                                            :surfaces-and-gaps (rows->surfaces-and-gaps (map parameters-as-numbers @rows))}]
                                (when (s/valid? ::cherry-spec/raytrace-inputs inputs)
                                  (take!
                                    (compute-results inputs)
                                    #(reset! results %))))))]
    [canvas-component results]
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
          [:button.button {:on-click #(reset! surfaces (surfaces-and-gaps->rows cherry-spec/planoconvex))} "Planoconvex"]]
        [:p.level-item
          [:button.button {:on-click #(reset! surfaces (surfaces-and-gaps->rows cherry-spec/petzval))} "Petzval"]]
        [:p.level-item
          [:button.button {:on-click #(reset! surfaces (into [] (gen/sample (s/gen ::row) 5)))}
           "I'm Feeling Lucky"]]
        [:p.level-item
          [:button.button.is-success {:on-click #(swap! surfaces conj nil)} "New"]]]]
    [:div.table-container
      [:table.table
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
                 [:button.button {:on-click #(swap! surfaces vec-remove i)} "Delete"]]])]]]])

(defn entrance-pupil [value]
  [:<>
    [:nav.level
      [:div.level-left
         [:div.level-item
           [:p.subtitle "Aperture"]]]
      [:div.level-right]]
    [:div.field
      [:label.label "Entrance pupil diameter"]
      [param-input ::cherry-spec/diam value #(reset! value %)]]])

(defonce surfaces (r/atom (vec (gen/sample (s/gen ::row) 3))))

(defn main []
  (r/with-let [results (r/atom nil)
               aperture (r/atom 25)]
    [:<>
       [:section.section
         [:h1.title "Cherry Raytracer"]
         [results-viewer-component surfaces aperture results]
         [surfaces-table surfaces]
         [entrance-pupil aperture]]
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
