
from .context import airgap


from typing import List, Dict, Any, Text
import unittest
from hypothesis import given, assume
import hypothesis
import os.path
import hypothesis.strategies as st
from parameterized import parameterized, param
import json
import os

import binascii
import glob
import sys
import subprocess
import tempfile
import shutil

#from builtins import bytes

SCRIPT_FILE = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'airgap.py')
TESTDATA_DIR = os.path.join(os.path.dirname(__file__), 'data')


def makeTestCaseDirs():
    for case in glob.glob(TESTDATA_DIR + '/files/*'):
        yield case
        #        yield os.path.join(TESTDATA_DIR, case)

def makeWifTestCases():
    for case_dir in makeTestCaseDirs():
        for seed in glob.glob(case_dir + '/seeds/*.txt'):
            yield param(seed,
                        os.path.join(case_dir, 'wif.tsv'))

def makePubkeyOutTestCases():
    for case_dir in makeTestCaseDirs():
        for seed in glob.glob(case_dir + '/seeds/*.txt'):
            yield param(seed,
                        os.path.join(case_dir, 'pubkey.tsv'))

def makePubkeyToAddrTestCases():
    for case_dir in makeTestCaseDirs():
        yield param(os.path.join(case_dir, 'pubkey.tsv'),
                    os.path.join(case_dir, 'addr.tsv'))


        
class TestStringMethods(unittest.TestCase):

    def setUp(self):
        self.tmp_dir = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(tmp_dir)
        
    
    def call_it(self, args):
        subprocess.check_call([sys.executable, SCRIPT_FILE] + args)

    @parameterized.expand(makeWifTestCases)
    def test_wifs(self, seed_path, wif_path):
        # type: (Text, Text) -> None
        actual_wif_path = os.path.join(self.tmp_dir, 'wif.out.tsv')
        self.call_it(['wif', seed_path, actual_wif_path])
        with open(wif_path) as f:
            expected_wif = f.read()
        with open(actual_wif_path) as f:
            actual_wif = f.read()

        self.assertEqual(expected_wif, actual_wif)
        
if __name__ == '__main__':
    unittest.main()
        
