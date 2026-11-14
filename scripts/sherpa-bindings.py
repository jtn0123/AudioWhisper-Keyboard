"""Generate the used config ABI from the pinned v2.0.1 header, without downloading at build time.

Usage: python3 -I scripts/sherpa-bindings.py path/to/c-api.h OUTPUT
All fields of every reachable structure are retained, including unused model families.
"""
import hashlib
from pathlib import Path
import re
import sys

source = Path(sys.argv[1]).read_bytes()
PIN = '5698f38ba09ad2bdf085c56a6cbbd7741acf15e25f4ce5c7097b9702cadf437f'
if hashlib.sha256(source).hexdigest() != PIN:
    raise ValueError('Wrong sherpa-onnx v2.0.1 C header')
clean = re.sub(r'/\*.*?\*/|//[^\n]*', '', source.decode(), flags=re.S)
structs = dict(re.findall(r'typedef struct (SherpaOnnx\w+)\s*\{([^}]+)\}\s*\w+;', clean))
roots = ['SherpaOnnxOfflineRecognizerConfig', 'SherpaOnnxVadModelConfig']
used = []

def visit(name):
    if name in used:
        return
    for child in re.findall(r'SherpaOnnx\w+', structs[name]):
        visit(child)
    used.append(name)

for root in roots:
    visit(root)
lines = ['//! Exact configuration ABI from sherpa-onnx v2.0.1 (Apache-2.0).',
         '//! Generated with scripts/sherpa-bindings.py; never omit unused trailing fields.',
         'use std::ffi::c_char;', '']
for name in used:
    lines += ['#[repr(C)]', '#[derive(Debug, Default)]', 'pub struct ' + name.removeprefix('SherpaOnnx') + ' {']
    for field in structs[name].split(';'):
        field = field.strip()
        if not field:
            continue
        match = re.fullmatch(r'(const char\s*\*|int32_t|float|SherpaOnnx\w+)\s*(\w+)', field)
        if not match:
            raise ValueError(field)
        ty, key = match.groups()
        ty = {'const char *': '*const c_char', 'int32_t': 'i32', 'float': 'f32'}.get(ty, ty.removeprefix('SherpaOnnx'))
        lines.append(f'    pub {key}: {ty},')
    lines += ['}', '']
Path(sys.argv[2]).write_text('\n'.join(lines))

# Ask the C compiler for the ABI independently of Rust's field/layout rules.
# Header inputs were pinned above; generated numeric tests need no C compiler during Gradle.
import subprocess
import tempfile
with tempfile.TemporaryDirectory(dir=root if (root := Path('build')).is_dir() else None) as scratch:
    scratch = Path(scratch)
    code = ['#include <stdio.h>', '#include <stddef.h>', '#include "' + str(Path(sys.argv[1]).resolve()) + '"', 'int main(void) {']
    checks = []
    for name in used:
        rust = name.removeprefix('SherpaOnnx')
        code += [f'printf("%zu\\n", sizeof({name}));', f'printf("%zu\\n", _Alignof({name}));']
        checks += [f'size_of::<{rust}>()', f'align_of::<{rust}>()']
        for field in structs[name].split(';'):
            if not field.strip():
                continue
            key = re.search(r'(\w+)\s*$', field).group(1)
            code.append(f'printf("%zu\\n", offsetof({name}, {key}));')
            checks.append(f'offset_of!({rust}, {key})')
    code.append('return 0; }')
    (scratch / 'layout.c').write_text('\n'.join(code))
    subprocess.run(['cc', '-std=c11', str(scratch / 'layout.c'), '-o', str(scratch / 'layout')], check=True)
    values = subprocess.check_output([str(scratch / 'layout')], text=True).splitlines()
    assert len(values) == len(checks)
    tests = ['//! ABI values independently emitted by the C compiler for the pinned header.',
             'use super::*;', 'use std::mem::{size_of, align_of, offset_of};', '#[test]', 'fn exact_v2_0_1_c_layout() {']
    tests += [f'    assert_eq!({check}, {value});' for check, value in zip(checks, values)]
    tests.append('}')
    target = Path(sys.argv[2]).parent / 'config/tests.rs'
    target.parent.mkdir(exist_ok=True)
    target.write_text('\n'.join(tests) + '\n')
with Path(sys.argv[2]).open('a') as output:
    output.write('\n#[cfg(all(test, target_pointer_width = "64"))]\nmod tests;\n')
