"""Benchmark ML-DSA key generation, signing and verification: ops/sec for each
parameter set.

Usage:

python benchmark.py

python benchmark.py --duration 5 --repeats 7 --json results.json

"""

from __future__ import annotations

import argparse
import gc
import json
import platform
import statistics
import time
from collections.abc import Callable
from dataclasses import asdict, dataclass
from typing import Any

from fastmldsa import MlDsa44, MlDsa65, MlDsa87, SigningKey, VerificationKey


@dataclass(frozen=True)
class Case:
    """One parameter set to benchmark.

    The benchmark lists the sets explicitly and carries its own labels rather
    than relying on enum iteration. `sizes` are FIPS 204 sizes in bytes:
    (vk, sk, sig).
    """

    name: str
    dsa: Any
    sizes: tuple[int, int, int]


CASES = [
    Case("ML_DSA_44", MlDsa44, (1312, 2560, 2420)),
    Case("ML_DSA_65", MlDsa65, (1952, 4032, 3309)),
    Case("ML_DSA_87", MlDsa87, (2592, 4896, 4627)),
]


# Fixed signing inputs so every round measures the same work.
MSG = bytes(range(256)) * 4  # 1 KiB message
CTX = b"benchmark"


@dataclass
class Result:
    operation: str
    parameter_set: str
    ops_per_sec_median: float
    ops_per_sec_mean: float
    ops_per_sec_min: float
    ops_per_sec_max: float
    ops_per_sec_stdev: float
    us_per_op_median: float
    iterations_per_round: int
    rounds: int


def calibrate(fn: Callable[[], object], target_seconds: float) -> int:
    """Find an iteration count whose runtime is roughly target_seconds."""
    n = 1
    while True:
        start = time.perf_counter()
        for _ in range(n):
            fn()
        elapsed = time.perf_counter() - start
        if elapsed >= 0.05:  # long enough to be measurable
            return max(1, int(n * target_seconds / elapsed))
        n *= 2


def bench(
    op: str,
    name: str,
    fn: Callable[[], object],
    duration: float,
    repeats: int,
    warmup: float,
) -> Result:
    # Warm-up: caches, branch predictors, CPU frequency ramp.
    end = time.perf_counter() + warmup
    while time.perf_counter() < end:
        fn()

    iters = calibrate(fn, duration / repeats)

    rates: list[float] = []
    gc.collect()
    gc.disable()  # keep GC pauses out of the measurement
    try:
        for _ in range(repeats):
            start = time.perf_counter()
            for _ in range(iters):
                fn()
            elapsed = time.perf_counter() - start
            rates.append(iters / elapsed)
    finally:
        gc.enable()

    median = statistics.median(rates)
    return Result(
        operation=op,
        parameter_set=name,
        ops_per_sec_median=median,
        ops_per_sec_mean=statistics.fmean(rates),
        ops_per_sec_min=min(rates),
        ops_per_sec_max=max(rates),
        ops_per_sec_stdev=statistics.stdev(rates) if len(rates) > 1 else 0.0,
        us_per_op_median=1e6 / median,
        iterations_per_round=iters,
        rounds=repeats,
    )


def _check(cond: bool, msg: str) -> None:
    if not cond:
        raise AssertionError(msg)


def _keys(case: Case) -> tuple[Any, VerificationKey, SigningKey]:
    """Fresh keypair with size and type sanity checks."""
    vk_len, sk_len, _ = case.sizes

    dsa = case.dsa
    vk, sk = dsa.key_gen()

    _check(isinstance(vk, VerificationKey), f"{case.name}: vk has wrong type")
    _check(isinstance(sk, SigningKey), f"{case.name}: sk has wrong type")
    _check(
        len(vk.to_bytes()) == vk_len,
        f"{case.name}: vk is {len(vk.to_bytes())} bytes, expected {vk_len}",
    )
    _check(
        len(sk.to_bytes()) == sk_len,
        f"{case.name}: sk is {len(sk.to_bytes())} bytes, expected {sk_len}",
    )
    _check(bytes(vk) == vk.to_bytes(), f"{case.name}: bytes(vk) != vk.to_bytes()")
    _check(bytes(sk) == sk.to_bytes(), f"{case.name}: bytes(sk) != sk.to_bytes()")

    return dsa, vk, sk


