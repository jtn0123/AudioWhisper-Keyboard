"""Verify normal APKs contain the pinned C/ORT payload and no sherpa JNI/classes."""
import hashlib
import json
from pathlib import Path
import sys
import zipfile

root = Path(__file__).resolve().parents[1]
apk = Path(sys.argv[1])
aar = root / 'app/libs/sherpa-onnx-2.0.1.aar'
assert hashlib.sha256(aar.read_bytes()).hexdigest() == 'a8d09d1d32f3eb8520482c6d3d43fae674681c5b050385967c70a3c2c9bf0ee7'
with zipfile.ZipFile(aar) as archive:
    expected = {name: archive.read('jni/arm64-v8a/' + name) for name in
                ('libsherpa-onnx-c-api.so', 'libonnxruntime.so')}
with zipfile.ZipFile(apk) as archive:
    libs = {Path(n).name for n in archive.namelist() if n.endswith('.so') and ('sherpa' in n or 'onnxruntime' in n)}
    assert libs == set(expected), 'Unexpected sherpa/ORT payload'
    for name, data in expected.items():
        assert archive.read('lib/arm64-v8a/' + name) == data, 'Packaged library differs from pinned AAR'
    for name in archive.namelist():
        if name.endswith('.dex'):
            assert b'com/k2fsa/sherpa/onnx/' not in archive.read(name), 'sherpa Kotlin/JNI descriptors remain'
report = {'apk_bytes': apk.stat().st_size, 'apk_sha256': hashlib.sha256(apk.read_bytes()).hexdigest(),
          'native_payload_sha256': {name: hashlib.sha256(data).hexdigest() for name, data in expected.items()},
          'sherpa_kotlin_classes': 0, 'sherpa_jni_libraries': 0}
print(json.dumps(report, indent=2))
