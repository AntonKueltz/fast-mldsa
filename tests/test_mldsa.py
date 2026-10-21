from random import randbytes, randint
from unittest import TestCase

from fastmldsa import ML_DSA, ParameterSet

ROUNDS = 1000


class Test_ML_DSA(TestCase):
    def test_mldsa44(self):
        dsa = ML_DSA(ParameterSet.ML_DSA_44)
        vk, sk = dsa.key_gen()

        for _ in range(ROUNDS):
            msg, ctx = randbytes(randint(8, 1024)), randbytes(randint(0, 128))
            sig = dsa.sign(sk, msg, ctx)
            assert dsa.verify(vk, msg, ctx, sig)

    def test_mldsa65(self):
        dsa = ML_DSA(ParameterSet.ML_DSA_65)
        vk, sk = dsa.key_gen()

        for _ in range(ROUNDS):
            msg, ctx = randbytes(randint(8, 1024)), randbytes(randint(0, 128))
            sig = dsa.sign(sk, msg, ctx)
            assert dsa.verify(vk, msg, ctx, sig)

    def test_mldsa87(self):
        dsa = ML_DSA(ParameterSet.ML_DSA_87)
        vk, sk = dsa.key_gen()

        for _ in range(ROUNDS):
            msg, ctx = randbytes(randint(8, 1024)), randbytes(randint(0, 128))
            sig = dsa.sign(sk, msg, ctx)
            assert dsa.verify(vk, msg, ctx, sig)
