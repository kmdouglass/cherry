(ns kmdouglass.cherry
  (:require [clojure.spec.alpha :as s]
            [clojure.spec.gen.alpha :as gen]))

(s/def ::surface (s/or
                   :ObjectPlane (s/keys :req [::ObjectPlane])
                   :ImagePlane (s/keys :req [::ImagePlane])
                   :RefractingCircularFlat (s/keys :req [::RefractingCircularFlat])
                   :RefractingCircularConic (s/keys :req [::RefractingCircularConic])
                   :Stop (s/keys :req [::Stop])))

(s/def ::ObjectPlane (s/keys :req [::diam]))
(s/def ::ImagePlane (s/keys :req [::diam]))
(s/def ::RefractingCircularFlat (s/keys :req [::diam]))
(s/def ::RefractingCircularConic (s/keys :req [::diam ::roc ::k]))
(s/def ::Stop (s/keys :req [::diam]))

(s/def ::gap (s/keys :req [::n ::thickness]))
(s/def ::surfaces (s/coll-of ::surface :min-count 2))
(s/def ::gaps (s/coll-of ::gap))

(def pos-number?
  (s/with-gen (s/and number? pos?)
              #(s/gen (s/and number? pos?))))

(s/def ::diam pos-number?)

; Radius of curvature. Convex is positive, concave is negative
(s/def ::roc number?)

; Conic constant
(s/def ::k number?)

(s/def ::n pos-number?)
(s/def ::thickness pos-number?)

; Aperture
(s/def ::aperture (s/keys :req [::EntrancePupilDiameter]))
(s/def ::EntrancePupilDiameter (s/keys :req [::diam]))

; Ray-trace inputs
(s/def ::raytrace-inputs (s/and (s/keys :req-un [::surfaces ::gaps ::aperture])
                                (fn [{:keys [surfaces gaps]}] (= (count gaps) (dec (count surfaces))))
                                (fn [{:keys [surfaces]}] (= :ObjectPlane (ffirst surfaces)))
                                (fn [{:keys [surfaces]}] (= :ImagePlane (first (last surfaces))))))

; Ray-race results
(s/def ::raytrace-results (s/and (s/coll-of ::ray)
                                 (fn all-same-length [rays] (= 1 (count (set (map count rays)))))))
(s/def ::ray (s/coll-of (s/keys :req-un [::pos ::dir])))
(s/def ::pos (s/tuple number? number? number?))
(s/def ::dir (s/tuple number? number? number?))

(s/def ::surface-samples (s/coll-of (s/keys :req-un [::samples])))
(s/def ::samples (s/coll-of (s/tuple number? number? number?) :min-count 1))

(def object-and-image-plane
  [{::ObjectPlane {::diam 50}}
   {::n 1 ::thickness 50}
   {::ImagePlane {::diam 50}}])

(def planoconvex
  [{::ObjectPlane {::diam 50}}
   {::n 1 ::thickness 50}
   {::RefractingCircularConic {::diam 25.0 ::roc 25.8 ::k 0.0}}
   {::n 1.515 ::thickness 5.3}
   {::RefractingCircularFlat {::diam 25.0}}
   {::n 1.0 ::thickness 46.6 ::diam 25.0}
   {::ImagePlane {::diam 50}}])

(def petzval
  [{::ObjectPlane {::diam 50}}
   {::n 1 ::thickness 200}
   {::RefractingCircularConic {::diam 56.956, ::roc 99.56266, ::k 0.0}}
   {::n 1.5168 ::thickness 13.0}
   {::RefractingCircularConic {::diam 52.552, ::roc -86.84002, ::k 0.0}}
   {::n 1.6645 ::thickness 4.0}
   {::RefractingCircularConic {::diam 42.04, ::roc -1187.63858, ::k 0.0}}
   {::n 1.0 ::thickness 40.0}
   {::Stop {::diam 33.262}}
   {::n 1.0 ::thickness 40.0,}
   {::RefractingCircularConic {::diam 41.086, ::roc 57.47491, ::k 0.0}}
   {::n 1.6074 ::thickness 12.0}
   {::RefractingCircularConic {::diam 40.148, ::roc -54.61685, ::k 0.0}}
   {::n 1.6727 ::thickness 3.0}
   {::RefractingCircularConic {::diam 32.984, ::roc -614.68633, ::k 0.0}}
   {::n 1.0 ::thickness 46.82210}
   {::RefractingCircularConic {::diam 34.594, ::roc -38.17110, ::k 0.0}}
   {::n 1.6727 ::thickness 2.0}
   {::RefractingCircularFlat {::diam 37.88}}
   {::n 1.0 ::thickness 1.87179}
   {::ImagePlane {::diam 50}}])

(comment
  (def ex [[{:pos [8.74227794156468e-7 -20 -1], :dir [0 0 1], :terminated false}
            {:pos [4.37113897078234e-7 -10 -1], :dir [0 0 1], :terminated false}
            {:pos [0 0 -1], :dir [0 0 1], :terminated false}
            {:pos [-4.37113897078234e-7 10 -1], :dir [0 0 1], :terminated false}
            {:pos [-8.74227794156468e-7 20 -1], :dir [0 0 1], :terminated false}]
           [{:pos [0 -19.999996185302734 0], :dir [0 0 1], :terminated false}
            {:pos [0 -9.999998092651367 0], :dir [0 0 1], :terminated false}
            {:pos [0 0 0], :dir [0 0 1], :terminated false}
            {:pos [0 9.999998092651367 0], :dir [0 0 1], :terminated false}
            {:pos [0 19.999996185302734 0], :dir [0 0 1], :terminated false}]
           [{:pos [0 -19.9999942779541 3], :dir [0 0 1], :terminated false}
            {:pos [0 -9.99999713897705 3], :dir [0 0 1], :terminated false}
            {:pos [0 0 3], :dir [0 0 1], :terminated false}
            {:pos [0 9.99999713897705 3], :dir [0 0 1], :terminated false}
            {:pos [0 19.9999942779541 3], :dir [0 0 1], :terminated false}]])

  (s/valid? ::raytrace-results ex)
  (s/explain ::raytrace-results ex)
  (gen/sample (s/gen ::raytrace-results) 5)
  (gen/sample (s/gen ::ray) 5)
  (gen/sample (s/gen ::surface-samples) 5)
  (gen/sample (s/gen ::gap) 5)
  (gen/sample (s/gen ::aperture) 5)
  (s/conform ::diam -10))
