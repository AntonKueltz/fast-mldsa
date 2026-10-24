from random import randbytes, randint
from unittest import TestCase

from fastmldsa import ML_DSA, ParameterSet, SigningKey, VerificationKey

ROUNDS = 1000


class Test_ML_DSA(TestCase):
    def test_mldsa44(self):
        dsa = ML_DSA(ParameterSet.ML_DSA_44)
        vk, sk = dsa.key_gen()

        for _ in range(ROUNDS):
            msg, ctx = randbytes(randint(8, 1024)), randbytes(randint(0, 128))
            sig = sk.sign(msg, ctx)
            self.assertTrue(vk.verify(sig, msg, ctx))

            vk_bytes = randbytes(1312)
            vk_ = VerificationKey.from_bytes(vk_bytes)
            self.assertEqual(bytes(vk_), vk_bytes)

            sk_bytes = randbytes(2560)
            try:
                sk_ = SigningKey.from_bytes(sk_bytes)
                self.assertEqual(bytes(sk_), sk_bytes)
            except ValueError:
                pass

    def test_mldsa65(self):
        dsa = ML_DSA(ParameterSet.ML_DSA_65)
        vk, sk = dsa.key_gen()

        for _ in range(ROUNDS):
            msg, ctx = randbytes(randint(8, 1024)), randbytes(randint(0, 128))
            sig = sk.sign(msg, ctx)
            self.assertTrue(vk.verify(sig, msg, ctx))

            vk_bytes = randbytes(1952)
            vk_ = VerificationKey.from_bytes(vk_bytes)
            self.assertEqual(bytes(vk_), vk_bytes)

            sk_bytes = randbytes(4032)
            try:
                sk_ = SigningKey.from_bytes(sk_bytes)
                self.assertEqual(bytes(sk_), sk_bytes)
            except ValueError:
                pass

    def test_mldsa87(self):
        dsa = ML_DSA(ParameterSet.ML_DSA_87)
        vk, sk = dsa.key_gen()

        for _ in range(ROUNDS):
            msg, ctx = randbytes(randint(8, 1024)), randbytes(randint(0, 128))
            sig = sk.sign(msg, ctx)
            self.assertTrue(vk.verify(sig, msg, ctx))

            vk_bytes = randbytes(2592)
            vk_ = VerificationKey.from_bytes(vk_bytes)
            self.assertEqual(bytes(vk_), vk_bytes)

            sk_bytes = randbytes(4896)
            try:
                sk_ = SigningKey.from_bytes(sk_bytes)
                self.assertEqual(bytes(sk_), sk_bytes)
            except ValueError:
                pass
