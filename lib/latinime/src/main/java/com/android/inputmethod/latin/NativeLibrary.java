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

/** Loads libjni_latinime once; its JNI_OnLoad registers the natives of every glue class. */
public final class NativeLibrary {
    private static final String LIBRARY_NAME = "jni_latinime";
    private static boolean sLoaded;

    private NativeLibrary() {
        // Static helper.
    }

    /** Idempotent. Throws {@link UnsatisfiedLinkError} if the library or its registration fails. */
    public static synchronized void load() {
        if (sLoaded) return;
        System.loadLibrary(LIBRARY_NAME);
        sLoaded = true;
    }
}
