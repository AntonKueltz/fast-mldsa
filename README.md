[![PyPI](https://img.shields.io/pypi/v/fast-mldsa.svg)](https://pypi.org/project/fast-mldsa/)
[![ReadTheDocs](https://readthedocs.org/projects/fast-mldsa/badge/?version=latest)](https://fast-mldsa.readthedocs.io/en/latest/?badge=latest)

# Module-Lattice-Based Digital Signature Algorithm (ML-DSA)
An implementation of the module-lattice-based digital signature algorithm (ML-DSA)
as described in [FIPS-204](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf).
This implementation's primary objectives are security and speed.

# Usage

```python
from fast_mldsa import MlDsa

vk, sk = MlDsa.key_gen()  # verification and signing key

msg, ctx = b"...", b"..."
sig = sk.sign(msg, ctx)  # signature over the message and context
valid = vk.verify(sig, msg, ctx)  # check if the signature is valid

# you can also sign with no context, which is equivalent to a context of b""
sig = sk.sign(msg)
```

## Keys from Seeds

In some cases, it may be desirable to transmit a keypair as the seed it was
generated from (for instance, where network bandwidth is a concern). In those
cases the keypair can be regenerated from the seed. Note that the seed must be
exactly 32 bytes long.

> [!WARNING]
> Use a cryptographically secure pseudorandom number generator (CSPRNG) if you generate a seed. If the seed can be guessed or recovered, then your signing key is compromised.

```python
from fast_mldsa import MlDsa

seed = bytearray(32)  # this would be a terrible seed (all zeroes) - easy to guess
vk, sk = MlDsa.key_gen(seed)  # verification and signing key from seed

wrong_len = bytearray(31)
vk, sk = MlDsa.key_gen(wrong_len)  # => ValueError
```

## Deterministic Signatures

By default, this library uses the "hedged" version of the algorithm which generates
32 random bytes as part of signature generation. You can also use the deterministic
variant of signing, which sets those bytes to 0, as follows.

```python
from fast_mldsa import MlDsa

vk, sk = MlDsa.key_gen()  # verification and signing key
msg, ctx = b"...", b"..."

sig1 = sk.sign(
    msg, ctx, deterministic=True
)  # detrministic signature over the message and context
sig2 = sk.sign(msg, ctx, deterministic=True)
assert sig1 == sig2

sig1 = sk.sign(msg, ctxt)  # "hedged" signature over the message and context
sig2 = sk.sign(msg, ctxt)
assert sig1 != sig2
```

## Parameter Sets

The default parameter set used is `ML-DSA-65` which is considered "Category 3" by NIST,
roughly equivalent to the security provided by AES-192. The parameter sets `ML-DSA-44`
and `ML-DSA-87` are also supported.

```python
from fast_mldsa import MlDsa87

vk, sk = MlDsa87.key_gen()  # verification and signing key for ML-DSA-87 parameter set
sig = sk.sign(msg)  # signature over the message and context
valid = vk.verify(sig, msg, ctx)  # check if the signature is valid
```

# Security

> [!WARNING]
> This package has not been audited by a third party. For security critical applications I recommend using a more established crypto library.

## Timing Attacks

This package aims for a constant time implementation of any code that operates on secret
material. There is no branching done on any secret material. This does not guarantee
that the compiler generates assembly with no branches or timing leakage in it. You can
use e.g. `cargo asm` to inspect the assembly for the target architecture. You can also
use some of the constant time tests in [`rs-core/examples`](./rs-core/examples/) to check
if code executes in constant time on your machine. These tests use the
[rust port](https://github.com/rozbb/dudect-bencher) of
[dudect](https://github.com/oreparaz/dudect). A constant time implementation will have a
max t value that stays more or less constant over time.

```bash
cargo ct-poly  # run the constant time tests for all relevant functions of polynomials
cargo ct-poly --continuous inv_ntt  # run the inv-ntt polynomial function test
```

There is also an opportunity to use additional tools like [ctgrind](https://github.com/agl/ctgrind)
and [timecop](https://www.post-apocalyptic-crypto.org/timecop/) to further increase confidence.

## Memory Management

In accordance with the FIPS standard a best effort is made to zero out all intermediate
values in key generation and signing that are considered secret. The rust core code is no-std
compatible and does not do any dynamic memory allocation.

Note that once sensitive material is in the hands of the consuming application it is up to that
application to properly manage that material. In particular, signing keys _will_ have their memory
zeroized by the underlying rust implementation, but only once the reference count on the python side
is 0 and `dealloc` is invoked during garbage collection.

Care should also be taken with sensitive material that is passed _to_ this package.
`MlDsa.key_gen(seed)` and `SigningKey.from_bytes(data)` allow a `bytearray` or `memoryview`
to be used for `seed`  and `data` for this reason - they are sensitive material that the
application may want to zero out after passing them as args. Immutable `bytes` would not support
this.

# Development

As a prerequisite, the [rust toolchain](https://rust-lang.org/tools/install/) and
[`uv`](https://docs.astral.sh/uv/#installation) are required  for this project.

Build the rust code and bindings.

```bash
uv run maturin develop -r
```

Run the test suite. The test suite covers the following test vectors:
- [NIST ACVP](https://github.com/usnistgov/ACVP-Server)
- [Wycheproof](https://github.com/C2SP/wycheproof)
- [CCTV Accumulated Tests](https://github.com/C2SP/CCTV/tree/main/ML-DSA/accumulated)

```bash
cargo test     # tests the rust code (generally internal interfaces)
uv run pytest  # tests the python code (generally external interfaces)
```

There are also slower tests for the higher iteration
[accumulated tests](https://github.com/C2SP/CCTV/tree/main/ML-DSA/accumulated) that
are skipped by default. The 60,000,000 iteration tests take a very long time to run.
On an i9-9900k the below command takes about 14h30m.

```bash
uv run pytest -m slow
```

# Performance

```bash
uv run benchmark
```

Below are some benchmarks for each parameter set, running on 64bit OS with a i9-9900k and
python3.14.

```
ops=['keygen', 'sign', 'verify'] duration=1.0s repeats=5 warmup=0.5s

op             param set      ops/sec (med)     µs/op           min           max       stdev
---------------------------------------------------------------------------------------------
keygen         ML_DSA_44             11,544      86.6        10,296        11,712         613
sign           ML_DSA_44              5,205     192.1         4,610         5,373         305
verify         ML_DSA_44             29,776      33.6        29,316        30,157         328
keygen         ML_DSA_65              6,818     146.7         6,711         6,864          59
sign           ML_DSA_65              3,396     294.4         3,119         3,526         178
verify         ML_DSA_65             21,789      45.9        21,594        21,839         108
keygen         ML_DSA_87              4,571     218.8         4,534         4,601          28
sign           ML_DSA_87              3,010     332.3         2,813         3,251         182
verify         ML_DSA_87             15,876      63.0        15,370        15,955         255
```

Depending on your specific CPU architecture, you may be able to see significant performance
improvements compiling the rust source specifically for your instruction set. For example,
compiling with `RUSTFLAGS='-C target-cpu=native'` yields the following performance gains
on the same i9-9900k.

```
ops=['keygen', 'sign', 'verify'] duration=1.0s repeats=5 warmup=0.5s

op             param set      ops/sec (med)     µs/op           min           max       stdev
---------------------------------------------------------------------------------------------
keygen         ML_DSA_44             13,898      72.0        13,582        14,008         175
sign           ML_DSA_44              9,475     105.5         9,099         9,532         209
verify         ML_DSA_44             48,847      20.5        47,495        48,895         606
keygen         ML_DSA_65              8,184     122.2         8,150         8,245          39
sign           ML_DSA_65              6,255     159.9         6,076         6,548         176
verify         ML_DSA_65             38,977      25.7        38,755        39,178         156
keygen         ML_DSA_87              5,636     177.4         5,068         5,704         272
sign           ML_DSA_87              5,861     170.6         5,705         5,980         101
verify         ML_DSA_87             29,253      34.2        25,930        29,601       1,547
```

## Rust Benchmarking

You can also run benchmarks for the rust code, as well as generate flamegraphs. This
can be handy when performance tuning, both for catching regressions and for identifying
candidate areas for further optimization.

```bash
# run the benchmark for all operations over all parameter sets
cargo bench --bench dsa
# generate a flamegraph for the sign operation using the ML-DSA-44 parameter set
RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph --bench dsa -- --bench --profile-time 5 "sign/44"
```
