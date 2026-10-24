# Module-Lattice-Based Digital Signature Algorithm (ML-DSA)
An implementation of the module-lattice-based digital signature algorithm (ML-DSA)
as described in [FIPS-204](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf).
This implementation's primary objectives are security and speed.

# Usage

```python
from fastmldsa import ML_DSA

dsa = ML_DSA()
vk, sk = dsa.key_gen()  # verification and signing key

msg, ctx = b"...", b"..."
sig = dsa.sign(sk, msg, ctx)  # signature over the message and context
valid = dsa.verify(vk, sig, msg, ctx)  # check if the signature is valid

# you can also sign with no context, which is equivalent to a context of b""
sig = dsa.sign(sk, msg)
```

## Keys from Seeds

In some cases, it may be desirable to transmit a keypair as the seed it was
generated from (for instance, where network bandwidth is a concern). In those
cases the keypair can be regenerated from the seed. Note that the seed must be
exactly 32 bytes long.

> [!WARNING]
> Use a cryptographically secure pseudorandom number generator (CSPRNG) if you generate a seed. If the seed can be guessed or recovered, then your signing key is compromised.

```python
from fastmldsa import ML_DSA

dsa = ML_DSA()
seed = b"\x00" * 32  # this would be a terrible seed - easy to guess
vk, sk = dsa.key_gen(seed)  # verification and signing key from seed

wrong_len = b"\x00" * 31
vk, sk = dsa.key_gen(wrong_len)  # => ValueError
```

## Deterministic Signatures

By default, this library uses the "hedged" version of the algorithm which generates
32 random bytes as part of signature generation. You can also use the deterministic
variant of signing, which sets those bytes to 0, as follows.

```python
from fastmldsa import ML_DSA

dsa = ML_DSA()
vk, sk = dsa.key_gen()  # verification and signing key
msg, ctx = b"...", b"..."

sig1 = dsa.sign(
    sk, msg, ctx, deterministic=True
)  # detrministic signature over the message and context
sig2 = dsa.sign(sk, msg, ctx, deterministic=True)
assert sig1 == sig2

sig1 = dsa.sign(sk, msg, ctxt)  # "hedged" signature over the message and context
sig2 = dsa.sign(sk, msg, ctxt)
assert sig1 != sig2
```

## Parameter Sets

The default parameter set used is `ML-DSA-65` which is considered "Category 3" by NIST,
roughly equivalent to the security provided by AES-192. The parameter sets `ML-DSA-44`
and `ML-DSA-87` are also supported.

```python
from fastmldsa import ML_DSA, ParameterSet

dsa = ML_DSA(ParameterSet.ML_DSA_87)
vk, sk = dsa.key_gen()  # verification and signing key for ML-DSA-87 parameter set
valid = dsa.verify(vk, sig, msg, ctx)  # check if the signature is valid
```

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
keygen         ML_DSA_44             11,535      86.7        11,365        11,727         146
sign           ML_DSA_44              5,340     187.3         5,122         5,479         141
verify         ML_DSA_44             29,753      33.6        29,508        30,015         205
keygen         ML_DSA_65              6,842     146.2         6,822         6,893          30
sign           ML_DSA_65              3,390     295.0         3,351         3,594         117
verify         ML_DSA_65             21,920      45.6        21,794        22,115         121
keygen         ML_DSA_87              4,619     216.5         4,553         4,645          34
sign           ML_DSA_87              3,075     325.2         2,965         3,175          83
verify         ML_DSA_87             16,082      62.2        15,889        16,101          87
```

Depending on your specific CPU architecture, you may be able to see significant performance
improvements compiling the rust source specifically for your instruction set. For example,
compiling with `RUSTFLAGS='-C target-cpu=native'` yields the following performance gains
on the same i9-9900k.

```
ops=['keygen', 'sign', 'verify'] duration=1.0s repeats=5 warmup=0.5s

op             param set      ops/sec (med)     µs/op           min           max       stdev
---------------------------------------------------------------------------------------------
keygen         ML_DSA_44             16,673      60.0        16,362        16,876         191
sign           ML_DSA_44              9,975     100.2         9,821        10,227         178
verify         ML_DSA_44             49,534      20.2        46,771        49,668       1,236
keygen         ML_DSA_65              9,134     109.5         9,115         9,205          44
sign           ML_DSA_65              7,014     142.6         6,896         7,176         116
verify         ML_DSA_65             38,854      25.7        38,666        39,045         147
keygen         ML_DSA_87              6,114     163.5         6,087         6,150          27
sign           ML_DSA_87              6,325     158.1         6,291         6,560         131
verify         ML_DSA_87             29,069      34.4        28,944        29,530         241
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
