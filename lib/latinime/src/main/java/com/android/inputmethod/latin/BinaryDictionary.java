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

import com.android.inputmethod.latin.utils.WordInputEventForPersonalization;

import java.util.ArrayList;

/**
 * Minimal read-only glue for the AOSP native dictionary in libjni_latinime.so.
 *
 * <p>The library's JNI_OnLoad registers every native below by name and signature, and refuses
 * to load if any one is missing, so all of them are declared even though only the read-only
 * subset is called.
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
    private static native long createOnMemoryNative(long formatVersion,
            String locale, String[] attributeKeyStringArray, String[] attributeValueStringArray);
    private static native void getHeaderInfoNative(long dict, int[] outHeaderSize,
            int[] outFormatVersion, ArrayList<int[]> outAttributeKeys,
            ArrayList<int[]> outAttributeValues);
    private static native boolean flushNative(long dict, String filePath);
    private static native boolean needsToRunGCNative(long dict, boolean mindsBlockByGC);
    private static native boolean flushWithGCNative(long dict, String filePath);
    private static native void closeNative(long dict);
    private static native int getFormatVersionNative(long dict);
    private static native int getProbabilityNative(long dict, int[] word);
    private static native int getMaxProbabilityOfExactMatchesNative(long dict, int[] word);
    private static native int getNgramProbabilityNative(long dict, int[][] prevWordCodePointArrays,
            boolean[] isBeginningOfSentenceArray, int[] word);
    private static native void getWordPropertyNative(long dict, int[] word,
            boolean isBeginningOfSentence, int[] outCodePoints, boolean[] outFlags,
            int[] outProbabilityInfo, ArrayList<int[][]> outNgramPrevWordsArray,
            ArrayList<boolean[]> outNgramPrevWordIsBeginningOfSentenceArray,
            ArrayList<int[]> outNgramTargets, ArrayList<int[]> outNgramProbabilityInfo,
            ArrayList<int[]> outShortcutTargets, ArrayList<Integer> outShortcutProbabilities);
    private static native int getNextWordNative(long dict, int token, int[] outCodePoints,
            boolean[] outIsBeginningOfSentence);
    private static native void getSuggestionsNative(long dict, long proximityInfo,
            long traverseSession, int[] xCoordinates, int[] yCoordinates, int[] times,
            int[] pointerIds, int[] inputCodePoints, int inputSize, int[] suggestOptions,
            int[][] prevWordCodePointArrays, boolean[] isBeginningOfSentenceArray,
            int prevWordCount, int[] outputSuggestionCount, int[] outputCodePoints,
            int[] outputScores, int[] outputIndices, int[] outputTypes,
            int[] outputAutoCommitFirstWordConfidence,
            float[] inOutWeightOfLangModelVsSpatialModel);
    private static native boolean addUnigramEntryNative(long dict, int[] word, int probability,
            int[] shortcutTarget, int shortcutProbability, boolean isBeginningOfSentence,
            boolean isNotAWord, boolean isPossiblyOffensive, int timestamp);
    private static native boolean removeUnigramEntryNative(long dict, int[] word);
    private static native boolean addNgramEntryNative(long dict,
            int[][] prevWordCodePointArrays, boolean[] isBeginningOfSentenceArray,
            int[] word, int probability, int timestamp);
    private static native boolean removeNgramEntryNative(long dict,
            int[][] prevWordCodePointArrays, boolean[] isBeginningOfSentenceArray, int[] word);
    private static native boolean updateEntriesForWordWithNgramContextNative(long dict,
            int[][] prevWordCodePointArrays, boolean[] isBeginningOfSentenceArray,
            int[] word, boolean isValidWord, int count, int timestamp);
    private static native int updateEntriesForInputEventsNative(long dict,
            WordInputEventForPersonalization[] inputEvents, int startIndex);
    private static native String getPropertyNative(long dict, String query);
    private static native boolean isCorruptedNative(long dict);
    private static native boolean migrateNative(long dict, String dictFilePath,
            long newFormatVersion);
}
