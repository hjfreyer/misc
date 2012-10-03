
package main

import (
//	"bytes"
	"fmt"
	"utf8"
	"sort"
	"rand"
)

import (
	"os"
	"log"
	"runtime/pprof"
)

type Card string

func NewCard(rank, suit int) Card {
	return Card(string([]byte{byte(rank), byte(suit)}))
}

const (
	CardNames = "XX23456789TJQKA"
)

var Suits = utf8.NewString("♥♦♠♣")

func (c Card) String() string {

	return fmt.Sprintf("%c%c", CardNames[c.Rank()], Suits.At(c.Suit()))
}

type CardSort []Card

func (c CardSort) Len() int { return len(c) }
func (c CardSort) Less(i, j int) bool { return c[i] < c[j] }
func (c CardSort) Swap(i, j int) { c[i], c[j] = c[j], c[i] }

type CardPair string

func NewPair(a, b Card) CardPair {
	if a.Rank() < b.Rank() {
		a, b = b, a
	}

	var s byte
	if a.Suit() == b.Suit() {s = 'S'} else {s = 'U'}
	return CardPair([]byte{a[0], b[0], s})
}

func (c CardPair) String() string {
	return fmt.Sprintf("%c%c %c", CardNames[c[0]], CardNames[c[1]], c[2])
}

func (c Card) Rank() int {
	return int(c[0])
}

func (c Card) Suit() int {
	return int(c[1])
}

type Hand struct {
	Type int
	Kickers []int
}

const (
	HIGH_CARD = iota
	PAIR
	TWO_PAIR
	THREE_OF_A_KIND
	STRAIGHT
	FLUSH
	FULL_HOUSE
	FOUR_OF_A_KIND
	STRAIGHT_FLUSH
)

const (
	JACK = 11
	QUEEN = 12
	KING = 13
	ACE = 14
)

func Deck() (out []Card) {
	out = make([]Card, 52)

	for r := 2; r < 15; r++ {
		for s := 0; s < 4; s++ {
			out[s*13 + r - 2] = NewCard(r, s)
		}
	}
	return
}

var SEVEN_COMBINATIONS = [][]int{
	[]int{0, 1, 2, 3, 4},
	[]int{0, 1, 2, 3, 5},
	[]int{0, 1, 2, 3, 6},
	[]int{0, 1, 2, 4, 5},
	[]int{0, 1, 2, 4, 6},
	[]int{0, 1, 2, 5, 6},
	[]int{0, 1, 3, 4, 5},
	[]int{0, 1, 3, 4, 6},
	[]int{0, 1, 3, 5, 6},
	[]int{0, 1, 4, 5, 6},
	[]int{0, 2, 3, 4, 5},
	[]int{0, 2, 3, 4, 6},
	[]int{0, 2, 3, 5, 6},
	[]int{0, 2, 4, 5, 6},
	[]int{0, 3, 4, 5, 6},
	[]int{1, 2, 3, 4, 5},
	[]int{1, 2, 3, 4, 6},
	[]int{1, 2, 3, 5, 6},
	[]int{1, 2, 4, 5, 6},
	[]int{1, 3, 4, 5, 6},
	[]int{2, 3, 4, 5, 6},
}

func BestOfSeven(cards []Card) (best Hand) {
	if len(cards) != 7 {
		panic("ahhhhh")
	}

	best = Hand{-1, []int{}}
	for _, perm := range SEVEN_COMBINATIONS {
		hand := make([]Card, 5)
		for i, p := range perm {
			hand[i] = cards[p]
		}
		rating := RateHand(hand)
		if CmpHand(rating, best) > 0 {
			best = rating
		}
	}
	return
}

func CmpHand(a, b Hand) int {
	cmp := a.Type - b.Type
		if cmp != 0 { return cmp }

	for i := 0; i < len(a.Kickers); i++ {
		cmp = a.Kickers[i] - b.Kickers[i]
		if cmp != 0 { return cmp }
	}
	return 0
}

