#!/usr/bin/env python3
"""Known-answer generator for the liminal_core tests (docs/specs/liminal_core.md, section 10).

Not part of the build; no test runs it. It is the independent implementation that the literals
in crates/liminal_core/tests/hashing.rs, rng.rs and time.rs name: XXH64, SplitMix64,
xoshiro256**, Lemire's bounded integers and the float conversions, written from the published
descriptions with Python's unbounded integers (every wrap-around is an explicit `& MASK`), not
translated from the Rust code. Needs Python 3.8+, standard library only.

Before printing anything it checks itself against published reference vectors:
  - XXH64, seed 0: XSUM_XXH64_testdata of tests/sanity_test_vectors.h, xxHash v0.8.3
    (github.com/Cyan4973/xxHash), over the test buffer of tests/sanity_test.c.
  - xoshiro256** and SplitMix64: the `reference` tests of rand_xoshiro
    (github.com/rust-random/rngs, commit 84683fca), whose values "were produced with the
    reference implementation" (xoshiro256starstar.c, splitmix64.c of prng.di.unimi.it).

Usage (run from anywhere):
  kat.py xxh64 <text>...                 XXH64 (seed 0) of the UTF-8 bytes of each text
  kat.py xxh64hex <name>=<hex>...        XXH64 (seed 0) of the bytes given as hex ("name=" is empty)
  kat.py stream <world_seed> <key> <n>   CORE-R091 seeding; prints x, the 4 state words, n outputs
  kat.py methods <world_seed> <key> <rounds>
                                         per round: next_u64, next_u32, below_u32 for each of
                                         BOUNDS32, below_u64 for each of BOUNDS64, next_f32 bits,
                                         next_f64 bits
  kat.py max-output-state                a state whose next_u64 is 2^64 - 1
  kat.py time                            literals of tests/time.rs
  kat.py rng-streams                     the hash of workload `rng_streams` of tests/determinism.rs
Integers are accepted in decimal or 0x form.
"""

import struct
import sys

MASK = (1 << 64) - 1

P1 = 0x9E3779B185EBCA87
P2 = 0xC2B2AE3D27D4EB4F
P3 = 0x165667B19E3779F9
P4 = 0x85EBCA77C2B2AE63
P5 = 0x27D4EB2F165667C5

BOUNDS32 = [1, 2, 6, 1000, 0x80000000, 0x80000001, 0xFFFFFFFF]
BOUNDS64 = [1, 6, 1000, 0x8000000000000000, 0x8000000000000001, 0xFFFFFFFFFFFFFFFF]


def rotl(value, count):
    return ((value << count) | (value >> (64 - count))) & MASK


def xxh_round(acc, lane):
    return (rotl((acc + lane * P2) & MASK, 31) * P1) & MASK


def xxh64(data, seed=0):
    """XXH64 as described in doc/xxhash_spec.md of the xxHash repository."""
    length = len(data)
    offset = 0
    if length >= 32:
        acc = [(seed + P1 + P2) & MASK, (seed + P2) & MASK, seed, (seed - P1) & MASK]
        while offset + 32 <= length:
            for lane in range(4):
                word = int.from_bytes(data[offset + 8 * lane:offset + 8 * lane + 8], "little")
                acc[lane] = xxh_round(acc[lane], word)
            offset += 32
        result = (rotl(acc[0], 1) + rotl(acc[1], 7) + rotl(acc[2], 12) + rotl(acc[3], 18)) & MASK
        for value in acc:
            result = ((result ^ xxh_round(0, value)) * P1 + P4) & MASK
    else:
        result = (seed + P5) & MASK
    result = (result + length) & MASK
    while offset + 8 <= length:
        word = int.from_bytes(data[offset:offset + 8], "little")
        result = (rotl(result ^ xxh_round(0, word), 27) * P1 + P4) & MASK
        offset += 8
    if offset + 4 <= length:
        word = int.from_bytes(data[offset:offset + 4], "little")
        result = (rotl(result ^ ((word * P1) & MASK), 23) * P2 + P3) & MASK
        offset += 4
    while offset < length:
        result = (rotl(result ^ ((data[offset] * P5) & MASK), 11) * P1) & MASK
        offset += 1
    result ^= result >> 33
    result = (result * P2) & MASK
    result ^= result >> 29
    result = (result * P3) & MASK
    result ^= result >> 32
    return result


