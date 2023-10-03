(ns net.thewagner.cherry
  (:require [goog.dom :as dom]
            [goog.events :as events]
            [goog.functions :refer [throttle]]
            [goog.dom.classlist :as classlist]
            [cljs.spec.alpha :as s]
            [cljs.spec.gen.alpha :as gen]
            [cljs.core.async :as async :refer [<! >!]]
            [cljs.core.async.interop :refer-macros [<p!]]
            [cljs.tools.reader.edn :as edn]
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

(defn random-surfaces-and-gaps []
  (-> (s/tuple ::cherry-spec/surface ::cherry-spec/gap)
      (s/gen)
      (gen/sample 5)
      flatten
      vec))

(defn wasm-system-model [constructor {:keys [aperture surfaces-and-gaps]}]
  (let [^js/WasmSystemModel m (new constructor)]
    (doseq [[i [s g]] (map vector (range) surfaces-and-gaps)]
       (.insertSurfaceAndGap m (inc i) (clj->js s) (clj->js g)))
    (.setAperture m (clj->js aperture))
    m))

(set! *warn-on-infer* true)
(defn compute-results [raytrace-input]
  (async/go
    (let [cherry (<p! cherry-async)
          model (wasm-system-model (.-WasmSystemModel cherry) raytrace-input)
          surfaces (.surfaces model)
          gaps (.gaps model)
          results (try
                    (.rayTrace model)
                    (catch js/Error e
                      (js/console.error "Unexpected error in rayTrace." e)
                      ::runtime-error))
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

(defn prefill-row [old selected]
  (let [default (get-in surface-types [selected :default])]
   (merge
     default
     (select-keys old (keys default))
     {:surface-type selected})))

(comment
  (prefill-row {::cherry-spec/n 2} ::cherry-spec/Stop))

(defprotocol IValidated
  (-set-valid! [input])
  (-set-invalid! [input]))

(extend-type js/HTMLInputElement
  IValidated
  (-set-valid! [input]
    (classlist/remove input "is-warning"))
  (-set-invalid! [input]
    (classlist/add input "is-warning")))

(defn valid-number [input-str spec]
  (let [number (parse-double input-str)]
    (and (s/valid? spec number) number)))

(defprotocol IDataGrid
  (-insert-row! [table row])
  (-delete-row! [table n]))

(defprotocol IPrefill
  (-prefill! [ui data]))

(defn- tbody [table]
  (first (dom/getElementsByTagName "tbody" table)))

(extend-type js/HTMLTableElement
  IDataGrid
  (-insert-row! [table data]
    (let [row (dom/createDom "tr")]
      (-prefill! row data)
      (dom/appendChild (tbody table) row)))

  (-delete-row! [table n]
    (.deleteRow table (inc n)))

  IPrefill
  (-prefill! [table rows]
    (dom/removeChildren (tbody table))
    (doseq [r rows]
      (-insert-row! table r))))

(extend-type js/HTMLTableRowElement
  IPrefill
  (-prefill! [row data]
    (dom/removeChildren row)
    ; Surface select combo-box
    (dom/appendChild row
      (dom/createDom "td" {}
        (dom/createDom "div" #js {:class "select"}
          (dom/createDom "select" {}
            ; Default option to promt the user to select
            (dom/createDom "option" #js {:selected (nil? (:surface-type data))
                                         :disabled true
                                         :hidden true}
                           "Select")
            ; Surface types
            (clj->js
              (for [[k v] surface-types]
                (dom/createDom "option" #js {:value (str k)
                                             :selected (= k (:surface-type data))}
                               (:display-name v))))))))
    ; Parameter columns
    (doseq [k parameters]
      (dom/appendChild row
        (dom/createDom "td" {}
          (when-let [value (get data k)]
            (dom/createDom "input" #js {:class "input" :value value})))))
    ; Actions
    (dom/appendChild row
      (dom/createDom "td" {}
        (dom/createDom "button" #js {:class "button"} "Delete")))))

(defonce state (atom {:event-handlers #{}}))

(defn tag-match [tag]
  (fn [el]
    (when-let [tag-name (.-tagName el)]
      (= tag (.toLowerCase tag-name)))))

(defn table-coords [el]
  (let [col (.closest el "td")
        row (.closest col "tr")]
    [(.-rowIndex row) (.-cellIndex col)]))

(defn listen-events! [{:keys [preset param-input row-edit select resize]}]
  (let [table (dom/getElement "surfaces-table")
        event-keys [(goog.events.listen js/window events/EventType.RESIZE
                       #(async/put! resize %))

                    (goog.events.listen (dom/getElement "new-row-button") events/EventType.CLICK
                       #(async/put! row-edit [:insert-row -1]))

                    (goog.events.listen (dom/getElement "preset-planoconvex-button") events/EventType.CLICK
                       #(async/put! preset cherry-spec/planoconvex))

                    (goog.events.listen (dom/getElement "preset-petzval-button") events/EventType.CLICK
                       #(async/put! preset cherry-spec/petzval))

                    (goog.events.listen (dom/getElement "preset-random-button") events/EventType.CLICK
                       #(async/put! preset (random-surfaces-and-gaps)))

                    (goog.events.listen table events/EventType.CLICK
                      (fn [e]
                        (let [el (.. e -target)]
                          (when ((tag-match "button") el)
                            (let [[row _col] (table-coords el)]
                              (async/put! row-edit [:delete-row (dec row)]))))))

                    (goog.events.listen table events/EventType.INPUT
                      (throttle
                        (fn [e]
                          (let [el (.. e -target)
                                [row col] (table-coords el)]
                            (cond
                              ((tag-match "input") el)
                              (let [spec (get parameters (dec col))]
                                (async/put! param-input [el row spec (.-value el)]))

                              ((tag-match "select") el)
                              (let [tr (.closest el "tr")
                                    value (edn/read-string (.-value el))]
                                (async/put! select [tr row value])))))
                        250))]]
    (swap! state update :event-handlers #(into % event-keys))))

(defn unlisten-events! []
  (doseq [k (@state :event-handlers)]
    (events/unlistenByKey k))
  (swap! state assoc :event-handlers #{}))

(def done (async/chan))

; https://code.thheller.com/blog/shadow-cljs/2019/08/25/hot-reload-in-clojurescript.html
(defn ^:dev/after-load start []
  (let [table (dom/getElement "surfaces-table")
        canvas (dom/getElement "systemModel")
        preset (async/chan)
        param-input (async/chan)
        row-edit (async/chan)
        select (async/chan)
        result (async/chan)
        resize (async/chan)]
    (listen-events! {:preset preset :param-input param-input :row-edit row-edit :select select :resize resize})
    ; Input process
    (async/go-loop [rows []]
      ; If we have valid inputs, send it to the raytracer
      (let [inputs {:aperture (entrance-pupil-diameter->aperture 20)
                    :surfaces-and-gaps (rows->surfaces-and-gaps rows)}]
        (if (s/valid? ::cherry-spec/raytrace-inputs inputs)
          (>! result (<! (compute-results inputs)))))

      ; Wait for events
      (async/alt!
        done ([_] ::done)
        preset ([d] (let [data (surfaces-and-gaps->rows d)]
                      (-prefill! table data)
                      (recur data)))
        row-edit ([[op row]] (case op
                               :insert-row (do (-insert-row! table {})
                                               (recur (conj rows {})))
                               :delete-row (do (-delete-row! table row)
                                               (recur (vec-remove rows row)))))
        select ([[tr row value]] (let [new-row (prefill-row (get rows row) value)]
                                   (-prefill! tr new-row)
                                   (recur (assoc rows row new-row))))
        param-input ([[el row spec value]] (if-let [n (valid-number value spec)]
                                             (do (-set-valid! el)
                                                 (recur (assoc-in rows [row spec] n)))
                                             (do (-set-invalid! el)
                                                 (recur rows))))))
    ; Render process
    (async/go-loop [r {}]
      (let [{:keys [surface-samples ray-samples]} r
            width (.-clientWidth (.closest canvas "div"))
            aspect-ratio 3]
         (set! (.-width canvas) width)
         (set! (.-height canvas) 150)
         (render canvas surface-samples ray-samples))
      (async/alt!
        done ([_] ::done)
        result ([d] (recur d))
        resize ([_] (recur r))))))

(defn ^:dev/before-load stop []
  (unlisten-events!)
  (async/close! done))

(defn init []
  (start))

(comment
  ; Evaluate these lines to enter into a ClojureScript REPL
  (require '[shadow.cljs.devtools.api :as shadow])
  (shadow/repl :app)
  ; Exit the CLJS session
  :cljs/quit)
