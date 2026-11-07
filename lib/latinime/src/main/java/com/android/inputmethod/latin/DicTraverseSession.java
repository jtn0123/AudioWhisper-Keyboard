/*
 * Copyright (C) 2012 The Android Open Source Project
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

package com.android.inputmethod.latin;

import java.util.Arrays;

/**
 * A native decoder session plus the input/output buffers one
 * {@link BinaryDictionary#getSuggestions} call reads and writes. Not thread-safe: callers must
 * serialize use of a session.
 */
public final class DicTraverseSession {
    /** Must equal MAX_RESULTS in native defines.h. */
    public static final int MAX_RESULTS = 18;
    /** Native NOT_A_COORDINATE: no touch position for this input code point. */
    public static final int NOT_A_COORDINATE = -1;
    /** Dictionary.NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL: let native pick the weight. */
    private static final float NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL = -1.0f;

    // Indices into the native SuggestOptions array (suggest_options.h).
    private static final int OPTION_IS_GESTURE = 0;
    private static final int OPTION_USE_FULL_EDIT_DISTANCE = 1;
    private static final int OPTION_BLOCK_OFFENSIVE_WORDS = 2;
    private static final int OPTION_SPACE_AWARE_GESTURE_ENABLED = 3;
    private static final int OPTION_WEIGHT_FOR_LOCALE_IN_THOUSANDS = 4;
    private static final int OPTIONS_SIZE = 5;

    static {
        NativeLibrary.load();
    }

    // Inputs.
    public final int[] mInputCodePoints = new int[BinaryDictionary.MAX_WORD_LENGTH];
    public final int[] mXCoordinates = new int[BinaryDictionary.MAX_WORD_LENGTH];
    public final int[] mYCoordinates = new int[BinaryDictionary.MAX_WORD_LENGTH];
    public final int[] mTimes = new int[BinaryDictionary.MAX_WORD_LENGTH];
    public final int[] mPointerIds = new int[BinaryDictionary.MAX_WORD_LENGTH];
    public final int[][] mPrevWordCodePointArrays =
            new int[BinaryDictionary.MAX_PREV_WORD_COUNT_FOR_N_GRAM][];
    public final boolean[] mIsBeginningOfSentenceArray =
            new boolean[BinaryDictionary.MAX_PREV_WORD_COUNT_FOR_N_GRAM];
    final int[] mSuggestOptions = new int[OPTIONS_SIZE];

    // Outputs.
    public final int[] mOutputSuggestionCount = new int[1];
    public final int[] mOutputCodePoints = new int[BinaryDictionary.MAX_WORD_LENGTH * MAX_RESULTS];
    public final int[] mSpaceIndices = new int[MAX_RESULTS];
    public final int[] mOutputScores = new int[MAX_RESULTS];
    public final int[] mOutputTypes = new int[MAX_RESULTS];
    public final int[] mOutputAutoCommitFirstWordConfidence = new int[1];
    public final float[] mInputOutputWeightOfLangModelVsSpatialModel = new float[1];

    private long mNativeDicTraverseSession;

    public DicTraverseSession(final String locale, final BinaryDictionary dictionary) {
        mNativeDicTraverseSession = setDicTraverseSessionNative(locale, dictionary.getDictSize());
        initDicTraverseSessionNative(mNativeDicTraverseSession, dictionary.getNativeHandle(),
                null /* previousWord */, 0 /* previousWordLength */);
    }

    public long getSession() {
        return mNativeDicTraverseSession;
    }

    /** Clears the inputs and sets the options for a typing (non-gesture) request. */
    public void resetForTyping(final boolean blockOffensiveWords) {
        Arrays.fill(mInputCodePoints, 0);
        Arrays.fill(mXCoordinates, NOT_A_COORDINATE);
        Arrays.fill(mYCoordinates, NOT_A_COORDINATE);
        Arrays.fill(mTimes, 0);
        Arrays.fill(mPointerIds, 0);
        Arrays.fill(mPrevWordCodePointArrays, null);
        Arrays.fill(mIsBeginningOfSentenceArray, false);
        mSuggestOptions[OPTION_IS_GESTURE] = 0;
        mSuggestOptions[OPTION_USE_FULL_EDIT_DISTANCE] = 0;
        mSuggestOptions[OPTION_BLOCK_OFFENSIVE_WORDS] = blockOffensiveWords ? 1 : 0;
        mSuggestOptions[OPTION_SPACE_AWARE_GESTURE_ENABLED] = 0;
        mSuggestOptions[OPTION_WEIGHT_FOR_LOCALE_IN_THOUSANDS] = 1000;
        mInputOutputWeightOfLangModelVsSpatialModel[0] = NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL;
    }

    public void close() {
        if (mNativeDicTraverseSession != 0) {
            releaseDicTraverseSessionNative(mNativeDicTraverseSession);
            mNativeDicTraverseSession = 0;
        }
    }

    private static native long setDicTraverseSessionNative(String locale, long dictSize);
    private static native void initDicTraverseSessionNative(long nativeDicTraverseSession,
            long dictionary, int[] previousWord, int previousWordLength);
    private static native void releaseDicTraverseSessionNative(long nativeDicTraverseSession);
}
