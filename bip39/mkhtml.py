# Little script for generating a BIP39 wordlist print-out.

import math

print '''
<style>
h1 {
  text-align: center;
  page-break-before: always;
  font-family: monospace;
}

.pagenum {
  text-align: center;
  font-family: monospace;
}

.colset {
  display: flex;
  font-size: 11px;
  justify-content: space-between;
}

pre {
  width: 115px;
}

</style>
'''

with open('wordlist.txt') as f:
  words = [word.strip() for word in f]

ROWS = 64
COLS = 6
PAGES = int(math.ceil((0.0+len(words))/(ROWS*COLS)))
for pageIdx in range(PAGES):
  print '''<h1>BIP 39 Wordlist</h1>
  <div class="colset">'''

  for colIdx in range(COLS):
    print '''<pre>'''
    for rowIdx in range(ROWS):
      idx = pageIdx*ROWS*COLS + colIdx*ROWS + rowIdx
      if idx < len(words):
        print '%04d %03X %s' % (idx, idx, words[idx])
    print '''</pre>'''



  print '''</div>
  <div class="pagenum">%d/%d</div>''' % (pageIdx + 1, PAGES)
