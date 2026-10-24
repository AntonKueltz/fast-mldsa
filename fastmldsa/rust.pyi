from enum import Enum
from typing import TypeAlias

Buf: TypeAlias = bytes | bytearray | memoryview

class ParameterSet(Enum):
    ML_DSA_44 = 44
    ML_DSA_65 = 65
    ML_DSA_87 = 87

class VerificationKey:
    def __bytes__(self) -> bytes: ...
    def to_bytes(self) -> bytes: ...
    @staticmethod
    def from_bytes(data: bytes) -> VerificationKey: ...

class SigningKey:
    def __bytes__(self) -> bytes: ...
    def to_bytes(self) -> bytes: ...
    @staticmethod
    def from_bytes(data: Buf) -> SigningKey: ...

class ML_DSA:
    def __init__(self, parameter_set: ParameterSet = ParameterSet.ML_DSA_65): ...
    def key_gen(
        self, seed: Buf | None = None
    ) -> tuple[VerificationKey, SigningKey]: ...
    def sign(
        self,
        sk: SigningKey,
        msg: bytes,
        ctx: bytes | None = None,
        deterministic: bool = False,
    ) -> bytes: ...
    def verify(
        self, vk: VerificationKey, sig: bytes, msg: bytes, ctx: bytes | None = None
    ) -> bool: ...
