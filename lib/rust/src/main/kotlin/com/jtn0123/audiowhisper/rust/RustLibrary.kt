package com.jtn0123.audiowhisper.rust

import java.lang.ref.Cleaner

/**
 * libawnative, the keyboard's Rust library, loaded once per process. Each `*Native` object
 * touches [load] before its first call. On a phone it comes from the APK; in JVM unit tests,
 * from the computer's build that Gradle puts on `java.library.path` (see lib/rust/build.gradle.kts).
 */
internal object RustLibrary {
    /** Frees native objects once nothing can reach their Kotlin owners. */
    val cleaner: Cleaner = Cleaner.create()

    init {
        NativeRuntime.requireLoaded()
    }

    /** Loads the library, if this is the first use. */
    fun load() = Unit
}
