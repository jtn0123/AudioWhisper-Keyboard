/*
 * Copyright (C) 2011 The Android Open Source Project
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

package com.android.inputmethod.keyboard;

import com.android.inputmethod.latin.NativeLibrary;

/**
 * Native keyboard geometry the typing decoder uses to rate how plausible a mistyped letter is.
 * The caller computes the grid and key arrays; this class only hands them to native code.
 */
public final class ProximityInfo {
    /** Must equal MAX_PROXIMITY_CHARS_SIZE in native defines.h. */
    public static final int MAX_PROXIMITY_CHARS_SIZE = 16;

    static {
        NativeLibrary.load();
    }

    private long mNativeProximityInfo;

    /**
     * @param proximityCharsArray {@code gridWidth * gridHeight * MAX_PROXIMITY_CHARS_SIZE} key
     *     codes near each grid cell, padded with -1.
     * @param keyCount number of entries in each key array.
     */
    public ProximityInfo(final int keyboardWidth, final int keyboardHeight, final int gridWidth,
            final int gridHeight, final int mostCommonKeyWidth, final int mostCommonKeyHeight,
            final int[] proximityCharsArray, final int keyCount, final int[] keyXCoordinates,
            final int[] keyYCoordinates, final int[] keyWidths, final int[] keyHeights,
            final int[] keyCharCodes) {
        mNativeProximityInfo = setProximityInfoNative(keyboardWidth, keyboardHeight, gridWidth,
                gridHeight, mostCommonKeyWidth, mostCommonKeyHeight, proximityCharsArray, keyCount,
                keyXCoordinates, keyYCoordinates, keyWidths, keyHeights, keyCharCodes,
                null /* sweetSpotCenterXs */, null /* sweetSpotCenterYs */,
                null /* sweetSpotRadii */);
    }

    public long getNativeProximityInfo() {
        return mNativeProximityInfo;
    }

    public synchronized void close() {
        if (mNativeProximityInfo != 0) {
            releaseProximityInfoNative(mNativeProximityInfo);
            mNativeProximityInfo = 0;
        }
    }

    private static native long setProximityInfoNative(int displayWidth, int displayHeight,
            int gridWidth, int gridHeight, int mostCommonKeyWidth, int mostCommonKeyHeight,
            int[] proximityCharsArray, int keyCount, int[] keyXCoordinates, int[] keyYCoordinates,
            int[] keyWidths, int[] keyHeights, int[] keyCharCodes, float[] sweetSpotCenterXs,
            float[] sweetSpotCenterYs, float[] sweetSpotRadii);

    private static native void releaseProximityInfoNative(long nativeProximityInfo);
}
