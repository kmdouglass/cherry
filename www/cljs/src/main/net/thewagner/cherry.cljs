(ns net.thewagner.cherry
  (:require [goog.dom :as dom]
            [goog.events :as events]
            [goog.functions :refer [throttle]]
            [goog.dom.classlist :as classlist]
            [clojure.string :as string]
            [cljs.spec.alpha :as s]
            [cljs.spec.gen.alpha :as gen]
            [cljs.core.async :as async :refer [chan <! >!]]
            [cljs.core.async.interop :refer-macros [<p!]]
            [cljs.tools.reader.edn :as edn]
            [rendering]
            [cherry :as cherry-async]
            [kmdouglass.cherry :as cherry-spec]
            [clojure.test.check.generators])
  (:import [goog.events EventType KeyCodes KeyHandler]))

(defmulti row-type :surface-type)
(defmethod row-type ::cherry-spec/ObjectPlane [_]
  (s/merge ::cherry-spec/ObjectPlane ::cherry-spec/gap))

(defmethod row-type ::cherry-spec/ImagePlane [_]
  ::cherry-spec/ImagePlane)

(defmethod row-type ::cherry-spec/RefractingCircularFlat [_]
  (s/merge ::cherry-spec/RefractingCircularFlat ::cherry-spec/gap))

(defmethod row-type ::cherry-spec/RefractingCircularConic [_]
  (s/merge ::cherry-spec/RefractingCircularConic ::cherry-spec/gap))

(defmethod row-type ::cherry-spec/Stop [_]
  (s/merge ::cherry-spec/Stop ::cherry-spec/gap))

(s/def ::row (s/multi-spec row-type :surface-type))

