open import Data.String

module Symbolic (R : Set) where


data Expr : Set where
  var : String → Expr
