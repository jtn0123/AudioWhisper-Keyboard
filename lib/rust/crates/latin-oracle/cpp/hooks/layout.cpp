// Hooks into LatinIME's suggest/core/layout (ProximityInfo, ProximityInfoState, the helpers), for
// comparing crates/latin's `layout` with it. Everything is read through the classes' public
// methods except two private arrays (the grid and a state's input proximities), which a
// template trick reaches without changing the C++: explicit instantiation ignores access.
#include <cstdint>
#include <cstring>
#include <vector>

#include "defines.h"
#include "suggest/core/layout/proximity_info.h"
#include "suggest/core/layout/proximity_info_state.h"
#include "suggest/core/layout/touch_position_correction_utils.h"
#include "suggest/core/session/dic_traverse_session.h"

using latinime::DicTraverseSession;
using latinime::ProximityInfo;
using latinime::ProximityInfoState;
using latinime::TouchPositionCorrectionUtils;

namespace {

template <typename Tag, typename Tag::type Member>
struct Reach {
    friend typename Tag::type reach(Tag) { return Member; }
};

struct GridMember {
    typedef int *ProximityInfo::*type;
    friend type reach(GridMember);
};
template struct Reach<GridMember, &ProximityInfo::mProximityCharsArray>;

struct ProximitiesMember {
    typedef int (ProximityInfoState::*type)[MAX_PROXIMITY_CHARS_SIZE * MAX_WORD_LENGTH];
    friend type reach(ProximitiesMember);
};
template struct Reach<ProximitiesMember, &ProximityInfoState::mInputProximities>;

const ProximityInfo *info(int64_t handle) { return reinterpret_cast<const ProximityInfo *>(handle); }
ProximityInfoState *state(int64_t handle) { return reinterpret_cast<ProximityInfoState *>(handle); }

std::vector<int> localeOf(const int *locale, int length) {
    return std::vector<int>(locale, locale + length);
}

} // namespace

