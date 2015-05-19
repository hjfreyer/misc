package main

import (
	"fmt"
	"strings"
	"time"
)

const (
	hexSecInDay = 65536
	hexSec      = time.Hour * 24 / hexSecInDay
)

func main() {
	t := time.Now().In(time.UTC)
	hour, min, sec := t.Clock()
	nsec := t.Nanosecond()

	timeIntoDay := (time.Duration(hour)*time.Hour +
		time.Duration(min)*time.Minute +
		time.Duration(sec)*time.Second +
		time.Duration(nsec)*time.Nanosecond)

	hexTime := int64(timeIntoDay / hexSec)
	fmt.Println(strings.ToUpper(fmt.Sprintf("%x", hexTime)))

}
