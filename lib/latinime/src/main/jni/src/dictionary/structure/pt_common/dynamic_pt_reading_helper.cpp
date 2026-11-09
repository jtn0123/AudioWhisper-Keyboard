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

#include "dictionary/structure/pt_common/dynamic_pt_reading_helper.h"

#include "dictionary/structure/pt_common/pt_node_array_reader.h"
#include "utils/char_utils.h"
#include "utils/scratch_buffer.h"

namespace latinime {

// To avoid infinite loop caused by invalid or malicious forward links.
const int DynamicPtReadingHelper::MAX_CHILD_COUNT_TO_AVOID_INFINITE_LOOP = 100000;
const int DynamicPtReadingHelper::MAX_PT_NODE_ARRAY_COUNT_TO_AVOID_INFINITE_LOOP = 100000;

int DynamicPtReadingHelper::getTerminalPtNodePositionOfWord(const int *const inWord,
        const size_t length, const bool forceLowerCaseSearch) {
    ScratchBuffer<int, MAX_WORD_LENGTH> searchCodePointsStorage(static_cast<int>(length));
    int *const searchCodePoints = searchCodePointsStorage.data();
    for (size_t i = 0; i < length; ++i) {
        searchCodePoints[i] = forceLowerCaseSearch ? CharUtils::toLowerCase(inWord[i]) : inWord[i];
    }
    while (!isEnd()) {
        const PtNodeParams ptNodeParams(getPtNodeParams());
        const int matchedCodePointCount = getPrevTotalCodePointCount();
        if (getTotalCodePointCount(ptNodeParams) > length
                || !isMatchedCodePoint(ptNodeParams, 0 /* index */,
                        searchCodePoints[matchedCodePointCount])) {
            // Current node has too many code points or its first code point is different from
            // target code point. Skip this node and read the next sibling node.
            readNextSiblingNode(ptNodeParams);
            continue;
        }
        // Check following merged node code points.
        const int nodeCodePointCount = ptNodeParams.getCodePointCount();
        for (int j = 1; j < nodeCodePointCount; ++j) {
            if (!isMatchedCodePoint(ptNodeParams, j, searchCodePoints[matchedCodePointCount + j])) {
                // Different code point is found. The given word is not included in the dictionary.
                return NOT_A_DICT_POS;
            }
        }
        // All characters are matched.
        if (length == getTotalCodePointCount(ptNodeParams)) {
            if (!ptNodeParams.isTerminal()) {
                return NOT_A_DICT_POS;
            }
            // Terminal position is found.
            return ptNodeParams.getHeadPos();
        }
        if (!ptNodeParams.hasChildren()) {
            return NOT_A_DICT_POS;
        }
        // Advance to the children nodes.
        readChildNode(ptNodeParams);
    }
    // If we already traversed the tree further than the word is long, there means
    // there was no match (or we would have found it).
    return NOT_A_DICT_POS;
}

// Read node array size and process empty node arrays. Nodes and arrays are counted up in this
// method to avoid an infinite loop.
void DynamicPtReadingHelper::nextPtNodeArray() {
    int ptNodeCountInArray = 0;
    int firstPtNodePos = NOT_A_DICT_POS;
    if (!mPtNodeArrayReader->readPtNodeArrayInfoAndReturnIfValid(
            mReadingState.mPos, &ptNodeCountInArray, &firstPtNodePos)) {
        mIsError = true;
        mReadingState.mPos = NOT_A_DICT_POS;
        return;
    }
    mReadingState.mRemainingPtNodeCountInThisArray = ptNodeCountInArray;
    mReadingState.mPos = firstPtNodePos;
    // Count up nodes and node arrays to avoid infinite loop.
    mReadingState.mTotalPtNodeIndexInThisArrayChain +=
            mReadingState.mRemainingPtNodeCountInThisArray;
    mReadingState.mPtNodeArrayIndexInThisArrayChain++;
    if (mReadingState.mRemainingPtNodeCountInThisArray < 0
            || mReadingState.mTotalPtNodeIndexInThisArrayChain
                    > MAX_CHILD_COUNT_TO_AVOID_INFINITE_LOOP
            || mReadingState.mPtNodeArrayIndexInThisArrayChain
                    > MAX_PT_NODE_ARRAY_COUNT_TO_AVOID_INFINITE_LOOP) {
        // Invalid dictionary.
        AKLOGI("Invalid dictionary. nodeCount: %d, totalNodeCount: %d, MAX_CHILD_COUNT: %d"
                "nodeArrayCount: %d, MAX_NODE_ARRAY_COUNT: %d",
                mReadingState.mRemainingPtNodeCountInThisArray,
                mReadingState.mTotalPtNodeIndexInThisArrayChain,
                MAX_CHILD_COUNT_TO_AVOID_INFINITE_LOOP,
                mReadingState.mPtNodeArrayIndexInThisArrayChain,
                MAX_PT_NODE_ARRAY_COUNT_TO_AVOID_INFINITE_LOOP);
        ASSERT(false);
        mIsError = true;
        mReadingState.mPos = NOT_A_DICT_POS;
        return;
    }
    if (mReadingState.mRemainingPtNodeCountInThisArray == 0) {
        // Empty node array. Try following forward link.
        followForwardLink();
    }
}

// Follow the forward link and read the next node array if exists.
void DynamicPtReadingHelper::followForwardLink() {
    int nextPtNodeArrayPos = NOT_A_DICT_POS;
    if (!mPtNodeArrayReader->readForwardLinkAndReturnIfValid(
            mReadingState.mPos, &nextPtNodeArrayPos)) {
        mIsError = true;
        mReadingState.mPos = NOT_A_DICT_POS;
        return;
    }
    if (nextPtNodeArrayPos != NOT_A_DICT_POS) {
        // Follow the forward link.
        mReadingState.mPos = nextPtNodeArrayPos;
        nextPtNodeArray();
    } else {
        // All node arrays have been read.
        mReadingState.mPos = NOT_A_DICT_POS;
    }
}

} // namespace latinime
