(ns net.thewagner.cherry
  (:require [goog.dom :as gdom]
            [goog.dom.element :refer [isHtmlAnchorElement isHtmlButtonElement isHtmlElementOfType]]
            [goog.events :as events]
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
            [clojure.test.check.generators]
            [science.browser.cherry.dom :as dom]
            [net.thewagner.html :as html])

  (:import [goog.events EventType KeyCodes KeyHandler]
           [goog.dom TagName]))

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
          sf (rendering/scaleFactor (clj->js surface-samples) w h 0.95)
          centerSamples (rendering/center (clj->js surface-samples))
          canvasCenterCoords [(/ w 2.) (/ h 2.)]
          canvasSurfs (rendering/toCanvasCoordinates (clj->js surface-samples)
                                                     (clj->js centerSamples)
                                                     (clj->js canvasCenterCoords)
                                                     sf)]
      (.clearRect ctx 0 0 w h)
      (rendering/draw canvasSurfs ctx "black" 1.0)
      (when ray-samples
        (let [canvasRays (rendering/toCanvasCoordinates (clj->js ray-samples)
                                                        (clj->js centerSamples)
                                                        (clj->js canvasCenterCoords)
                                                        sf)]
          (rendering/draw canvasRays ctx "#cc252c" 1.0))))))

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

(defn isHtmlTableCellElement [el] (isHtmlElementOfType el TagName/TD))
(defn isHtmlSelectElement [el] (isHtmlElementOfType el TagName/SELECT))

(defn hidden-padding [pad]
  (dom/build [:span {:style "visibility:hidden"} pad]))

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
  (-append-row! [table row])
  (-insert-row-at! [table row index])
  (-delete-row! [table n]))

(defprotocol IPrefill
  (-prefill! [ui data]))

(defprotocol IEditable
  (-start-edit! [ui])
  (-stop-edit! [ui]))

(extend-type js/HTMLTableCellElement
  IValidated
  (-set-valid! [ui]
    (-set-valid! (first (gdom/getElementsByTagName "input" ui))))
  (-set-invalid! [ui]
    (-set-invalid! (first (gdom/getElementsByTagName "input" ui))))

  IEditable
  (-start-edit! [ui]
    (let [input (dom/build [:input.input {:value (.-innerText ui)}])]
      (dom/remove-children ui)
      (dom/append ui input)
      (.focus input)))
  (-stop-edit! [ui]
    (let [input (first (gdom/getElementsByTagName "input" ui))
          value (.-value input)
          pad (decimal-padding value 5)]
      (-> (dom/remove-children ui)
          (dom/append (dom/text value))
          (dom/append (hidden-padding pad))))))

(defn- tbody [table]
  (first (gdom/getElementsByTagName "tbody" table)))

(extend-type js/HTMLTableElement
  IDataGrid
  (-append-row! [table data]
    (let [row (dom/build [:tr])]
      (-prefill! row data)
      (dom/append (tbody table) row)))

  (-insert-row-at! [table data index]
    (let [row (dom/build [:tr])]
      (-prefill! row data)
      (gdom/insertChildAt (tbody table) row index)))

  (-delete-row! [table n]
    (.deleteRow table (inc n)))

  IPrefill
  (-prefill! [table rows]
    (dom/remove-children (tbody table))
    (doseq [r rows]
      (-append-row! table r))))

