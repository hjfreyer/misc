
package main

import (
	"fmt"
)

type Move int

const (
	COOPERATE Move = iota
	DEFECT
)

func GetPoints(aMove, bMove Move) (int, int) {
	switch {
	case aMove == COOPERATE && bMove == COOPERATE:
		return 10, 10
	case aMove == DEFECT && bMove == COOPERATE:
		return 15, -10
	case aMove == COOPERATE && bMove == DEFECT:
		return -10, 15
	case aMove == DEFECT && bMove == DEFECT:
		return -5, -5
	}
	panic("")
}

type Player struct {
	Strategy
	Score int
}

type Strategy interface {
	Name() string
	NextMove(mine, theirs []Move) Move
}

func Compete(a, b *Player) {
	aHist := make([]Move, 0)
	bHist := make([]Move, 0)

	for i := 0; i < 100; i++ {
		aMove, bMove := a.NextMove(aHist, bHist), b.NextMove(bHist, aHist)
		aScore, bScore := GetPoints(aMove, bMove)

		aHist, bHist = append(aHist, aMove), append(bHist, bMove)
		a.Score += aScore
		b.Score += bScore
	}
}

func Round(players []*Player) {
	for i := 0; i < len(players); i++ {
		for j := i + 1; j < len(players); j++ {
			Compete(players[i], players[j])
		}
	}
}

type ConstantCoop struct{}
func (ConstantCoop) Name() string { return "ConstantCoop" }
func (ConstantCoop) NextMove(mine, theirs []Move) Move { return COOPERATE }

type ConstantDef struct{}
func (ConstantDef) Name() string { return "ConstantDef" }
func (ConstantDef) NextMove(mine, theirs []Move) Move { return DEFECT }

var STRATS = []Strategy{
	ConstantCoop{},
	ConstantDef{},
}

func main() {
	players := []*Player{
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
		&Player{Strategy : STRATS[0]},
	}

	Round(players)
	for _, p := range players {
		fmt.Println(*p)
	}
}