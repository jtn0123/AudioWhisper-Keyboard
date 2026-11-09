# Copyright (C) 2013 The Android Open Source Project
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

LATIN_IME_JNI_SRC_FILES := \
    com_android_inputmethod_keyboard_ProximityInfo.cpp \
    com_android_inputmethod_latin_BinaryDictionary.cpp \
    com_android_inputmethod_latin_DicTraverseSession.cpp \
    jni_common.cpp

# Every .cpp the keyboard's dictionary calls reach: the version 2 (read-only) dictionary format and
# the typing decoder. The writable formats (v4, v402), their writers and the test list are gone.
LATIN_IME_CORE_SRC_FILES := \
    $(addprefix dictionary/header/, \
        header_policy.cpp \
        header_read_write_utils.cpp) \
    dictionary/property/ngram_context.cpp \
    dictionary/structure/dictionary_structure_with_buffer_policy_factory.cpp \
    $(addprefix dictionary/structure/pt_common/, \
        bigram/bigram_list_read_write_utils.cpp \
        dynamic_pt_reading_helper.cpp \
        patricia_trie_reading_utils.cpp \
        shortcut/shortcut_list_reading_utils.cpp) \
    $(addprefix dictionary/structure/v2/, \
        patricia_trie_policy.cpp \
        ver2_patricia_trie_node_reader.cpp \
        ver2_pt_node_array_reader.cpp) \
    $(addprefix dictionary/utils/, \
        byte_array_utils.cpp \
        format_utils.cpp \
        mmapped_buffer.cpp \
        multi_bigram_map.cpp) \
    suggest/core/suggest.cpp \
    $(addprefix suggest/core/dicnode/, \
        dic_node.cpp \
        dic_node_utils.cpp \
        dic_nodes_cache.cpp) \
    $(addprefix suggest/core/dictionary/, \
        dictionary.cpp \
        digraph_utils.cpp \
        error_type_utils.cpp) \
    $(addprefix suggest/core/layout/, \
        additional_proximity_chars.cpp \
        proximity_info.cpp \
        proximity_info_params.cpp \
        proximity_info_state.cpp \
        proximity_info_state_utils.cpp) \
    suggest/core/policy/weighting.cpp \
    suggest/core/session/dic_traverse_session.cpp \
    $(addprefix suggest/core/result/, \
        suggestion_results.cpp \
        suggestions_output_utils.cpp) \
    suggest/policyimpl/gesture/gesture_suggest_policy_factory.cpp \
    $(addprefix suggest/policyimpl/typing/, \
        scoring_params.cpp \
        typing_scoring.cpp \
        typing_suggest_policy.cpp \
        typing_traversal.cpp \
        typing_weighting.cpp) \
    $(addprefix utils/, \
        char_utils.cpp \
        jni_data_utils.cpp \
        log_utils.cpp \
        time_keeper.cpp)
