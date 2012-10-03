
import pygame
import math

import numpy
import fractions
import wave

from scipy.io import wavfile

N = 2**8
sample_rate = 44100

tau = 2*math.pi

notes = [440,
         466.16,
         493.88,
         523.25,
         554.37,
         587.33,
         622.25,
         659.26,
         698.46,
         739.99,
         783.99,
         830.61]


def GetNoteDuration(layer):
    return sample_rate / 10


def GetNoteForLayer(layer):
    return 7 * (layer - 1) % 12
    # return 0

def GetAmpForLayer(layer):
    return 0.5 / layer


def GetQuarterForLayer(degree, layer, length):
    if degree % layer != 0:
        return numpy.zeros(length)
    written_len = 0
    segs = []

    for i in range(layer):
        seglen = int((i + 1.0) * length / layer) - written_len
        written_len += seglen

        if fractions.gcd(i, layer) == 1:
        # if True:
            note_dur = min(GetNoteDuration(layer), seglen)
            segs.append(GetNote(GetNoteForLayer(layer),
                                GetAmpForLayer(layer),
                                note_dur))
            segs.append(numpy.zeros(seglen - note_dur))
        else:
            segs.append(numpy.zeros(seglen))

    return numpy.concatenate(segs)


def GetQuarter(degree, length):
    out = numpy.zeros(length)

    for i in range(degree):
        out += GetQuarterForLayer(degree, i + 1, length)

    return out

def GetMeasures(degrees, length):
    return numpy.concatenate([GetMeasure(d, length)
                              for d in degrees])

def GetMeasure(degree, length):
    return numpy.tile(GetQuarter(degree, length), 4)


def GetNote(note, amp, length):
    return GetSineArray(GetFrameLengthForNote(note), amp, length)


def GetSineArray(frames_per_cycle, amp, length):
    cycles = int(length / frames_per_cycle)
    frames_of_cycling = cycles * frames_per_cycle

    array = numpy.arange(0, frames_of_cycling, dtype=numpy.double)
    array = (array % frames_per_cycle) / frames_per_cycle
    array = numpy.concatenate([array, numpy.zeros(length - len(array))])
    array = amp * numpy.sin(2 * math.pi * array)

    return array


def GetFrameLengthForNote(note):
    octave = note / 12
    note = note % 12

    hz = 2**octave * notes[note]
    return sample_rate / hz

def ConvertToWave(sound):
    # Deal with clipping
    maxArray = numpy.repeat(1, len(sound))
    minArray = numpy.repeat(-1, len(sound))

    print numpy.nonzero(sound > 1)
    print numpy.nonzero(sound < -1)

    sound = numpy.minimum(sound, maxArray)
    sound = numpy.maximum(sound, minArray)

    sound = sound * 2**15
    sound = sound.astype(numpy.int16)

    return sound


def Play(sound):
    pygame.mixer.Sound(ConvertToWave(sound)).play()


def drawDots(deg, m, n, x, y, w, h):
    screen.fill(0)

    for i in range(4):
        for beat in range(deg):
            x_spot = beat * w / deg / 4 + x
            note =  deg / fractions.gcd(deg, beat)

            color = (255, 0,0) if i == m and beat == n else (255, 255, 255)

            pygame.draw.circle(screen, color, (x_spot, (y+h) - h/note), 25 / (1+math.log(note+1)))
        x += w/4

    pygame.display.flip()

def doIt(deg):
    Play(GetMeasure(deg, sample_rate))
    clock = pygame.time.Clock()
    time = 0

    while time < 4:
        time += clock.tick(60) / 1000.0

        tt = time % 4

        mtime = time % 1

        drawDots(deg, int(tt), int(mtime*deg), 100, 100, 600, 400)


def Write(sound, name):
    wavfile.write(name, sample_rate, ConvertToWave(sound))


screen = None
def main(argv):
    global screen
    pygame.mixer.pre_init(sample_rate, -16, 1)
    pygame.mixer.init()


    # screen = pygame.display.set_mode((800, 600)) #make screen
#    do_it()


if __name__ == '__main__':
    import sys
    main(sys.argv)


