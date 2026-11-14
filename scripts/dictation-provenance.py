"""Verify vendored sherpa against its published v2.0.1 SHA-256; no network in builds."""
import hashlib
from pathlib import Path
import zipfile

AAR_SHA256 = 'a8d09d1d32f3eb8520482c6d3d43fae674681c5b050385967c70a3c2c9bf0ee7'
C_API_SHA256 = '0eadae93c4d595a26584d7eebde7b397b9a36d01eee3ebc7262e0d34df2fce10'
root = Path(__file__).resolve().parents[1]
aar = root / 'app/libs/sherpa-onnx-2.0.1.aar'
assert hashlib.sha256(aar.read_bytes()).hexdigest() == AAR_SHA256
with zipfile.ZipFile(aar) as archive:
    assert hashlib.sha256(archive.read('jni/arm64-v8a/libsherpa-onnx-c-api.so')).hexdigest() == C_API_SHA256
print('Verified sherpa-onnx 2.0.1 AAR and arm64 C API SHA-256')
