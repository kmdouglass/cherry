(ns kmdouglass.cherry
  (:require [clojure.spec.alpha :as s]
            [clojure.test.check.generators :as gen]))

(s/def ::surface (s/or
                   :ObjectOrImagePlane (s/keys :req-un [::ObjectOrImagePlane])
                   :RefractingCircularFlat (s/keys :req-un [::RefractingCircularFlat])
                   :RefractingCircularConic (s/keys :req-un [::RefractingCircularConic])))

(s/def ::ObjectOrImagePlane (s/keys :req [::diam]))
(s/def ::RefractingCircularFlat (s/keys :req [::diam]))
(s/def ::RefractingCircularConic (s/keys :req [::diam ::roc ::k]))

(s/def ::gap (s/keys :req [::n ::thickness]))
(s/def ::surface-and-gap (s/cat :surface ::surface :gap ::gap))

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

; Ray-race results
(s/def ::raytrace-results (s/and (s/coll-of ::ray)
                                 (fn all-same-length [rays] (= 1 (count (into #{} (map count rays)))))))
(s/def ::ray (s/coll-of (s/keys :req-un [::pos ::dir])))
(s/def ::pos (s/tuple number? number? number?))
(s/def ::dir (s/tuple number? number? number?))

(s/def ::surface-samples (s/coll-of (s/keys :req-un [::samples])))
(s/def ::samples (s/coll-of (s/tuple number? number? number?) :min-count 1))

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
  (fn [rays] (= 1 (count (into #{} (map count [[1] [1]])))))
  (gen/sample (s/gen ::raytrace-results) 5)
  (gen/sample (s/gen ::ray) 5)
  (gen/sample (s/gen ::surface-samples) 5)
  (gen/sample (s/gen ::gap) 5))
