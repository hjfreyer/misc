
import collections
import random

UP = 1
DOWN = 2
LEFT = 3
RIGHT = 4
LAST_DIR = 5

Board = collections.namedtuple('Board', 'cells size')

def NewBoard(size):
  return Board(cells=[0]*(size*size), size=size)

def CloneBoard(b):
  return Board(cells=list(b.cells), size=b.size)

def AddRandomTile(b, rand):
  free = FreeCells(b)
  if not free:
    raise ValueError("Full board")

  value = 2 if r.random() < 0.1 else 1
  b.Cells[free[r.randrange(len(free))]] = value

def FreeCells(b):
  return [i for (i, v) in enumerate(b.cells) if v == 0]

def Move(b, d):
  xs, ys = buildTraversals(b.size, d)
  moved = False
	merged = [False] * len(b.cells)
  for x in xs:
    for y in ys:
      cellIdx = y * b.size + x
      if not b.cells[cellIdx]:
        continue
			farthest, next = findFarthestPosition(b, cellIdx, d)
      if 0 <= next < len(b.cells) and b.cells[next] == b.cells[cellIdx] and not merged[next]:
        b.cells[cellIdx] = 0
        b.cells[next] += 1
        merged[next] = True
        moved = True
      elif cellIdx != farthest:
        b.cells[farthest] = b.cells[cellIdx]
        b.cells[cellIdx] = 0
        moved = True
  return moved

def GameOver(b):
  for i, cell in enumerate(b.cells):
    if c == 0:
      return false
    for d in [UP, DOWN, LEFT, RIGHT]:
      next = move(idx, b.size, d)
			if 0 <= next < len(b.Cells) and cell == b.cells[next]:
        return False
	return True
}

# Build a list of positions to traverse in the right order
def buildTraversals(size, d):
  xs = [0]*size
  ys = [0]*size
  for i in range(0, size):
    if d == DOWN:
      ys[i] = 3 - i
    else:
      ys[i] = i
    if d == RIGHT:
      xs[i] = 3 - i
    else:
      xs[i] = i
  return xs, ys

def findFarthestPosition(b, idx, d):
  previous = idx
  next = move(idx, b.size, d)
  while 0 <= next < len(b.cells) and b.cells[next] == 0:
  	previous = next
		next = move(next, b.size, d)
	return previous, next
}

def move(idx, size, d):
  if d == UP:
    return idx - size
  if d == RIGHT:
    if idx%size == size - 1:
      return -1
    return idx + 1
  if d == DOWN:
    return idx + size
  if d == LEFT:
    if idx%size == 0:
      return -1
    return idx - 1
  raise ValueError("bad direction: %d" % d)