(def surface-types
  {::cherry-spec/ObjectPlane
   {:display-name "Object Plane"
    :default #::cherry-spec{:diam 1 :n 1 :thickness 1}}

   ::cherry-spec/ImagePlane
   {:display-name "Image Plane"
    :default #::cherry-spec{:diam 1}}

   ::cherry-spec/RefractingCircularConic
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
  (reduce
    (fn [result row]
      (let [gap     (select-keys row [::cherry-spec/thickness ::cherry-spec/n])
            surface (dissoc row :surface-type ::cherry-spec/thickness ::cherry-spec/n)]
        (cond-> result
          true          (update :surfaces conj {(row :surface-type) surface})
          (not= gap {}) (update :gaps conj gap))))
    {:surfaces [] :gaps []}
    rows))

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
    (partition 2 2 nil surfaces-and-gaps)))

(defn random-surfaces-and-gaps []
  (-> (s/tuple ::cherry-spec/surface ::cherry-spec/gap)
      (s/gen)
      (gen/sample 5)
      flatten
      vec))

(defn wasm-system-model [constructor {:keys [surfaces gaps aperture]}]
  (doto
    (new constructor)
    (.setSurfaces (clj->js surfaces))
    (.setGaps (clj->js gaps))
    (.setApertureV2 (clj->js aperture))
    (.setFields (clj->js [{:angle 0} {:angle 5}]))
    (.build)))

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

(defn decimal-padding [s width]
  (let [[l r] (string/split s ".")
        diff (max 0 (- width (count r)))
        pad (apply str (repeat diff "0"))]
    (if (= s l)
      (str "." pad)
      pad)))

(comment
  (decimal-padding "1.234" 5)
  (decimal-padding "1" 5))

(defn hidden-padding [pad]
  (dom/createDom "span" #js {:style "visibility:hidden"} pad))

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

(defprotocol IEditable
  (-start-edit! [ui])
  (-stop-edit! [ui]))

(extend-type js/HTMLTableCellElement
  IEditable
  (-start-edit! [ui]
    (let [value (.-innerText ui)
          input (dom/createDom "input" #js {:class "input" :value value})]
      (dom/removeChildren ui)
      (dom/appendChild ui input)
      (.focus input)))
  (-stop-edit! [ui]
    (let [input (first (dom/getElementsByTagName "input" ui))
          value (.-value input)
          pad (decimal-padding value 5)]
      (dom/removeChildren ui)
      (dom/appendChild ui (dom/createTextNode value))
      (dom/appendChild ui (hidden-padding pad)))))

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
        (if-let [value (get data k)]
          (let [pad (decimal-padding (str value) 5)]
             (dom/createDom "td" {}
               (dom/createTextNode value)
               (hidden-padding pad)))
          (dom/createDom "td" {}))))
    ; Actions
    (dom/appendChild row
      (dom/createDom "td" {}
        (dom/createDom "button" #js {:class "button"} "Delete")))))

(defn tag-match [tag]
  (fn [el]
    (when-let [tag-name (.-tagName el)]
      (= tag (.toLowerCase tag-name)))))

(defonce event-handlers (atom #{}))

(defn listen
  ([src event-type] (listen src event-type (chan)))
  ([src event-type c]
   (let [handler (events/listen src event-type (fn [e] (async/offer! c e)))]
     (swap! event-handlers conj handler)
     c)))

(defn unlisten-all! []
  (doseq [k @event-handlers] (events/unlistenByKey k))
  (reset! event-handlers #{}))

(comment
  (unlisten-all!)
  (listen js/window EventType.MOUSEMOVE (chan 1 (map (fn [e] (do (js/console.log e) e))))))

(def done (chan))

(defn locate-in-table
  "Locate the provided node in the nearest <table>"
  [node]
  (let [td (.closest node "td")
        tr (.closest td "tr")
        column-index (dec (.-cellIndex td))
        row-index (dec (.-rowIndex tr))]
    {:node node
     :value (.-value node)
     :column (get parameters column-index)
     :tr tr
     :td td
     :row-index row-index
     :column-index column-index}))

; https://code.thheller.com/blog/shadow-cljs/2019/08/25/hot-reload-in-clojurescript.html
(defn ^:dev/after-load start []
  (let [table (dom/getElement "surfaces-table")
        resize (listen js/window EventType/RESIZE)
        preset (listen (dom/getElement "surfaces-table-nav") EventType/CLICK
                 (chan 1 (comp (map #(case (.. % -target -id)
                                           "preset-planoconvex-button" cherry-spec/planoconvex
                                           "preset-petzval-button"     cherry-spec/petzval
                                           "preset-random-button"      (random-surfaces-and-gaps)
                                            nil))
                               (filter some?))))
        param-input (listen table EventType/INPUT
                      (chan 1 (comp
                                (map #(.-target %))
                                (filter (tag-match "input"))
                                (map locate-in-table))))
        select (listen table EventType/INPUT
                 (chan 1 (comp
                           (map #(.-target %))
                           (filter (tag-match "select"))
                           (map locate-in-table)
                           (map #(update % :value edn/read-string)))))
        row-edit (async/merge
                   [(listen (dom/getElement "new-row-button") EventType.CLICK
                      (chan 1 (map (constantly {:op :insert-row}))))
                    (listen (KeyHandler. table) KeyHandler/EventType.KEY
                      (chan 1 (comp
                                (filter #(= (.-keyCode %) KeyCodes/ENTER))
                                (map #(.-target %))
                                (filter (tag-match "input"))
                                (map locate-in-table)
                                (map #(assoc % :op :stop-edit)))))
                    (listen table EventType.CLICK
                      (chan 1 (comp
                                (map #(.-target %))
                                (filter #(contains? #{"button" "td"} (.. % -tagName toLowerCase)))
                                (map locate-in-table)
                                (map (fn [{:keys [node column-index] :as loc}]
                                       (cond
                                         ((tag-match "button") node)
                                         (assoc loc :op :delete-row)

                                         (and ((tag-match "td") node)
                                              (not (empty? (.-innerText node)))
                                              (<= 0 column-index 5))
                                         (assoc loc :op :start-edit))))
                                (filter some?))))
                    (listen table EventType.FOCUSOUT
                      (chan 1 (comp
                                (map #(.-target %))
                                (filter (tag-match "input"))
                                (map locate-in-table)
                                (map #(assoc % :op :stop-edit)))))])
        result (chan)]
    ; Input process
    (async/go-loop [rows []]
      ; If we have valid inputs, send it to the raytracer
      (let [inputs (merge (rows->surfaces-and-gaps rows)
                          {:aperture (entrance-pupil-diameter->aperture 10)})]
        (when (s/valid? ::cherry-spec/raytrace-inputs inputs)
          (>! result (<! (compute-results inputs)))))

      ; Wait for events
      (async/alt!
        done ([_] ::done)
        preset ([d] (let [data (surfaces-and-gaps->rows d)]
                      (-prefill! table data)
                      (recur data)))
        row-edit ([{:keys [op td row-index]}]
                  (case op
                   :start-edit (do (-start-edit! td)
                                   (recur rows))
                   :stop-edit (do (-stop-edit! td)
                                  (recur rows))
                   :insert-row (do (-insert-row! table {})
                                   (recur (conj rows {})))
                   :delete-row (do (-delete-row! table row-index)
                                   (recur (vec-remove rows row-index)))))
        select ([{:keys [tr row-index value]}]
                (let [new-row (prefill-row (get rows row-index) value)]
                 (-prefill! tr new-row)
                 (recur (assoc rows row-index new-row))))
        param-input ([{:keys [node row-index column value]}]
                     (if-let [n (valid-number value column)]
                       (do (-set-valid! node)
                           (recur (assoc-in rows [row-index column] n)))
                       (do (-set-invalid! node)
                           (recur rows))))))
    ; Render process
    (async/go-loop [r {}]
      (let [canvas (dom/getElement "systemModel")
            {:keys [surface-samples ray-samples]} r
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
  (unlisten-all!)
  (async/close! done))

(defn init []
  (start))

(comment
  ; Evaluate these lines to enter into a ClojureScript REPL
  (require '[shadow.cljs.devtools.api :as shadow])
  (shadow/repl :app)
  ; Exit the CLJS session
  :cljs/quit)
