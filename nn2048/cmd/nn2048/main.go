package main

import (
	"encoding/json"
	"log"
	"math/rand"
	"os"
	"time"

	"github.com/hjfreyer/misc/nn2048"
	"github.com/hjfreyer/misc/nn2048/strats"
	"gopkg.in/mgo.v2/bson"
)

type TestObj struct {
	ID  bson.ObjectId `bson:"_id"`
	Val string
}

func max(i nn2048.Board) int {
	var res int
	for _, c := range i {
		n := nn2048.ToInt(c)
		if res < n {
			res = n
		}
	}
	return res
}

type Summary struct {
	Strategy string  `json:"strategy"`
	Scores   []int32 `json:"scores"`
	MaxTiles []int32 `json:"maxTiles"`
}

func main() {
	var rs strats.Strategy
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	const count = 20000
	/*
		s, err := mgo.Dial("localhost")
		if err != nil {
			log.Fatal(err)
		}
		games := s.DB("nn2048").C("games")
		states := s.DB("nn2048").C("states")
	*/
	s := Summary{
		Strategy: "random",
		Scores:   make([]int32, count),
		MaxTiles: make([]int32, count),
	}
	for i := 0; i < count; i++ {
		boards, err := nn2048.PlayGame(rs, r)
		if err != nil {
			log.Fatal(err)
		}
		s.Scores[i] = int32(len(boards))
		s.MaxTiles[i] = int32(max(boards[len(boards)-1]))

		/*
			gameID := bson.NewObjectId()
			if err := games.Insert(Game{
				ID:          gameID,
				Strategy:    "rand",
				Score:       len(boards),
				LargestTile: max(boards[len(boards)-1]),
			}); err != nil {
				log.Fatal(err)
			}

			serialBoards := make([]interface{}, len(boards))
			for i, b := range boards {
				serialBoards[i] = State{
					GameID: gameID,
					Board:  tostr(b),
				}
			}

			if err := states.Insert(serialBoards...); err != nil {
				log.Fatal(err)
			}*/
	}

	e := json.NewEncoder(os.Stdout)
	e.Encode(s)
}
