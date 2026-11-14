"""Library-name observations must understand APK-backed mmap regions."""
from pathlib import Path
import runpy
import unittest

loaded = runpy.run_path(str(Path(__file__).with_name('apk-maps.py')))['loaded']


class ApkMapsTest(unittest.TestCase):
    def maps(self, line):
        # A real process has many unrelated mappings; only the supplied line is relevant.
        return '1000-2000 rw-p 00000000 00:00 0 [anon:test]\n' * 101 + line

    def test_apk_backed_jni_is_detected_by_member_offset(self):
        entries = [('/owned/base.apk', 0x4000, 0x9000, 'libsherpa-onnx-jni.so')]
        value = loaded(self.maps('3000-4000 r-xp 00005000 00:01 1 /owned/base.apk\n'), entries)
        self.assertIn('libsherpa-onnx-jni.so', value)

    def test_apk_resource_mapping_and_other_package_do_not_imply_jni(self):
        entries = [('/owned/base.apk', 0x4000, 0x9000, 'libsherpa-onnx-jni.so')]
        for path, offset in [('/owned/base.apk', 0), ('/other/base.apk', 0x5000),
                             ('/owned/base.apk', 0x9000)]:
            value = loaded(self.maps(f'3000-4000 r-xp {offset:08x} 00:01 1 {path}\n'), entries)
            self.assertNotIn('libsherpa-onnx-jni.so', value)

    def test_extracted_library_and_deleted_apk_are_supported(self):
        entries = [('/owned/base.apk', 0x4000, 0x9000, 'libsherpa-onnx-c-api.so')]
        value = loaded(self.maps('3000-4000 r-xp 00005000 00:01 1 /owned/base.apk (deleted)\n'
            '5000-6000 r-xp 00000000 00:01 2 /owned/libonnxruntime.so\n'), entries)
        self.assertEqual({'libsherpa-onnx-c-api.so', 'libonnxruntime.so'}, value)

    def test_cat_error_is_not_an_empty_library_set(self):
        with self.assertRaisesRegex(AssertionError, 'Invalid process-map'):
            loaded('cat: /proc/1/maps: Permission denied\n', [])


if __name__ == '__main__':
    unittest.main()
