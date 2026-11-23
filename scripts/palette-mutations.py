"""Faults in the palette's `sin` and `cos` (crates/palette/src/jmath/trig.rs): each must compile and
then fail `cargo test -p palette`, the bionic goldens or the MaterialKolor digests; the source bytes
are always restored. python3 -I.

    python3 -I scripts/palette-mutations.py build/palette/mutations [case ...]

The faults are the ways a port of bionic's msun `sin` and `cos` goes wrong without looking wrong:
fdlibm 5.3's polynomial order instead of FreeBSD's split one, an operation fused that Android
rounds twice (`-ffp-contract=off`), a reduction round or a cancellation guard dropped, the wrong
quadrant, a constant off by its last bit, and the tiny-argument cut-offs moved. A compilation
failure never counts as caught: the mutated crate must build and a test must fail.
Run with no other cargo or Gradle operation in this checkout.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parent.parent
WORKSPACE = ROOT / 'lib/rust'
TRIG = ROOT / 'lib/rust/crates/palette/src/jmath/trig.rs'
SIN_SPLIT = 'let r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);'
COS_SPLIT = 'let r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));'
SIN_QUADRANTS = ('        1 => kernel_cos(y0, y1),\n        2 => -kernel_sin(y0, y1, true),\n'
                 '        _ => -kernel_cos(y0, y1),\n')
# Each case: name, why, [(old, new)] in trig.rs, each old held exactly once.
CASES = [
    ('sin-fdlibm-horner', "fdlibm 5.3's k_sin.c order, one Horner chain",
     [(SIN_SPLIT, 'let r = S2 + z * (S3 + z * (S4 + z * (S5 + z * S6)));')]),
    ('cos-fdlibm-horner', "fdlibm 5.3's k_cos.c order, one Horner chain",
     [(COS_SPLIT, 'let r = z * (C1 + z * (C2 + z * (C3 + z * (C4 + z * (C5 + z * C6)))));')]),
    ('sin-kernel-fused', "k_sin's last step fused, as -ffp-contract=on would",
     [('x + v * (S1 + z * r)', 'v.mul_add(S1 + z * r, x)')]),
    ('cos-kernel-fused', "k_cos's z*r - x*y fused",
     [('(z * r - x * y)', 'z.mul_add(r, -(x * y))')]),
    # Not cases: fusing a step of the reduction. `x - n * pio2_1` is exact (pio2_1 holds 33 bits),
    # and fusing `z - k * pio2_1t`, `r - n * pio2_1t` or the second round's `n * pio2_2t` term
    # changed no answer in 550 million arguments, the whole medium range and its cancellations
    # among them: the tail y1 takes up what y0 gains. Only the kernels' fusions reach a result.
    ('rnint-truncates', 'n rounded toward zero instead of to nearest',
     [('let fn_ = (x * INVPIO2 + TO_INT) - TO_INT;', 'let fn_ = (x * INVPIO2).trunc();')]),
    ('no-second-round', 'the medium reduction never takes its second 33 bits of pi/2',
     [('> 16 {', '> 1_000 {')]),
    ('no-third-round', 'the medium reduction never takes its third 33 bits of pi/2',
     [('> 49 {', '> 1_000 {')]),
    ('no-pi-guard', 'pi/2 and pi take the one-round path, which cancels there',
     [('if ix & 0xf_ffff != 0x9_21fb {', 'if ix & 0xf_ffff != 0 {')]),
    ('no-3pi2-guard', '3pi/2 takes the one-round path, which cancels there',
     [('(3, 0x4012_d97c)', '(3, 0x4012_d97d)')]),
    ('sin-quadrants-swapped', "sin's quadrants 1 and 3 exchanged",
     [(SIN_QUADRANTS, SIN_QUADRANTS.replace('1 => kernel_cos', '1 => -kernel_cos')
       .replace('_ => -kernel_cos', '_ => kernel_cos'))]),
    ('s1-last-bit', 'S1 one ulp off', [('0xBFC5_5555_5555_5549', '0xBFC5_5555_5555_554A')]),
    ('c1-last-bit', 'C1 one ulp off', [('0x3FA5_5555_5555_554C', '0x3FA5_5555_5555_554D')]),
    ('pio2-1t-last-bit', 'pi/2 - pio2_1 one ulp off', [('0x3DD0_B461_1A62_6331', '0x3DD0_B461_1A62_6330')]),
    ('sin-tiny-cutoff', "sin returns x below 2^-25, not 2^-26", [('ix < 0x3e50_0000', 'ix < 0x3e60_0000')]),
    ('cos-tiny-cutoff', "cos returns one up to 2^-26, sin's cut-off", [('ix < 0x3e46_a09e', 'ix <= 0x3e50_0000')]),
    ('kernel-bound', 'sin sends the last hi word below pi/4 through the reduction',
     [('    if ix <= 0x3fe9_21fb {\n        // |x| < 2^-26', '    if ix < 0x3fe9_21fb {\n        // |x| < 2^-26')]),
]


def check_edits():
    return [mutation_cases.Edit(name, TRIG, old, new) for name, _, edits in CASES for old, new in edits]


def env():
    e = dict(os.environ)
    e.pop('RUSTFLAGS', None)
    return e


def test(log_path):
    """`cargo test -p palette`'s exit status, whether the crate built, and the tests that failed. Run from the
    workspace, so its .cargo/config.toml puts the build under lib/rust/build/cargo."""
    with log_path.open('wb') as log:
        result = subprocess.run(['cargo', 'test', '--locked', '-p', 'palette', '--no-fail-fast'],
                                cwd=WORKSPACE, stdout=log, stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    # A failing test prints `error: test failed`; only a compiler error is `error[E...]` or `could not compile`.
    built = 'could not compile' not in text and not re.search(r'^error\[E\d+\]', text, re.M)
    failed = sorted(set(re.findall(r'^test (\S+) \.\.\. FAILED$', text, re.M)))
    return result.returncode, built, failed


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {case[0] for case in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    original = TRIG.read_bytes()
    status, built, failed = test(output / 'baseline.log')
    assert status == 0 and built and not failed, 'the unmutated source must pass first'
    results = []
    try:
        for name, why, edits in cases:
            source = original.decode()
            for old, new in edits:
                assert source.count(old) == 1, f'mutation location changed: {name}: {old!r}'
                source = source.replace(old, new)
            TRIG.write_text(source)
            try:
                status, built, failed = test(output / (name + '.log'))
            finally:
                TRIG.write_bytes(original)
            assert built, f'{name} did not compile; that never counts as caught'
            assert status != 0 and failed, f'{name} survived: every palette test passed'
            results.append({'mutation': name, 'fault': why, 'failed_tests': failed})
            print(f'{name}: caught by {len(failed)} test(s): {", ".join(failed)}', flush=True)
    finally:
        TRIG.write_bytes(original)
    status, built, failed = test(output / 'restored.log')
    assert status == 0 and built and not failed, 'restored source fails'
    report = {'mutations': results, 'restored_exit': status,
              'source_sha256': hashlib.sha256(TRIG.read_bytes()).hexdigest()}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'All {len(results)} mutations caught; restored source passes', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