class SplitMix64:
    """splitmix64.c."""

    def __init__(self, x):
        self.x = x & MASK

    def next(self):
        self.x = (self.x + 0x9E3779B97F4A7C15) & MASK
        z = self.x
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
        return z ^ (z >> 31)


class Xoshiro256StarStar:
    """xoshiro256starstar.c, function next()."""

    def __init__(self, state):
        self.s = list(state)

    def next(self):
        s = self.s
        result = (rotl((s[1] * 5) & MASK, 7) * 9) & MASK
        t = (s[1] << 17) & MASK
        s[2] ^= s[0]
        s[3] ^= s[1]
        s[1] ^= s[2]
        s[0] ^= s[3]
        s[2] ^= t
        s[3] = rotl(s[3], 45)
        return result

    def next32(self):
        return self.next() >> 32

    def below(self, bound, bits):
        """Lemire 2019, "Fast Random Integer Generation in an Interval", algorithm 5."""
        draw = self.next32 if bits == 32 else self.next
        modulus = 1 << bits
        m = draw() * bound
        low = m % modulus
        if low < bound:
            threshold = (modulus - bound) % bound
            while low < threshold:
                m = draw() * bound
                low = m % modulus
        return m // modulus

    def f32_bits(self):
        value = (self.next() >> 40) / float(1 << 24)  # exact: 24-bit integer times 2^-24
        return struct.unpack("<I", struct.pack("<f", value))[0]

    def f64_bits(self):
        value = (self.next() >> 11) / float(1 << 53)  # exact: 53-bit integer times 2^-53
        return struct.unpack("<Q", struct.pack("<d", value))[0]


def seed_stream(world_seed, key):
    x = xxh64(struct.pack("<QQ", world_seed, key))
    mix = SplitMix64(x)
    return x, [mix.next() for _ in range(4)]


def sanity_buffer(length):
    """fillTestBuffer of tests/sanity_test.c."""
    out = bytearray()
    generator = 2654435761
    for _ in range(length):
        out.append(generator >> 56)
        generator = (generator * 11400714785074694797) & MASK
    return bytes(out)


def self_check():
    # (length, XXH64 seed 0) from XSUM_XXH64_testdata, xxHash v0.8.3.
    vectors = [
        (0, 0xEF46DB3751D8E999), (1, 0xE934A84ADB052768), (3, 0xFF7E1959CB50794A),
        (4, 0x9136A0DCA57457EE), (7, 0x6C83909A9F01ED25), (8, 0xCDBCF538E71D1348),
        (15, 0x180719316D622D84), (16, 0x98C90B57FDFCB55C), (27, 0x88945AA08051FC2D),
    ]
    vectors += EXTRA_XXH64_VECTORS
    buffer = sanity_buffer(max(length for length, _ in vectors))
    for length, expected in vectors:
        assert xxh64(buffer[:length]) == expected, f"XXH64 length {length}"

    generator = Xoshiro256StarStar([1, 2, 3, 4])
    for expected in [11520, 0, 1509978240, 1215971899390074240, 1216172134540287360,
                     607988272756665600, 16172922978634559625, 8476171486693032832,
                     10595114339597558777, 2904607092377533576]:
        assert generator.next() == expected, "xoshiro256**"

    mix = SplitMix64(1477776061723855037)
    for expected in [1985237415132408290, 2979275885539914483, 13511426838097143398,
                     8488337342461049707, 15141737807933549159, 17093170987380407015,
                     16389528042912955399, 13177319091862933652, 10841969400225389492,
                     17094824097954834098, 3336622647361835228, 9678412372263018368]:
        assert mix.next() == expected, "SplitMix64"


# More XSUM_XXH64_testdata rows (seed 0), covering the 32-byte stripe loop.
EXTRA_XXH64_VECTORS = [
    (31, 0x299B39A290E6D783),
    (32, 0x18B216492BB44B70),
    (33, 0x55C8DC3E578F5B59),
    (63, 0xA9EFBE0FA0F3F4E7),
    (64, 0xEF558F8ACAC2B5CD),
    (65, 0xDE0F20DC2631AF7A),
    (100, 0x4BFE019CD91D9EA4),
    (255, 0xA80F35BB0DC8E3A7),
    (256, 0x5E3F5BF94D574981),
    (1024, 0x4775BF7CACE4D177),
    (4160, 0xEEE6A4E2AC952A5E),
]


def rust_u64(value):
    digits = f"{value:016X}"
    return "0x" + "_".join(digits[i:i + 4] for i in range(0, 16, 4))


