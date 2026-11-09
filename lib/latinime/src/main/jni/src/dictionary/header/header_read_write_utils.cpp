/*
 * Copyright (C) 2013, The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "dictionary/header/header_read_write_utils.h"

#include <cctype>
#include <cstdio>
#include <memory>
#include <vector>

#include "defines.h"
#include "dictionary/utils/byte_array_utils.h"

namespace latinime {

const int HeaderReadWriteUtils::MAX_ATTRIBUTE_KEY_LENGTH = 256;
const int HeaderReadWriteUtils::MAX_ATTRIBUTE_VALUE_LENGTH = 2048;

const int HeaderReadWriteUtils::HEADER_MAGIC_NUMBER_SIZE = 4;
const int HeaderReadWriteUtils::HEADER_DICTIONARY_VERSION_SIZE = 2;
const int HeaderReadWriteUtils::HEADER_FLAG_SIZE = 2;
const int HeaderReadWriteUtils::HEADER_SIZE_FIELD_SIZE = 4;
const char *const HeaderReadWriteUtils::CODE_POINT_TABLE_KEY = "codePointTable";

typedef DictionaryHeaderStructurePolicy::AttributeMap AttributeMap;

/* static */ int HeaderReadWriteUtils::getHeaderSize(const uint8_t *const dictBuf) {
    // See the format of the header in the comment in
    // BinaryDictionaryFormatUtils::detectFormatVersion()
    return ByteArrayUtils::readUint32(dictBuf, HEADER_MAGIC_NUMBER_SIZE
            + HEADER_DICTIONARY_VERSION_SIZE + HEADER_FLAG_SIZE);
}

/* static */ HeaderReadWriteUtils::DictionaryFlags
        HeaderReadWriteUtils::getFlags(const uint8_t *const dictBuf) {
    return ByteArrayUtils::readUint16(dictBuf,
            HEADER_MAGIC_NUMBER_SIZE + HEADER_DICTIONARY_VERSION_SIZE);
}

/* static */ void HeaderReadWriteUtils::fetchAllHeaderAttributes(const uint8_t *const dictBuf,
        AttributeMap *const headerAttributes) {
    const int headerSize = getHeaderSize(dictBuf);
    int pos = getHeaderOptionsPosition();
    if (pos == NOT_A_DICT_POS) {
        // The header doesn't have header options.
        return;
    }
    int keyBuffer[MAX_ATTRIBUTE_KEY_LENGTH];
    std::unique_ptr<int[]> valueBuffer(new int[MAX_ATTRIBUTE_VALUE_LENGTH]);
    while (pos < headerSize) {
        // The values in the header don't use the code point table for their encoding.
        const int keyLength = ByteArrayUtils::readStringAndAdvancePosition(dictBuf,
                MAX_ATTRIBUTE_KEY_LENGTH, nullptr /* codePointTable */, keyBuffer, &pos);
        std::vector<int> key;
        key.insert(key.end(), keyBuffer, keyBuffer + keyLength);
        const int valueLength = ByteArrayUtils::readStringAndAdvancePosition(dictBuf,
                MAX_ATTRIBUTE_VALUE_LENGTH, nullptr /* codePointTable */, valueBuffer.get(), &pos);
        std::vector<int> value;
        value.insert(value.end(), valueBuffer.get(), valueBuffer.get() + valueLength);
        headerAttributes->insert(AttributeMap::value_type(key, value));
    }
}

/* static */ const int *HeaderReadWriteUtils::readCodePointTable(
        AttributeMap *const headerAttributes) {
    AttributeMap::key_type keyVector;
    insertCharactersIntoVector(CODE_POINT_TABLE_KEY, &keyVector);
    AttributeMap::const_iterator it = headerAttributes->find(keyVector);
    if (it == headerAttributes->end()) {
        return nullptr;
    }
    return it->second.data();
}

/* static */ const std::vector<int> HeaderReadWriteUtils::readCodePointVectorAttributeValue(
        const AttributeMap *const headerAttributes, const char *const key) {
    AttributeMap::key_type keyVector;
    insertCharactersIntoVector(key, &keyVector);
    AttributeMap::const_iterator it = headerAttributes->find(keyVector);
    if (it == headerAttributes->end()) {
        return std::vector<int>();
    } else {
        return it->second;
    }
}

/* static */ bool HeaderReadWriteUtils::readBoolAttributeValue(
        const AttributeMap *const headerAttributes, const char *const key,
        const bool defaultValue) {
    const int intDefaultValue = defaultValue ? 1 : 0;
    const int intValue = readIntAttributeValue(headerAttributes, key, intDefaultValue);
    return intValue != 0;
}

/* static */ int HeaderReadWriteUtils::readIntAttributeValue(
        const AttributeMap *const headerAttributes, const char *const key,
        const int defaultValue) {
    AttributeMap::key_type keyVector;
    insertCharactersIntoVector(key, &keyVector);
    return readIntAttributeValueInner(headerAttributes, &keyVector, defaultValue);
}

/* static */ int HeaderReadWriteUtils::readIntAttributeValueInner(
        const AttributeMap *const headerAttributes, const AttributeMap::key_type *const key,
        const int defaultValue) {
    AttributeMap::const_iterator it = headerAttributes->find(*key);
    if (it != headerAttributes->end()) {
        int value = 0;
        bool isNegative = false;
        for (size_t i = 0; i < it->second.size(); ++i) {
            if (i == 0 && it->second.at(i) == '-') {
                isNegative = true;
            } else {
                if (!isdigit(it->second.at(i))) {
                    // If not a number.
                    return defaultValue;
                }
                value *= 10;
                value += it->second.at(i) - '0';
            }
        }
        return isNegative ? -value : value;
    }
    return defaultValue;
}

/* static */ void HeaderReadWriteUtils::insertCharactersIntoVector(const char *const characters,
        std::vector<int> *const vector) {
    for (int i = 0; characters[i]; ++i) {
        vector->push_back(characters[i]);
    }
}

} // namespace latinime
