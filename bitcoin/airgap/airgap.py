#!/usr/bin/env python

import click
import hashlib
import binascii
import csv


try:
    from typing import List, Text
except ImportError: pass

def bytes2int(str):
    return int(binascii.hexlify(str), 16)
 
#from builtins import bytes, bytearray
#from builtins import int, str


if hasattr(int, "from_bytes"):
    int_from_bytes = int.from_bytes
else:
    def int_from_bytes(data, byteorder, signed=False):
        assert byteorder == 'big'
        assert not signed

        return int(binascii.hexlify(data), 16)


if hasattr(int, "to_bytes"):
    def int_to_bytes(integer, length):
        return integer.to_bytes(length, byteorder='big')    
else:
    def int_to_bytes(integer, length):
        hex_string = '%x' % integer
        return binascii.unhexlify(hex_string.zfill(length * 2))


def seed_to_sk(seed_bytes, index):
    # type: (bytes, int) -> int
    """Given the contents of a seed file, generate a secret key as an int."""
    phrase = b' '.join(seed_bytes.split())
    phrase += b' ' + str(index).encode('ascii') + b'\n'
    
    return int_from_bytes(hashlib.sha256(phrase).digest(), byteorder='big')


def sk_to_pk(sk):
    # type: (int) -> bytes
    """Converts private keys to public keys.
    
    The input is an integer as returned by seed_to_sk. The output is an
    uncompressed secp256k1 public key, as a byte string, as described in SEC 1
    v2.0 section 2.3.3.
    """
    from cryptography.hazmat.primitives.asymmetric import ec
    from cryptography.hazmat import backends
    priv_key = ec.derive_private_key(sk, ec.SECP256K1(), backends.default_backend())
    k = priv_key.public_key().public_numbers().encode_point()
    return k


def sk_to_wif(sk):
    # type: (int) -> Text
    """Converts a private key to WIF format.

    The input is an integer as returned by seed_to_sk. The output is an
    "uncompressed" WIF-format private key, ready to be added to your favorite
    bitcoin program.
    """
    from pycoin import key
    k = key.Key(secret_exponent=sk, netcode='BTC')
    return k.wif(use_uncompressed=True)

def pk_to_addr(pk):
    # type: (bytes) -> Text
    """Converts a public key to an uncompressed bitcoin address.

    The input is an uncompressed secp256k1 public key, as a byte string, as
    described in SEC 1 v2.0 section 2.3.3. The output is an uncompressed bitcoin
    address.
    """
    from cryptography.hazmat.primitives.asymmetric import ec
    #    from cryptography.hazmat import backends
    from pycoin import key
    pk_nums = ec.EllipticCurvePublicNumbers.from_encoded_point(ec.SECP256K1(), pk)
    
    k = key.Key.from_sec(pk, netcode='BTC')
    # k = key.Key(public_pair=(pk_nums.x, pk_nums.y), netcode='BTC')
    return k.address(use_uncompressed=True)




def base58(b):
    # type(bytes) -> bytes
    """Convert a byte sequence to base58. Borrowed from Pycoin.

    b: bytes to convert.
    """
    leading_zeros = 0
    while leading_zeros < len(b) and b[leading_zeros] == 0:
        leading_zeros += 1
    v = int.from_bytes(b, byteorder='big')

    l = bytearray()
    while v > 0:
        v, mod = divmod(v, len(BASE58_ALPHABET))
        l.append(BASE58_ALPHABET[mod])
    l.extend([BASE58_ALPHABET[0]] * leading_zeros)
    l.reverse()
    return bytes(l)



class Deriver(object):
    def __init__(self, seed):
        # type: (List[str]) -> None

        # Normalize,
        self._seed = [s.strip().lower() for s in seed]

    def private_key(self, index):
        # type: (int) -> int
        phrase = ' '.join(self._seed + [str(index)]) + '\n'
        sk = hashlib.sha256(phrase.encode('ascii')).digest()
        return int.from_bytes(sk, byteorder='big')

    def wif(self, index):
        # type: (int) -> bytes
        sk = self.private_key(index)
        d = b'\x80' + sk.to_bytes(32, byteorder='big')
        h = hashlib.sha256(hashlib.sha256(d).digest()).digest()

        return base58(d + h[:4])
#        from pycoin import key
#       
#        k = key.Key(sk)
#        return k.wif(use_uncompressed=True)

@click.group()
def cli():
    pass

@cli.command()
@click.argument('seed_in', type=click.File('r'))
@click.argument('wif_out', type=click.File('w'))
@click.option('--start', default=0, help='First index for which to generate a WIF')
@click.option('--count', default=10, help='Number of WIFs to generate, starting from --start')
def wif(seed_in, wif_out, start, count):
    """Generates WIFs from a file with seed words."""
    with csv.writer(wif_out, delimeter='\t') as tsv_out:
        for idx in range():
            pass
    

if __name__ == '__main__':
    cli()