(def object-or-image-plane? #{::cherry-spec/ObjectPlane ::cherry-spec/ImagePlane})

(extend-type js/HTMLTableRowElement
  IPrefill
  (-prefill! [row {:keys [surface-type] :as data}]
    (dom/remove-children row)
    ; Surface select combo-box
    (dom/append row
      (dom/build
        [:td
         (if (object-or-image-plane? surface-type)
           (dom/text (get-in surface-types [surface-type :display-name]))
           [:div.select
             (into
               [:select
                 [:option {:selected (nil? surface-type) :disabled true :hidden true} "Select"]]
               (for [[k v] (remove #(object-or-image-plane? (first %)) surface-types)]
                 [:option {:selected (= k surface-type) :value (str k)} (:display-name v)]))])]))

    ; Parameter columns
    (doseq [k parameters]
      (dom/append row
        (dom/build
          (if-let [value (get data k)]
            (let [pad (decimal-padding (str value) 5)]
              [:td (dom/text value) (hidden-padding pad)])
            [:td]))))
    ; Actions
    (dom/append row
      (dom/build
        [:td
          (cond-> [:div.field.is-grouped]
            (not= surface-type ::cherry-spec/ImagePlane)
            (conj [:p.control [:button.button.action-insert "Insert"]])

            (not (object-or-image-plane? surface-type))
            (conj [:p.control [:button.button.action-delete "Delete"]]))]))))

(defprotocol ITabs
  (-set-active! [ui tab-name])
  (-render-tab! [ui tab-name opts]))

(extend-type js/HTMLDivElement
  ITabs
  (-set-active! [ui tab-name]
    (-> ui
        (dom/remove-children)
        (dom/append (dom/build (html/tabs-nav tab-name)))))
  (-render-tab! [ui tab-name opts]
    (dom/remove-children ui)
    (dom/append ui (dom/build (html/tabs-body tab-name opts)))))

(defn listen! [src event-type c]
  (let [node (if (keyword? src) (dom/get-element src) src)]
    (events/listen node event-type (fn [e] (async/put! c e)))))

(defn unlisten! [& ks]
  (doseq [k ks]
    (events/unlistenByKey k)))

(def done (chan))

(defn start-render!
  "Listen on the result channel and display the surface and ray samples on the canvas.
   Stop when the done channel closes."
  [done & {:keys [canvas result]}]
  (async/go
    (let [resize (chan)
          handler (listen! js/window EventType/RESIZE resize)]
      (try
        (loop [r {}]
          (let [{:keys [surface-samples ray-samples]} r
                width (.-clientWidth (.closest canvas "div"))]
             (set! (.-width canvas) width)
             (set! (.-height canvas) 150)
             (render canvas surface-samples ray-samples))
          (async/alt!
            done ([_] ::done)
            result ([d] (recur d))
            resize ([_] (recur r))))

        (finally
          (unlisten! handler))))))

(defn start-validator!
  "Validate values from the UI element element and put the valid values to out-ch"
  [done & {:keys [ui default-value spec out-ch]}]
  (async/go
    (let [keypress-ch (chan 1 (filter #(= (.-keyCode %) KeyCodes/ENTER)))
          input-ch (chan 1 (map #(.. % -target -value)))
          focusout-ch (chan)
          keypress-handler (listen! (KeyHandler. ui) KeyHandler/EventType.KEY keypress-ch)
          input-handler (listen! ui EventType/INPUT input-ch)
          focusout-handler (listen! ui EventType/FOCUSOUT focusout-ch)]
      (try
        (loop [last-valid default-value]
          (async/alt!
            [done focusout-ch keypress-ch]
            ([_] {:default-value last-valid})

            input-ch
            ([value] (if-let [n (valid-number value spec)]
                       (do (-set-valid! ui)
                         (async/offer! out-ch n)
                         (recur value))
                       (do (-set-invalid! ui)
                         (recur last-valid))))))
        (finally
          (unlisten! keypress-handler input-handler focusout-handler))))))

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

(defn table-events [table]
  (let [cell-click (chan 1
                     (comp
                       (map #(.-target %))
                       (filter (some-fn isHtmlButtonElement isHtmlTableCellElement))
                       (map locate-in-table)
                       (keep (fn [{:keys [node column-index] :as loc}]
                               (cond
                                 (and (isHtmlButtonElement node) (classlist/contains node "action-insert"))
                                 (assoc loc :op :insert-row)

                                 (and (isHtmlButtonElement node) (classlist/contains node "action-delete"))
                                 (assoc loc :op :delete-row)

                                 (and (isHtmlTableCellElement node)
                                      (seq (.-innerText node))
                                      (<= 0 column-index 4))
                                 (assoc loc :op :start-edit))))))
         select (chan 1 (comp
                          (map #(.-target %))
                          (filter isHtmlSelectElement)
                          (map locate-in-table)
                          (map #(update % :value edn/read-string))
                          (map #(assoc % :op :change-row-type))))]
    (listen! table EventType.CLICK cell-click)
    (listen! table EventType.INPUT select)
    {:row-edit (async/merge [cell-click select])}))

(defn start-table!
  "Data grid process"
  [done & {:keys [table rows preset row-edit out-ch]}]
  (-prefill! table rows)
  (async/go-loop [rows rows]
    (async/alt!
      done ([_] {:rows rows})
      preset ([data] (do (-prefill! table data)
                         (recur data)))
      row-edit
      ([{:keys [op tr td column row-index value] :as cmd}]
       (let [new-rows (case op
                        :edit-cell       (assoc-in rows [row-index column] value)
                        :insert-row      (let [[before after] (split-at (inc row-index) rows)]
                                           (vec (concat before [{}] after)))
                        :delete-row      (vec-remove rows row-index)
                        :change-row-type (assoc rows row-index (prefill-row (get rows row-index) value))
                        rows)]
         (case op
           :start-edit
           (let [valid-values (chan 1 (map (fn [v] (assoc cmd :op :edit-cell :value v))))]
             (async/pipe valid-values row-edit false)
             (-start-edit! td)
             (async/go
               (<! (start-validator! done :ui td :spec column :out-ch valid-values))
               (-stop-edit! td)
               (async/close! valid-values)))
           :insert-row
           (-insert-row-at! table {} (inc row-index))
           :delete-row
           (-delete-row! table row-index)
           :change-row-type
           (-prefill! tr (get new-rows row-index))
           ::noop)
        (async/offer! out-ch new-rows)
        (recur new-rows))))))

(defn start-input!
  "Send valid values from the input channel to the raytracer. The output of the
  ray tracer is sent to result.  Returns when done closes."
  [done & {:keys [input preset result]}]
  (async/go-loop [raytrace-inputs {}]
   (when (s/valid? ::cherry-spec/raytrace-inputs raytrace-inputs)
     (>! result (<! (compute-results raytrace-inputs))))
   (async/alt!
      done ([_] ::done)
      [preset input]
      ([new-rows] (if (number? new-rows)
                    (recur (merge raytrace-inputs
                                  {:aperture (entrance-pupil-diameter->aperture new-rows)}))
                    (recur (merge {:aperture (entrance-pupil-diameter->aperture 10)}
                                  raytrace-inputs
                                  (rows->surfaces-and-gaps new-rows))))))))

(defn start-tabs!
  "Start the Tabs process and listen on the tabs channel. Returns when done
  closes."
  [done & {:keys [tabs input preset]}]
  (async/go-loop [active-tab :surfaces
                  tab-done (chan)
                  tab-proc (start-table! tab-done
                                         (merge (table-events (dom/get-element :surfaces-table))
                                                {:table (dom/get-element :surfaces-table)
                                                 :rows (surfaces-and-gaps->rows cherry-spec/object-and-image-plane)
                                                 :preset preset
                                                 :out-ch input}))
                  tabs-state {}]
    (async/alt!
      done ([_] ::done)
      tabs
      ([[el tab]]
       (if (= active-tab tab)
         (recur active-tab tab-done tab-proc tabs-state)
         (let [state (when tab-proc
                       (async/close! tab-done) ; ask the tab proces to close
                       (<! tab-proc))]         ; wait for the process to exit
           (-set-active! el tab)
           (-render-tab! (dom/get-element :tab-body) tab (tabs-state tab))
           (let [tab-done (chan)]
             (recur
               tab
               tab-done
               (case tab
                 :surfaces
                 (let [table (dom/get-element :surfaces-table)]
                   (start-table! tab-done (merge {:table table :preset preset :out-ch input}
                                             (table-events table)
                                             (tabs-state tab))))
                 :aperture
                 (let [ui (dom/get-element :aperture-input)]
                   (start-validator! tab-done (merge {:ui ui :spec ::cherry-spec/diam :out-ch input}
                                                     (tabs-state tab))))
                 nil)
               (assoc tabs-state active-tab state)))))))))

; https://code.thheller.com/blog/shadow-cljs/2019/08/25/hot-reload-in-clojurescript.html
(defn ^:dev/after-load start []
  (let [preset (chan 1 (comp (map #(.. % -target -id))
                             (keep {"preset-planoconvex" cherry-spec/planoconvex
                                    "preset-petzval"     cherry-spec/petzval})
                             (map surfaces-and-gaps->rows)))
        preset-mult (async/mult preset)
        preset->input (chan)
        preset->tabs (chan (async/sliding-buffer 1))
        tabs (chan 1 (comp
                       (map #(.-target %))
                       (filter isHtmlAnchorElement)
                       (map (fn [el] [(.closest el "div") (keyword (.-id el))]))))
        input (chan)
        result (chan)]

    (listen! (.querySelector js/document "nav.navbar") EventType/CLICK preset)
    (listen! :system-parameters EventType.CLICK tabs)

    (async/tap preset-mult preset->input)
    (async/tap preset-mult preset->tabs)

    (start-input! done :input input :result result :preset preset->input)
    (start-tabs! done :tabs tabs :input input :preset preset->tabs)
    (start-render! done :canvas (dom/get-element :systemModel) :result result)))

(def burger-toggle-handler
  (let [burger (.querySelector js/document ".navbar-burger")]
    (events/listen burger EventType/CLICK
      (fn [_]
        (classlist/toggle burger "is-active")
        (classlist/toggle (gdom/getElement (.. burger -dataset -target)) "is-active")))))

(defn ^:dev/before-load stop []
  (async/close! done)
  (unlisten! burger-toggle-handler))

(defn init []
  (start))

(comment
  ; Evaluate these lines to enter into a ClojureScript REPL
  (require '[shadow.cljs.devtools.api :as shadow])
  (shadow/repl :app)
  ; Exit the CLJS session
  :cljs/quit)
