package nn2048

import (
	"fmt"
	"math/rand"
)

type Strategy interface {
	NextMove(b Board) Direction
}

type RandomStrategy struct{}

func (RandomStrategy) NextMove(b Board) Direction {
	order := rand.Perm(4)
	for _, d := range order {
		_, moved := Move(b, Up+Direction(d))
		if moved {
			return Direction(d)
		}
	}
	panic("NextMove called when no move available")
}

func PrintBoard(b Board) {
	idx := 0
	for y := 0; y < 4; y++ {
		for x := 0; x < 4; x++ {
			fmt.Printf("%c", b[idx])
			idx++
		}
		fmt.Println()
	}
	fmt.Println()
}

func PlayGame(s Strategy, r *rand.Rand) ([]Board, error) {
	var boards []Board
	b := NewBoard(4)
	b, err := AddRandomTile(b, r)
	if err != nil {
		panic(err)
	}
	for !GameOver(b) {
		b, _ = Move(b, s.NextMove(b))
		boards = append(boards, b)
		b, err = AddRandomTile(b, r)
		if err != nil {
			panic(err)
		}
	}

	return boards, nil
}

func pickDir(move []float64, r *rand.Rand) Direction {
	rand := r.Float64()
	for d := Up; d < LastDir; d++ {
		if rand < move[d] {
			return d
		}
		rand -= move[d]
	}
	panic(fmt.Sprintf("sum(prob) < 1: %f", rand))
}

func DirStr(d Direction) string {
	switch d {
	case Up:
		return "Up"
	case Down:
		return "Down"
	case Left:
		return "Left"
	case Right:
		return "Right"
	}
	panic("wut")
}
