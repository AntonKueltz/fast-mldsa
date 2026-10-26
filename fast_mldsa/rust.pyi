from typing import ClassVar, TypeAlias

Buf: TypeAlias = bytes | bytearray | memoryview

class ParameterSet:
    ML_DSA_44: ClassVar[ParameterSet]
    ML_DSA_65: ClassVar[ParameterSet]
    ML_DSA_87: ClassVar[ParameterSet]

class VerificationKey:
    def __bytes__(self) -> bytes: ...
    def to_bytes(self) -> bytes: ...
    @staticmethod
    def from_bytes(data: bytes) -> VerificationKey: ...
    def verify(self, sig: bytes, msg: bytes, ctx: bytes | None = None) -> bool: ...

class SigningKey:
    def __bytes__(self) -> bytes: ...
    def to_bytes(self) -> bytearray: ...
    @staticmethod
    def from_bytes(data: Buf) -> SigningKey: ...
    def sign(
        self, msg: bytes, ctx: bytes | None = None, deterministic: bool = False
    ) -> bytes: ...

class ML_DSA:
    @property
    def parameter_set(self) -> ParameterSet: ...
    def key_gen(
        self, seed: Buf | None = None
    ) -> tuple[VerificationKey, SigningKey]: ...

MlDsa: ML_DSA
MlDsa44: ML_DSA
MlDsa65: ML_DSA
MlDsa87: ML_DSA
