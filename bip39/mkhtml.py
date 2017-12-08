# Little script for generating a BIP39 wordlist print-out.

import math

print '''
<style>
.page {
  page-break-before: always;
  display: flex;
  flex-direction: column;
  align-items: center;
  font-family: monospace;
}

.pagenum {
  font-family: monospace;
}

pre {
  marign: auto;
  font-size: 11px;
}

</style>
'''

with open('wordlist.txt') as f:
  words = [word.strip() for word in f]

ROWS = 64
COLS = 5
PAGES = int(math.ceil((0.0+len(words))/(ROWS*COLS)))

maxWordLen = max(len(x) for x in words)

def pageIdxToWordIdx(page, row, col):
  idx = page * ROWS * COLS
  idx += col * ROWS
  idx += row
  return idx

for pageIdx in range(PAGES):
  pageStartIdx = pageIdx*ROWS*COLS
  print '''<div class="page"><h1>BIP 39 Wordlist</h1><pre>'''

  for colIdx in range(COLS):
    if pageIdxToWordIdx(pageIdx, 0, colIdx) < len(words):
      print 'DECI OCTA WORD' + (' ' * (maxWordLen - 4)) + ' ',
  print
  for colIdx in range(COLS):
    if pageIdxToWordIdx(pageIdx, 0, colIdx) < len(words):
      print '==== ==== ' + ('=' * maxWordLen) + ' ',
  print
  for rowIdx in range(ROWS):
    for colIdx in range(COLS):
      idx = pageIdxToWordIdx(pageIdx, rowIdx, colIdx)
      if idx < len(words):
        word = words[idx]
        word += ' ' * (maxWordLen - len(word))
        print '%04d %04o %s ' % (idx, idx, word),
    print
  print '''</pre>'''
  print '''<div class="pagenum">%d/%d</div></div>''' % (pageIdx + 1, PAGES)
