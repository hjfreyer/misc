
package main

import (
        "fmt"
        "log"
        "os"
        "bufio"

        "image"
        "image/png"
        "code.google.com/p/draw2d/draw2d"
)


func saveToPngFile(filePath string, m image.Image) {
        f, err := os.Open(filePath, os.O_CREAT|os.O_WRONLY, 0600)
        if err != nil {
                log.Println(err)
                os.Exit(1)
        }
        defer f.Close()
        b := bufio.NewWriter(f)
        err = png.Encode(b, m)
        if err != nil {
                log.Println(err)
                os.Exit(1)
        }
        err = b.Flush()
        if err != nil {
                log.Println(err)
                os.Exit(1)
        }
        fmt.Printf("Wrote %s OK.\n", filePath)
}

func main() {
	r := image.Rect(0,0,200,200)
        i := image.NewRGBA(r)
        gc := draw2d.NewGraphicContext(i)
        gc.MoveTo(10.0, 10.0)
        gc.LineTo(100.0, 10.0)
        gc.Stroke()
        saveToPngFile("TestPath.png", i)
}