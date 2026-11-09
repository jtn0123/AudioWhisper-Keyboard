// Hooks into LatinIME's ByteArrayUtils and FormatUtils, for comparing crates/latin's `bytes`.
// The caller pads every buffer, so the C++ (which reads without bounds checks) stays inside it.
#include <cstdint>

#include "dictionary/utils/byte_array_utils.h"
#include "dictionary/utils/format_utils.h"
#include "utils/byte_array_view.h"

using latinime::ByteArrayUtils;
using latinime::FormatUtils;

extern "C" {
uint32_t aw_hook_read_uint(const uint8_t *buffer, int size, int pos) {
    return ByteArrayUtils::readUint(buffer, size, pos);
}
int aw_hook_read_s24_advance(const uint8_t *buffer, int *pos) {
    return ByteArrayUtils::readSint24AndAdvancePosition(buffer, pos);
}
int aw_hook_read_code_point_advance(const uint8_t *buffer, const int *table, int *pos) {
    return ByteArrayUtils::readCodePointAndAdvancePosition(buffer, table, pos);
}
int aw_hook_read_string_advance(const uint8_t *buffer, int maxLength, const int *table, int *out,
        int *pos) {
    return ByteArrayUtils::readStringAndAdvancePosition(buffer, maxLength, table, out, pos);
}
int aw_hook_advance_past_string(const uint8_t *buffer, int maxLength, int *pos) {
    return ByteArrayUtils::advancePositionToBehindString(buffer, maxLength, pos);
}
int aw_hook_detect_format_version(const uint8_t *buffer, size_t size) {
    return FormatUtils::detectFormatVersion(latinime::ReadOnlyByteArrayView(buffer, size));
}
}
