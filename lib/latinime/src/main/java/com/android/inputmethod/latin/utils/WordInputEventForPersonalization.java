/*
 * Copyright (C) 2014 The Android Open Source Project
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

package com.android.inputmethod.latin.utils;

import com.android.inputmethod.latin.BinaryDictionary;

/**
 * Parameter type of BinaryDictionary.updateEntriesForInputEventsNative. Native code reads these
 * fields by name (personalization), so the class and field names must not change. Unused here:
 * it exists so the native method's signature resolves at registration time.
 */
public final class WordInputEventForPersonalization {
    public final int[] mTargetWord;
    public final int mPrevWordsCount;
    public final int[][] mPrevWordArray =
            new int[BinaryDictionary.MAX_PREV_WORD_COUNT_FOR_N_GRAM][];
    public final boolean[] mIsPrevWordBeginningOfSentenceArray =
            new boolean[BinaryDictionary.MAX_PREV_WORD_COUNT_FOR_N_GRAM];
    public final boolean mIsValid;
    /** Seconds since the epoch. */
    public final int mTimestamp;

    public WordInputEventForPersonalization(final int[] targetWord, final int prevWordsCount,
            final boolean isValid, final int timestamp) {
        mTargetWord = targetWord;
        mPrevWordsCount = prevWordsCount;
        mIsValid = isValid;
        mTimestamp = timestamp;
    }
}
