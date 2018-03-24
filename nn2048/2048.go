package nn2048

import (
	"errors"
	"math/rand"
	"strings"
)

type Direction int

const (
	Up Direction = iota
	Down
	Left
	Right
	LastDir
)

type Board string

func NewBoard(size int) Board {
	return Board(strings.Repeat("0", size*size))
}

func ToInt(b rune) int {
	if '0' <= b && b <= '9' {
		return int(b) - '0'
	}
	return int(b) - 'A' + 10
}

func Size(b Board) int {
	if len(b) == 16 {
		return 4
	}

	for i := 0; true; i++ {
		sq := i * i
		if sq == len(b) {
			return i
		}
		if len(b) < sq {
			panic("board has non-square size")
		}
	}
	panic("can't get here")
}

// Adds a tile in a random position
func AddRandomTile(b Board, r *rand.Rand) (Board, error) {
	free := FreeCells(b)
	if len(free) == 0 {
		return "", errors.New("Full board!")
	}

	value := byte('1')
	if r.Float64() < 0.1 {
		value = byte('2')
	}
	buff := []byte(b)
	buff[free[r.Intn(len(free))]] = value
	return Board(buff), nil
}

func FreeCells(b Board) []int {
	var res []int
	for i, c := range b {
		if c == '0' {
			res = append(res, i)
		}
	}
	return res
}

// Move tiles on the grid in the specified direction
func Move(b Board, d Direction) (Board, bool) {
	size := Size(b)
	xs, ys := buildTraversals(size, d)
	moved := false

	merged := make([]bool, len(b))
	out := []byte(b)
	for _, x := range xs {
		for _, y := range ys {
			cellIdx := y*size + x
			if b[cellIdx] == '0' {
				continue
			}

			farthest, next := findFarthestPosition(b, cellIdx, size, d)
			if 0 <= next && next < len(b) && b[next] == b[cellIdx] && !merged[next] {
				out[cellIdx] = '0'
				out[next] = succ(out[next])
				merged[next] = true
				moved = true
			} else if cellIdx != farthest {
				out[farthest] = out[cellIdx]
				out[cellIdx] = '0'
				moved = true
			}
		}
	}
	return Board(out), moved
}

func succ(b byte) byte {
	if b == '9' {
		return 'A'
	}
	return b + 1
}

func GameOver(b Board) bool {
	size := Size(b)
	for idx, cellRune := range b {
		cell := byte(cellRune)
		if cell == '0' {
			return false
		}
		for d := Up; d < LastDir; d++ {
			next := move(idx, size, d)
			if 0 <= next && next < len(b) && cell == b[next] {
				return false
			}
		}
	}
	return true
}

// Build a list of positions to traverse in the right order
func buildTraversals(size int, d Direction) ([]int, []int) {
	xs := make([]int, size)
	ys := make([]int, size)
	for i := 0; i < size; i++ {
		if d == Down {
			ys[i] = 3 - i
		} else {
			ys[i] = i
		}
		if d == Right {
			xs[i] = 3 - i
		} else {
			xs[i] = i
		}
	}
	return xs, ys
}

func findFarthestPosition(b Board, idx int, size int, d Direction) (int, int) {
	previous := idx
	next := move(idx, size, d)
	for 0 <= next && next < len(b) && b[next] == '0' {
		previous = next
		next = move(next, size, d)

	}
	return previous, next
}

func move(idx, size int, d Direction) int {
	switch d {
	case Up:
		return idx - size
	case Right:
		if idx%size == size-1 {
			return -1
		}
		return idx + 1
	case Down:
		return idx + size
	case Left:
		if idx%size == 0 {
			return -1
		}
		return idx - 1
	}
	panic("bad direction")
}