func RateHand(cards []Card) (hand Hand) {
	if len(cards) != 5 {
		panic("non-5 card hand")
	}
	ranks := make([]int, 5)
	rank_map := make([]int, 15)

	suit := cards[0].Suit()
	flush := true

	for i, c := range cards {
		ranks[i] = c.Rank()
		rank_map[c.Rank()]++
		flush = flush && c.Suit() == suit
	}

	sort.Sort(sort.IntSlice(ranks))

	straight_rank := 0

	if ranks[0] == 2 &&
		ranks[1] == 3 &&
		ranks[2] == 4 &&
		ranks[3] == 5 &&
		ranks[4] == ACE {
		straight_rank = 5
	} else if ranks[0] + 1 == ranks[1] &&
		ranks[1] + 1 == ranks[2] &&
		ranks[2] + 1 == ranks[3] &&
		ranks[3] + 1 == ranks[4] {
		straight_rank = ranks[4]
	}

	var pair1, pair2, triple, quadruple int

	for rank, count := range rank_map {
		switch count {
		case 2:
			if pair1 == 0 { pair1 = rank
			} else { pair2 = rank }
		case 3: triple = rank
		case 4: quadruple = rank
		}
	}

	if pair1 < pair2 {
		pair1, pair2 = pair2, pair1
	}

	kickers := func(kicker1, kicker2 int) []int {
		out := make([]int, 0, 5)
		if kicker1 != 0 { out = append(out, kicker1) }
		if kicker2 != 0 { out = append(out, kicker2) }
		for i := len(ranks) - 1; i >= 0; i-- {
			if ranks[i] != kicker1 && ranks[i] != kicker2 {
				out = append(out, ranks[i])
			}
		}
		return out
	}

	switch {
	case straight_rank != 0 && flush:
		return Hand{ STRAIGHT_FLUSH, []int{straight_rank} }
	case quadruple != 0:
		return Hand{ FOUR_OF_A_KIND, kickers(quadruple, 0) }
	case triple != 0 && pair1 != 0:
		return Hand{ FULL_HOUSE, kickers(triple, pair1) }
	case flush:
		return Hand{ FLUSH, kickers(0, 0) }
	case straight_rank != 0:
		return Hand{ STRAIGHT, []int{straight_rank} }
	case triple != 0:
		return Hand{ THREE_OF_A_KIND, kickers(triple, 0) }
	case pair2 != 0:
		return Hand{ TWO_PAIR, kickers(pair1, pair2) }
	case pair1 != 0:
		return Hand{ PAIR, kickers(pair1, 0) }
	default:
		return Hand{ HIGH_CARD, kickers(0, 0) }
	}
	panic("bad hand")
}

type Stats struct {
	win, loss int
}

const PLAYERS = 8

func RunHand(deck []Card, result map[int]map[CardPair]Stats) {
	community, deck := deck[:5], deck[5:]

	hands := make([][]Card, PLAYERS)
	for i, _ := range hands {
		hands[i], deck = deck[0:2], deck[2:]
	}

	ratings := make([]Hand, PLAYERS)
	best_idx := 0
	for i, hand := range hands {
		ratings[i] = BestOfSeven(append(community, hand[0], hand[1]))
		if CmpHand(ratings[i], ratings[best_idx]) > 0 {
			best_idx = i
		}
	}

	for i, hand := range hands {
		pair := NewPair(hand[0], hand[1])
		stats := result[pair]
		if CmpHand(ratings[i], ratings[best_idx]) >= 0 {
			stats.win++
		} else {
			stats.loss++
		}
		result[pair] = stats
	}
}

const HANDS = 100000

func RunHands() map[CardPair]Stats {
	d := Deck()
	result := make(map[CardPair]Stats)

	for i := 0; i < HANDS; i++ {
		d2 := make([]Card, 52)

		for i, p := range rand.Perm(52) {
			d2[i] = d[p]
		}

		RunHand(d2, result)
	}
	return result
}

func CompareHands() {
	d := Deck()
	d2 := make([]Card, 52)

	for i, p := range rand.Perm(52) {
		d2[i] = d[p]
	}
	hand1, hand2 := d2[:7], d2[7:14]

	sort.Sort(CardSort(hand1))
	sort.Sort(CardSort(hand2))

	cmp := CmpHand(BestOfSeven(hand1), BestOfSeven(hand2))
	var symbol byte
	switch {
	case cmp == 0: symbol = '='
	case cmp < 0: symbol = '<'
	case cmp > 0: symbol = '>'
	}
	fmt.Printf("%s %c %s\n\n", hand1, symbol, hand2)
}

func main () {
  f, err := os.Create("prof")
  if err != nil {
    log.Fatal(err)
  }
  pprof.StartCPUProfile(f)
  defer pprof.StopCPUProfile()

	for i := 0; i < 1000000; i++ {
		CompareHands()
	}

	// result := RunHands()

	// for k,v := range result {
	// 	fmt.Printf("%s: %.2f\n", k.String(), 100.0 * float64(v.win) / float64(v.win+v.loss))
	// }
}
