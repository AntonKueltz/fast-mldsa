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
valid = dsa.verify(vk, msg, ctx, sig)  # check if the signature is valid
```

## Parameter Sets

The default parameter set used is `ML-DSA-65` which is considered "Category 3" by NIST,
roughly equivalent to the security provided by AES-192. The parameter sets `ML-DSA-44`
and `ML-DSA-87` are also supported.

```python
from fastmldsa import ML_DSA, ParameterSet

dsa = ML_DSA(ParameterSet.ML_DSA_87)
vk, sk = dsa.key_gen()  # verification and signing key for ML-DSA-87 parameter set
valid = dsa.verify(vk, msg, ctx, sig)  # check if the signature is valid
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
uv run pytest  # tests the python code
```

# Performance

Below are some benchmarks for each parameter set, running on 64bit OS with a i9-9900k and
python3.14.

```
ops=['keygen', 'sign', 'verify'] duration=1.0s repeats=5 warmup=0.5s

op             param set      ops/sec (med)     µs/op           min           max       stdev
---------------------------------------------------------------------------------------------
keygen         ML_DSA_44             12,913      77.4        12,846        13,111         125
sign           ML_DSA_44              5,119     195.3         4,823         5,217         177
verify         ML_DSA_44             29,168      34.3        28,942        29,332         156
keygen         ML_DSA_65              7,134     140.2         7,015         7,246          82
sign           ML_DSA_65              3,234     309.2         3,105         3,308          76
verify         ML_DSA_65             21,574      46.4        21,117        21,640         220
keygen         ML_DSA_87              4,778     209.3         4,757         4,820          25
sign           ML_DSA_87              2,929     341.4         2,821         2,950          66
verify         ML_DSA_87             15,940      62.7        15,674        15,977         132
```

Depending on your specific CPU architecture, you may be able to see significant performance
improvements compiling the rust source specifically for your instruction set. For example,
compiling with `RUSTFLAGS='-C target-cpu=native'` yields the following performance gains
on the same i9-9900k.

```
ops=['keygen', 'sign', 'verify'] duration=1.0s repeats=5 warmup=0.5s

op             param set      ops/sec (med)     µs/op           min           max       stdev
---------------------------------------------------------------------------------------------
keygen         ML_DSA_44             16,177      61.8        16,065        16,377         135
sign           ML_DSA_44              9,834     101.7         9,576         9,917         141
verify         ML_DSA_44             52,846      18.9        52,105        52,899         333
keygen         ML_DSA_65              8,838     113.1         8,764         8,905          54
sign           ML_DSA_65              6,727     148.7         6,473         6,899         162
verify         ML_DSA_65             42,286      23.6        42,168        42,561         166
keygen         ML_DSA_87              5,928     168.7         5,808         5,977          76
sign           ML_DSA_87              5,963     167.7         5,674         6,178         206
verify         ML_DSA_87             31,839      31.4        31,242        32,059         343
```
