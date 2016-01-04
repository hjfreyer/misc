module Analysis where

Rel : Set → Set₁
Rel A = (A → A → Set)

data ⊥ : Set where

¬ : Set → Set
¬ A = A → ⊥

data Decision {A : Set} (_~_ : Rel A) : A → A → Set where
  yes : {a b : A} →  (a ~ b) → Decision _~_ a b
  no  : {a b : A} → ¬(a ~ b) → Decision _~_ a b

data Bool : Set where
  true : Bool
  false : Bool

data IsTrue : Bool → Set where
  indeed : IsTrue true

data ℕ : Set where
  zero : ℕ
  succ : ℕ → ℕ

_ℕ≡_ : ℕ → ℕ → Bool
zero ℕ≡ zero         = true
zero ℕ≡ (succ _)     = false
(succ _) ℕ≡ zero     = false
(succ n) ℕ≡ (succ m) = n ℕ≡ m

_ℕ+_ : ℕ → ℕ → ℕ
zero ℕ+ m = m
(succ n) ℕ+ m = succ (n ℕ+ m)

_ℕ*_ : ℕ → ℕ → ℕ
zero ℕ* _ = zero
(succ n) ℕ* m = m ℕ+ (n ℕ* m)

data ℤ : Set where
  _-_ : ℕ → ℕ → ℤ

_ℤ≡_ : ℤ → ℤ → Bool
(a - b) ℤ≡ (c - d) = (a ℕ+ d) ℕ≡ (b ℕ+ c)

_ℤ+_ : ℤ → ℤ → ℤ
(a - b) ℤ+ (c - d) = (a ℕ+ c) - (b ℕ+ d)

_ℤ*_ : ℤ → ℤ → ℤ
(a - b) ℤ* (c - d) = ((a ℕ* c) ℕ+ (b ℕ+ d)) - ((a ℕ* d) ℕ+ (b ℕ+ c))

data ℚ : Set where
  _/_ : ℕ → ℕ → ℚ
