package strats

import "github.com/hjfreyer/misc/nn2048"

type Strategy struct{}

type Evaluator interface {
	Evaluate(b nn2048.Board) float64
}

type MoveSelector interface {
}

type RolloutEvaluator struct {
}

func (s Strategy) NextMove(b nn2048.Board) nn2048.Direction {
	for d := nn2048.Up; d < nn2048.LastDir; d++ {
		_, moved := nn2048.Move(b, d)
		if moved {
			return d
		}
	}
	panic("no moves")
}
