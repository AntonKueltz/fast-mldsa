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
```

## Parameter Sets

The default parameter set used is `ML-DSA-65` which is considered "Category 3" by NIST,
roughly equivalent to the security provided by AES-192. The parameter sets `ML-DSA-44`
and `ML-DSA-87` are also supported.

```python
from fastmldsa import ML_DSA, ParameterSet

dsa = ML_DSA(ParameterSet.ML_DSA_87)
vk, sk = dsa.key_gen()  # verification and signing key for ML-DSA-87 parameter set
```

# Development

As a prerequisite, the [rust toolchain](https://rust-lang.org/tools/install/) and
[`uv`](https://docs.astral.sh/uv/#installation) are required  for this project.

Build the rust code and bindings.

```bash
uv run maturin develop -r
```

Run the test suite.

```bash
cargo test     # tests the rust code
```

# Performance

Below are some benchmarks for each parameter set, running on 64bit OS with a i9-9900k and
python3.14.

```
ops=['keygen'] duration=1.0s repeats=5 warmup=0.5s

op             param set      ops/sec (med)     µs/op           min           max       stdev
---------------------------------------------------------------------------------------------
keygen         ML_DSA_44             14,143      70.7        13,331        14,296         381
keygen         ML_DSA_65              7,610     131.4         6,699         7,647         412
keygen         ML_DSA_87              5,035     198.6         4,999         5,096          36
```

Depending on your specific CPU architecture, you may be able to see significant performance
improvements compiling the rust source specifically for your instruction set. For example,
compiling with `RUSTFLAGS='-C target-cpu=native'` yields the following performance gains
on the same i9-9900k.

```
ops=['keygen'] duration=1.0s repeats=5 warmup=0.5s

op             param set      ops/sec (med)     µs/op           min           max       stdev
---------------------------------------------------------------------------------------------
keygen         ML_DSA_44             18,255      54.8        17,444        18,399         387
keygen         ML_DSA_65              9,619     104.0         9,502         9,708          74
keygen         ML_DSA_87              6,464     154.7         6,403         6,518          45
```