def rust_u32(value):
    digits = f"{value:08X}"
    return "0x" + digits[:4] + "_" + digits[4:]


def f32_bits(value):
    return struct.unpack("<I", struct.pack("<f", value))[0]


def f64_bits(value):
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def main(argv):
    self_check()
    command, args = (argv[1], argv[2:]) if len(argv) > 1 else ("", [])
    if command == "xxh64":
        for text in args:
            print(f"{rust_u64(xxh64(text.encode('utf-8')))}  {text!r}")
    elif command == "xxh64hex":
        for item in args:
            name, _, digits = item.partition("=")
            print(f"{rust_u64(xxh64(bytes.fromhex(digits)))}  {name}")
    elif command == "stream" and len(args) == 3:
        x, state = seed_stream(int(args[0], 0), int(args[1], 0))
        generator = Xoshiro256StarStar(state)
        print("x      ", rust_u64(x))
        for word in state:
            print("state  ", rust_u64(word))
        for _ in range(int(args[2], 0)):
            print("output ", rust_u64(generator.next()))
    elif command == "methods" and len(args) == 3:
        _, state = seed_stream(int(args[0], 0), int(args[1], 0))
        generator = Xoshiro256StarStar(state)
        for _ in range(int(args[2], 0)):
            print("next_u64 ", rust_u64(generator.next()))
            print("next_u32 ", rust_u32(generator.next32()))
            for bound in BOUNDS32:
                print("below_u32", rust_u32(generator.below(bound, 32)), f" bound {bound:#x}")
            for bound in BOUNDS64:
                print("below_u64", rust_u64(generator.below(bound, 64)), f" bound {bound:#x}")
            print("next_f32 ", rust_u32(generator.f32_bits()))
            print("next_f64 ", rust_u64(generator.f64_bits()))
    elif command == "max-output-state":
        # next_u64 = rotl(s1 * 5, 7) * 9 = 2^64 - 1; invert the two odd multiplications.
        inverse_9 = pow(9, -1, 1 << 64)
        inverse_5 = pow(5, -1, 1 << 64)
        rotated = (MASK * inverse_9) & MASK
        s1 = (rotl(rotated, 64 - 7) * inverse_5) & MASK
        generator = Xoshiro256StarStar([1, s1, 3, 4])
        assert generator.next() == MASK
        print("s1     ", rust_u64(s1))
    elif command == "rng-streams":
        # Workload `rng_streams` of tests/determinism.rs, with the StateHasher encodings of
        # CORE-R053: integers little-endian, floats by their bits.
        bounds32 = [1, 6, 1000, 0x80000001, 0xFFFFFFFF]
        bounds64 = [1, 6, 1000, 0x8000000000000001, 0xFFFFFFFFFFFFFFFF]
        keys = [0, MASK, xxh64(b"determinism.rng_streams")]
        data = bytearray()
        for world_seed in [0, 42, MASK]:
            for key in keys:
                _, state = seed_stream(world_seed, key)
                generator = Xoshiro256StarStar(state)
                for index in range(1000):
                    data += struct.pack("<Q", generator.next())
                    data += struct.pack("<I", generator.next32())
                    data += struct.pack("<I", generator.below(bounds32[index % 5], 32))
                    data += struct.pack("<Q", generator.below(bounds64[index % 5], 64))
                    data += struct.pack("<I", generator.f32_bits())
                    data += struct.pack("<Q", generator.f64_bits())
                data += struct.pack("<4Q", *generator.s)
        print(rust_u64(xxh64(bytes(data))), " rng_streams")
    elif command == "time":
        for rate in [1, 3, 60, 1000]:
            print(f"dt f32 rate {rate:4}", rust_u32(f32_bits(1.0 / rate)), " (1/rate rounded to f32)")
            print(f"dt f64 rate {rate:4}", rust_u64(f64_bits(1.0 / rate)))
        for count, rate in [(1, 3), (2**53 + 1, 1), (2**53 + 3, 1), (2**64 - 1, 1000), (2**64 - 1, 7)]:
            print(f"secs count {count} rate {rate}", rust_u64(f64_bits(float(count) / rate)))
        for millis, rate in [(2**64 - 1, 1), (2**64 - 1, 7), (2**64 - 1, 1000)]:
            print(f"ticks_from_millis {millis} rate {rate}:", (millis * rate + 500) // 1000)
    else:
        print(__doc__, file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
