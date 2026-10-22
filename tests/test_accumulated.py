from unittest import TestCase

from Crypto.Hash import SHAKE128
from pytest import mark

from fastmldsa import ML_DSA, ParameterSet


class TestAccumulated(TestCase):
    def _runner(self, params: ParameterSet, iterations: int) -> bytes:
        s = SHAKE128.new()
        a = SHAKE128.new()
        dsa = ML_DSA(params)

        for _ in range(iterations):
            seed = s.read(32)
            pk, sk = dsa.key_gen(seed)
            a.update(bytes(pk))
            sig = dsa.sign(sk, b"", b"", deterministic=True)
            a.update(sig)
            assert dsa.verify(pk, sig, b"", b"")

        return a.read(32)

    def test_rapid_mldsa44(self):
        expected = bytes.fromhex(
            "d51148e1f9f4fa1a723a6cf42e25f2a99eb5c1b378b3d2dbbd561b1203beeae4"
        )
        actual = self._runner(ParameterSet.ML_DSA_44, 100)
        self.assertEqual(actual, expected)

    @mark.slow
    def test_ci_mldsa44(self):
        expected = bytes.fromhex(
            "e7fd21f6a59bcba60d65adc44404bb29a7c00e5d8d3ec06a732c00a306a7d143"
        )
        actual = self._runner(ParameterSet.ML_DSA_44, 10_000)
        self.assertEqual(actual, expected)

    @mark.slow
    def test_dev_mldsa44(self):
        expected = bytes.fromhex(
            "080b48049257f5cd30dee17d6aa393d6c42fe52a29099df84a460ebaf4b02330"
        )
        actual = self._runner(ParameterSet.ML_DSA_44, 60_000_000)
        self.assertEqual(actual, expected)

    def test_rapid_mldsa65(self):
        expected = bytes.fromhex(
            "8358a1843220194417cadbc2651295cd8fc65125b5a5c1a239a16dc8b57ca199"
        )
        actual = self._runner(ParameterSet.ML_DSA_65, 100)
        self.assertEqual(actual, expected)

    @mark.slow
    def test_ci_mldsa65(self):
        expected = bytes.fromhex(
            "5ff5e196f0b830c3b10a9eb5358e7c98a3a20136cb677f3ae3b90175c3ace329"
        )
        actual = self._runner(ParameterSet.ML_DSA_65, 10_000)
        self.assertEqual(actual, expected)

    @mark.slow
    def test_dev_mldsa65(self):
        expected = bytes.fromhex(
            "0af0165db2b180f7a83dbecad1ccb758b9c2d834b7f801fc49dd572a9d4b1e83"
        )
        actual = self._runner(ParameterSet.ML_DSA_65, 60_000_000)
        self.assertEqual(actual, expected)

    def test_rapid_mldsa87(self):
        expected = bytes.fromhex(
            "8c3ad714777622b8f21ce31bb35f71394f23bc0fcf3c78ace5d608990f3b061b"
        )
        actual = self._runner(ParameterSet.ML_DSA_87, 100)
        self.assertEqual(actual, expected)

    @mark.slow
    def test_ci_mldsa87(self):
        expected = bytes.fromhex(
            "80a8cf39317f7d0be0e24972c51ac152bd2a3e09bc0c32ce29dd82c4e7385e60"
        )
        actual = self._runner(ParameterSet.ML_DSA_87, 10_000)
        self.assertEqual(actual, expected)

    @mark.slow
    def test_dev_mldsa87(self):
        expected = bytes.fromhex(
            "011166e9d5032c9bdc5c9bbb5dbb6c86df1c3d9bf3570b65ebae942dd9830057"
        )
        actual = self._runner(ParameterSet.ML_DSA_87, 60_000_000)
        self.assertEqual(actual, expected)