extern "C" {

// ---- ProximityInfo -------------------------------------------------------------------------

// keyCount, cellWidth, cellHeight, gridWidth, gridHeight, keyboardHeight, mostCommonKeyWidth.
void aw_hook_info_params(int64_t handle, int *out) {
    const ProximityInfo *it = info(handle);
    out[0] = it->getKeyCount();
    out[1] = it->getCellWidth();
    out[2] = it->getCellHeight();
    out[3] = it->getGridWidth();
    out[4] = it->getGridHeight();
    out[5] = it->getKeyboardHeight();
    out[6] = it->getMostCommonKeyWidth();
}

// mProximityCharsArray, `count` ints.
void aw_hook_info_grid(int64_t handle, int *out, int count) {
    const ProximityInfo *it = info(handle);
    memcpy(out, it->*reach(GridMember()), count * sizeof(int));
}

// The key's lower case code point and its original one.
void aw_hook_info_key(int64_t handle, int key, int *out) {
    const ProximityInfo *it = info(handle);
    out[0] = it->getCodePointOf(key);
    out[1] = it->getOriginalCodePointOf(key);
}

int aw_hook_info_key_index_of(int64_t handle, int c) { return info(handle)->getKeyIndexOf(c); }

int aw_hook_info_on_keyboard(int64_t handle, int c) {
    return info(handle)->isCodePointOnKeyboard(c) ? 1 : 0;
}

int aw_hook_info_has_space_proximity(int64_t handle, int x, int y) {
    return info(handle)->hasSpaceProximity(x, y) ? 1 : 0;
}

float aw_hook_info_normalized_squared_distance(int64_t handle, int keyId, int x, int y) {
    return info(handle)->getNormalizedSquaredDistanceFromCenterFloatG(keyId, x, y, false);
}

int aw_hook_info_key_center_x(int64_t handle, int keyId, int referenceX) {
    return info(handle)->getKeyCenterXOfKeyIdG(keyId, referenceX, false);
}

int aw_hook_info_key_center_y(int64_t handle, int keyId, int referenceY) {
    return info(handle)->getKeyCenterYOfKeyIdG(keyId, referenceY, false);
}

// ProximityInfo::initializeProximities into `out` (16 ints per input).
void aw_hook_info_initialize_proximities(int64_t handle, const int *codes, const int *xs,
        const int *ys, int size, const int *locale, int localeLength, int *out) {
    const std::vector<int> localeCodes = localeOf(locale, localeLength);
    info(handle)->initializeProximities(codes, xs, ys, size, out, &localeCodes);
}

float aw_hook_sweet_spot_factor(int enabled, float x) {
    return TouchPositionCorrectionUtils::getSweetSpotFactor(enabled != 0, x);
}

// ---- ProximityInfoState --------------------------------------------------------------------

int64_t aw_hook_state_new() { return reinterpret_cast<int64_t>(new ProximityInfoState()); }

void aw_hook_state_free(int64_t handle) { delete state(handle); }

// A decode session's state for the one pointer typing has (borrowed: not to be freed).
int64_t aw_hook_session_state(int64_t session) {
    const DicTraverseSession *it = reinterpret_cast<const DicTraverseSession *>(session);
    return reinterpret_cast<int64_t>(const_cast<ProximityInfoState *>(it->getProximityInfoState(0)));
}

// initInputParams as DicTraverseSession::initializeProximityInfoStates calls it for typing.
// `times` and `pointerIds` may be null.
void aw_hook_state_init(int64_t handle, float maxPointToKeyLength, int64_t infoHandle,
        const int *codes, int size, const int *xs, const int *ys, const int *times,
        const int *pointerIds, const int *locale, int localeLength) {
    const std::vector<int> localeCodes = localeOf(locale, localeLength);
    state(handle)->initInputParams(0, maxPointToKeyLength, info(infoHandle), codes, size, xs, ys,
            times, pointerIds, false, &localeCodes);
}

// size, isUsed, touchPositionCorrectionEnabled, isContinuousSuggestionPossible.
void aw_hook_state_summary(int64_t handle, int *out) {
    const ProximityInfoState *it = state(handle);
    out[0] = it->size();
    out[1] = it->isUsed() ? 1 : 0;
    out[2] = it->touchPositionCorrectionEnabled() ? 1 : 0;
    out[3] = it->isContinuousSuggestionPossible() ? 1 : 0;
}

// mInputProximities, every row.
void aw_hook_state_proximities(int64_t handle, int *out) {
    const ProximityInfoState *it = state(handle);
    memcpy(out, it->*reach(ProximitiesMember()), sizeof(int) * MAX_PROXIMITY_CHARS_SIZE * MAX_WORD_LENGTH);
}

// getInputX, getInputY and getInputIndexOfSampledPoint of the point.
void aw_hook_state_point(int64_t handle, int index, int *out) {
    const ProximityInfoState *it = state(handle);
    out[0] = it->getInputX(index);
    out[1] = it->getInputY(index);
    out[2] = it->getInputIndexOfSampledPoint(index);
}

float aw_hook_state_point_to_key_length(int64_t handle, int index, int codePoint) {
    return state(handle)->getPointToKeyLength(index, codePoint);
}

float aw_hook_state_point_to_key_by_id_length(int64_t handle, int index, int keyId) {
    return state(handle)->getPointToKeyByIdLength(index, keyId);
}

// The ProximityType as its enum value; `proximityIndex` is set only when the C++ sets it.
int aw_hook_state_proximity_type(int64_t handle, int index, int codePoint, int check,
        int *proximityIndex) {
    return state(handle)->getProximityType(index, codePoint, check != 0, proximityIndex);
}

int aw_hook_state_same_as_typed(int64_t handle, const int *word, int length) {
    return state(handle)->sameAsTyped(word, length) ? 1 : 0;
}

int aw_hook_state_exists_in_proximity(int64_t handle, int index, int c) {
    return state(handle)->existsCodePointInProximityAt(index, c) ? 1 : 0;
}

int aw_hook_state_exists_adjacent(int64_t handle, int index) {
    return state(handle)->existsAdjacentProximityChars(index) ? 1 : 0;
}

int aw_hook_state_has_space_proximity(int64_t handle, int index) {
    return state(handle)->hasSpaceProximity(index) ? 1 : 0;
}

int aw_hook_state_primary(int64_t handle, int index) {
    return state(handle)->getPrimaryCodePointAt(index);
}

int aw_hook_state_primary_original(int64_t handle, int index) {
    return state(handle)->getPrimaryOriginalCodePointAt(index);
}
}
