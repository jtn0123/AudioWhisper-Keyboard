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

import com.android.inputmethod.latin.NativeLibrary;

/** Glue for the static native helpers; JNI_OnLoad requires all three natives to exist. */
public final class BinaryDictionaryUtils {
    static {
        NativeLibrary.load();
    }

    private BinaryDictionaryUtils() {
        // Static helper.
    }

    /**
     * AOSP's normalized autocorrect score: 0 is a bad match, ~0.2 is enough to autocorrect,
     * ~1.5 is a very good match.
     */
    public static float calcNormalizedScore(final int[] before, final int[] after,
            final int score) {
        return calcNormalizedScoreNative(before, after, score);
    }

    private static native boolean createEmptyDictFileNative(String filePath, long dictVersion,
            String locale, String[] attributeKeyStringArray, String[] attributeValueStringArray);
    private static native float calcNormalizedScoreNative(int[] before, int[] after, int score);
    private static native int setCurrentTimeForTestNative(int currentTime);
}
