/*
 * Copyright (C) 2008 The Android Open Source Project
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

/**
 * Minimal read-only glue for the AOSP native dictionary in libjni_latinime.so.
 *
 * <p>The library's JNI_OnLoad registers every native below by name and signature, and refuses
 * to load if any one is missing; it registers only these, the ones the keyboard calls.
 */
public final class BinaryDictionary {
    /** Must equal MAX_WORD_LENGTH in native defines.h. */
    public static final int MAX_WORD_LENGTH = 48;
    /** Must equal MAX_PREV_WORD_COUNT_FOR_N_GRAM in native defines.h. */
    public static final int MAX_PREV_WORD_COUNT_FOR_N_GRAM = 3;
    /** Native NOT_A_PROBABILITY: the word is not in the dictionary. */
    public static final int NOT_A_PROBABILITY = -1;

    static {
        NativeLibrary.load();
    }

    private long mNativeDict;
    private final long mDictSize;

    /**
     * Opens a dictionary that lives at [offset, offset + length) inside {@code path}, which may
     * be a plain file or an uncompressed APK entry.
     */
    public BinaryDictionary(final String path, final long offset, final long length) {
        mDictSize = length;
        mNativeDict = openNative(path, offset, length, false /* isUpdatable */);
    }

    public boolean isValid() {
        return mNativeDict != 0;
    }

    /** The size passed to the constructor; native traverse sessions use it to size caches. */
    public long getDictSize() {
        return mDictSize;
    }

    /** The native handle, for {@link DicTraverseSession#init}. */
    public long getNativeHandle() {
        return mNativeDict;
    }

    /** Unigram probability (0..255), or {@link #NOT_A_PROBABILITY} if the word is unknown. */
    public int getProbability(final int[] codePoints) {
        if (!isValid()) return NOT_A_PROBABILITY;
        return getProbabilityNative(mNativeDict, codePoints);
    }

    /**
     * Probability of {@code word} directly after the given previous words, or
     * {@link #NOT_A_PROBABILITY} if the dictionary has no such n-gram (or does not know the word).
     * The arrays hold exactly the previous words to use, most recent first.
     */
    public int getNgramProbability(final int[][] prevWordCodePointArrays,
            final boolean[] isBeginningOfSentenceArray, final int[] word) {
        if (!isValid()) return NOT_A_PROBABILITY;
        return getNgramProbabilityNative(mNativeDict, prevWordCodePointArrays,
                isBeginningOfSentenceArray, word);
    }

    /**
     * Every word that has a probability, each followed by {@code '\n'}, in the dictionary's own
     * order: entries that are only shortcuts, the sentence-start marker and words holding a
     * control character are left out. One native walk of the whole dictionary.
     */
    public String getWords() {
        if (!isValid()) return "";
        final String words = getWordsNative(mNativeDict);
        return words != null ? words : "";
    }

    /**
     * Runs the typing decoder (or, with {@code inputSize == 0}, next-word prediction) and writes
     * the results into {@code session}'s output buffers.
     */
    public void getSuggestions(final DicTraverseSession session, final long proximityInfo,
            final int inputSize, final int prevWordCount) {
        session.mOutputSuggestionCount[0] = 0;
        if (!isValid()) return;
        getSuggestionsNative(mNativeDict, proximityInfo, session.getSession(),
                session.mXCoordinates, session.mYCoordinates, session.mTimes, session.mPointerIds,
                session.mInputCodePoints, inputSize, session.mSuggestOptions,
                session.mPrevWordCodePointArrays, session.mIsBeginningOfSentenceArray,
                prevWordCount, session.mOutputSuggestionCount, session.mOutputCodePoints,
                session.mOutputScores, session.mSpaceIndices, session.mOutputTypes,
                session.mOutputAutoCommitFirstWordConfidence,
                session.mInputOutputWeightOfLangModelVsSpatialModel);
    }

    public synchronized void close() {
        if (mNativeDict != 0) {
            closeNative(mNativeDict);
            mNativeDict = 0;
        }
    }

    private static native long openNative(String sourceDir, long dictOffset, long dictSize,
            boolean isUpdatable);
    private static native void closeNative(long dict);
    private static native int getProbabilityNative(long dict, int[] word);
    private static native int getNgramProbabilityNative(long dict, int[][] prevWordCodePointArrays,
            boolean[] isBeginningOfSentenceArray, int[] word);
    private static native String getWordsNative(long dict);
    private static native void getSuggestionsNative(long dict, long proximityInfo,
            long traverseSession, int[] xCoordinates, int[] yCoordinates, int[] times,
            int[] pointerIds, int[] inputCodePoints, int inputSize, int[] suggestOptions,
            int[][] prevWordCodePointArrays, boolean[] isBeginningOfSentenceArray,
            int prevWordCount, int[] outputSuggestionCount, int[] outputCodePoints,
            int[] outputScores, int[] outputIndices, int[] outputTypes,
            int[] outputAutoCommitFirstWordConfidence,
            float[] inOutWeightOfLangModelVsSpatialModel);
}
