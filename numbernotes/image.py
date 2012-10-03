
import Image
import ImageDraw
import fractions
import math
import itertools

def Add(t, *l):
    assert len(t) == len(l)
    return tuple([x + y for x, y in zip(t, l)])

SIZE = (1920 / 2, 1080 / 2)
STAFF_MARGIN = 75
STAFF_START = (STAFF_MARGIN, STAFF_MARGIN)
STAFF_SIZE = Add(SIZE, -2 * STAFF_MARGIN, -2 * STAFF_MARGIN)

NOTE_BASE_RADIUS = 20

BACKGROUND = (255, 255, 255)
INACTIVE_NOTE = (0, 0, 0)
ACTIVE_NOTE = (0, 0, 255)

ANTIALIAS = 4
BEATS = 4

def RadiusForLayer(layer):
    return NOTE_BASE_RADIUS * (layer ** math.log(0.5))


def ImagesForDegrees(degrees, frames_per_measure):
    return itertools.chain(*[ImagesForDegree(d, frames_per_measure)
                             for d in degrees])

def ImagesForDegree(degree, total_frame):
    for i in range(total_frame):
        yield ImageForDegree(degree, i, total_frame)


def ImageForDegree(degree, frame, total_frame):
    im = Image.new("RGBA", Trans(SIZE))
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0) + im.size, fill=BACKGROUND)

    active_beat = int(float(frame) / total_frame * degree * BEATS)

    for i in range(degree):
        line_y = STAFF_SIZE[1] - float(STAFF_SIZE[1]) / (i + 1)
        Line(d,
             Add(STAFF_START, 0, line_y) +
             Add(STAFF_START, STAFF_SIZE[0], line_y),
             fill=(230,230,230),
             width=ANTIALIAS/2)

    for beat in range(BEATS):
        for i in range(degree):
            note =  degree / fractions.gcd(degree, i)

            x = float(i) * STAFF_SIZE[0] / degree
            x = (x + float(beat) * STAFF_SIZE[0]) / BEATS
            y = STAFF_SIZE[1] - float(STAFF_SIZE[1]) / note

            fill = (ACTIVE_NOTE if degree * beat + i == active_beat
                    else INACTIVE_NOTE)

            Circle(d, Add(STAFF_START, x, y), RadiusForLayer(note), fill=fill)

    return im.resize(SIZE, Image.ANTIALIAS)


def Circle(d, xy, radius, fill):
    pos = Trans(xy)
    rad = Trans(radius)
    d.ellipse(Add(pos, -rad, -rad) + Add(pos, rad, rad), fill=fill)

def Line(d, xy, **kw):
    d.line(Trans(xy), **kw)

def Trans(t):
    if isinstance(t, tuple):
        return tuple([x * ANTIALIAS for x in t])
    else:
        return t * ANTIALIAS
