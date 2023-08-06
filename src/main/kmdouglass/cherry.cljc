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

(comment
  (gen/sample (s/gen ::surface) 5)
  (gen/sample (s/gen ::gap) 5))
