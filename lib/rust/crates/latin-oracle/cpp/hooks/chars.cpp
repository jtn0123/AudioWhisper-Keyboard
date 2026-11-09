// Hooks into LatinIME's CharUtils, for comparing crates/latin's `chars` with it.
#include "utils/char_utils.h"

using latinime::CharUtils;

extern "C" {
int aw_hook_to_lower_case(int c) { return CharUtils::toLowerCase(c); }
int aw_hook_to_base_code_point(int c) { return CharUtils::toBaseCodePoint(c); }
int aw_hook_to_base_lower_case(int c) { return CharUtils::toBaseLowerCase(c); }
}
