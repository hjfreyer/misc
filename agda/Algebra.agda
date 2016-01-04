open import agda.Relation

module agda.Algebra where

record IsGroup {A : Set}
  (_≡_ : Rel A)
  (e : A)
  (_+_ : Op₂ A)
  (- : Op₁ A) where
  assoc :
