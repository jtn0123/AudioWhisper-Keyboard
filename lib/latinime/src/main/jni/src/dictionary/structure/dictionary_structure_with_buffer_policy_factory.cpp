/*
 * Copyright (C) 2013 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "dictionary/structure/dictionary_structure_with_buffer_policy_factory.h"

#include "defines.h"
#include "dictionary/structure/v2/patricia_trie_policy.h"
#include "dictionary/utils/format_utils.h"
#include "dictionary/utils/mmapped_buffer.h"
#include "utils/byte_array_view.h"

namespace latinime {

/* static */ DictionaryStructureWithBufferPolicy::StructurePolicyPtr
        DictionaryStructureWithBufferPolicyFactory::newPolicyForExistingDictFile(
                const char *const path, const int bufOffset, const int size,
                const bool isUpdatable) {
    if (isUpdatable) {
        AKLOGE("Only read-only dictionaries are supported. path: %s", path);
        ASSERT(false);
        return nullptr;
    }
    return newPolicyForFileDict(path, bufOffset, size);
}

/* static */ DictionaryStructureWithBufferPolicy::StructurePolicyPtr
        DictionaryStructureWithBufferPolicyFactory::newPolicyForFileDict(
                const char *const path, const int bufOffset, const int size) {
    // Allocated buffer in MmapedBuffer::openBuffer() will be freed in the destructor of
    // MmappedBufferPtr if the instance has the responsibility.
    MmappedBuffer::MmappedBufferPtr mmappedBuffer(
            MmappedBuffer::openBuffer(path, bufOffset, size, false /* isUpdatable */));
    if (!mmappedBuffer) {
        return nullptr;
    }
    switch (FormatUtils::detectFormatVersion(mmappedBuffer->getReadOnlyByteArrayView())) {
        case FormatUtils::VERSION_2:
        case FormatUtils::VERSION_201:
            AKLOGE("Dictionary versions 2 and 201 are incompatible with this version");
            break;
        case FormatUtils::VERSION_202:
            return DictionaryStructureWithBufferPolicy::StructurePolicyPtr(
                    new PatriciaTriePolicy(std::move(mmappedBuffer)));
        case FormatUtils::VERSION_4_ONLY_FOR_TESTING:
        case FormatUtils::VERSION_402:
        case FormatUtils::VERSION_403:
            AKLOGE("Version 4 dictionaries are not supported. path: %s", path);
            break;
        default:
            AKLOGE("DICT: dictionary format is unknown, bad magic number. path: %s", path);
            break;
    }
    ASSERT(false);
    return nullptr;
}

} // namespace latinime