def make_keygen(case: Case) -> Callable[[], object]:
    dsa, vk, _ = _keys(case)  # sanity check only; timed region is key_gen itself

    # Keygen is randomized: two calls must yield different keys.
    vk2, _ = dsa.key_gen()
    _check(vk2.to_bytes() != vk.to_bytes(), f"{case.name}: key_gen is not randomized")

    return dsa.key_gen


def make_sign(case: Case) -> Callable[[], object]:
    _, _, sk = _keys(case)  # key generation is outside the timed region
    sig_len = case.sizes[2]

    sig = sk.sign(MSG, CTX)
    _check(isinstance(sig, bytes), f"{case.name}: signature has wrong type")
    _check(
        len(sig) == sig_len,
        f"{case.name}: signature is {len(sig)} bytes, expected {sig_len}",
    )

    # Signing uses rejection sampling, so per-call time varies; the harness
    # averages over many iterations per round to smooth that out.
    return lambda: sk.sign(MSG, CTX)


def make_verify(case: Case) -> Callable[[], object]:
    _, vk, sk = _keys(case)  # key generation is outside the timed region
    sig = sk.sign(MSG, CTX)  # so is signing

    # Correctness: the signature verifies, and tampered inputs are rejected.
    _check(vk.verify(sig, MSG, CTX) is True, f"{case.name}: valid sig rejected")
    bad_msg = bytes([MSG[0] ^ 1]) + MSG[1:]
    _check(
        vk.verify(sig, bad_msg, CTX) is False,
        f"{case.name}: sig accepted for a modified message",
    )
    _check(
        vk.verify(sig, MSG, CTX + b"!") is False,
        f"{case.name}: sig accepted for a different context",
    )
    bad_sig = bytes([sig[0] ^ 1]) + sig[1:]
    _check(
        vk.verify(bad_sig, MSG, CTX) is False,
        f"{case.name}: modified sig accepted",
    )

    # Timed region is the success path only.
    return lambda: vk.verify(sig, MSG, CTX)


OPS: dict[str, Callable[[Case], Callable[[], object]]] = {
    "keygen": make_keygen,
    "sign": make_sign,
    "verify": make_verify,
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--duration",
        type=float,
        default=1.0,
        help="total measured seconds per op/parameter set (default: 1)",
    )
    parser.add_argument(
        "--repeats",
        type=int,
        default=5,
        help="rounds per op/parameter set; stats are over rounds (default: 5)",
    )
    parser.add_argument(
        "--warmup",
        type=float,
        default=0.5,
        help="warm-up seconds per op/parameter set (default: 0.5)",
    )
    parser.add_argument("--json", metavar="PATH", help="also write results as JSON")
    args = parser.parse_args()

    print(
        f"Python {platform.python_version()} on {platform.platform()} ({platform.machine()})"
    )
    print(
        f"ops={list(OPS)} duration={args.duration}s repeats={args.repeats} "
        f"warmup={args.warmup}s\n"
    )

    results: list[Result] = []
    for case in CASES:
        for op, make in OPS.items():
            fn = make(case)  # includes sanity check
            results.append(
                bench(op, case.name, fn, args.duration, args.repeats, args.warmup)
            )

    header = (
        f"{'op':<15}{'param set':<12}{'ops/sec (med)':>16}{'µs/op':>10}"
        f"{'min':>14}{'max':>14}{'stdev':>12}"
    )
    print(header)
    print("-" * len(header))
    for r in results:
        print(
            f"{r.operation:<15}{r.parameter_set:<12}{r.ops_per_sec_median:>16,.0f}"
            f"{r.us_per_op_median:>10.1f}{r.ops_per_sec_min:>14,.0f}"
            f"{r.ops_per_sec_max:>14,.0f}{r.ops_per_sec_stdev:>12,.0f}"
        )

    if args.json:
        payload = {
            "python": platform.python_version(),
            "platform": platform.platform(),
            "machine": platform.machine(),
            "results": [asdict(r) for r in results],
        }
        with open(args.json, "w") as f:
            json.dump(payload, f, indent=2)
        print(f"\nWrote {args.json}")


if __name__ == "__main__":
    main()
