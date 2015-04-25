package main

import (
	"fmt"
	"math/rand"
	"sort"

	"github.com/pkg/profile"
)

const (
	numPlayers           = 10000
	avgNumGamesPerPlayer = 100

	dumbLuck    = 0.05
	skillStdDev = 0.20

	noLostStarsBelow = 11
	minStreak        = 3
)

type player struct {
	skill  float64
	stars  int
	streak int
}

type arena struct {
	r *rand.Rand

	allPlayers     []*player
	playersByStars map[int]map[*player]bool

	bedrockStars int
	streakStars  int
}

func (a *arena) fight() {
	playerOne := a.allPlayers[a.r.Intn(len(a.allPlayers))]
	playerTwo := a.findPlayerNear(playerOne.stars, playerOne)

	oneWon := firstPlayerWins(playerOne.skill, playerTwo.skill, a.r)

	var winner, loser *player
	if oneWon {
		winner, loser = playerOne, playerTwo
	} else {
		winner, loser = playerTwo, playerOne
	}

	loser.streak = 0
	winner.streak++
	if loser.stars < noLostStarsBelow {
		a.bedrockStars++
	} else {
		a.changeStars(loser, -1)
	}

	if winner.streak > 2 {
		a.streakStars++
		a.changeStars(winner, 2)
	} else {
		a.changeStars(winner, 1)
	}
}

func (a *arena) changeStars(p *player, delta int) {
	oldStars := p.stars
	p.stars += delta
	m := a.playersByStars[p.stars]
	if m == nil {
		m = make(map[*player]bool)
		a.playersByStars[p.stars] = m
	}
	m[p] = true
	delete(a.playersByStars[oldStars], p)
}

func firstPlayerWins(skill1, skill2 float64, r *rand.Rand) bool {
	advantageOne := skill1 - skill2
	chanceOneWins := 0.5 + advantageOne/2

	roll := r.Float64()
	if roll < dumbLuck {
		return true
	}
	if roll > 1-dumbLuck {
		return false
	}
	return roll < chanceOneWins
}

func (a *arena) appendPlayersWithStars(stars int, slice []*player) []*player {
	for p := range a.playersByStars[stars] {
		slice = append(slice, p)
	}
	return slice
}

func (a *arena) findPlayerNear(stars int, not *player) *player {
	players := a.appendPlayersWithStars(stars, nil)
	for window := 1; window == 1 || len(players) < 2; window++ {
		players = a.appendPlayersWithStars(stars+window, players)
		players = a.appendPlayersWithStars(stars-window, players)
	}
	for {
		p := players[a.r.Intn(len(players))]
		if p != not {
			return p
		}
	}
}

func (a *arena) print() {
	fmt.Println("Stars: ")
	var levelsPresent []int
	for key := range a.playersByStars {
		levelsPresent = append(levelsPresent, key)
	}
	sort.Ints(levelsPresent)
	for _, star := range levelsPresent {
		fmt.Printf("  %d: %d\n", star, len(a.playersByStars[star]))
	}
	fmt.Println("Bedrock: ", a.bedrockStars)
	fmt.Println("Streak:  ", a.streakStars)
}

func main() {
	r := rand.New(rand.NewSource(4))

	players := make([]*player, numPlayers)
	m := make(map[int]map[*player]bool)
	m[0] = make(map[*player]bool)
	for i := range players {
		players[i] = &player{
			skill: r.NormFloat64() * skillStdDev,
		}
		m[0][players[i]] = true
	}

	a := arena{
		r:              r,
		allPlayers:     players,
		playersByStars: m,
	}
	defer profile.Start().Stop()
	for i := 0; i < numPlayers*avgNumGamesPerPlayer; i++ {

		a.fight()
	}
	a.print()
}
