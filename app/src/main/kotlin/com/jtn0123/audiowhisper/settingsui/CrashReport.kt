package com.jtn0123.audiowhisper.settingsui

/** Bootstrap diagnostics have an allowlist, never a Throwable formatter or app-state dump. */
internal data class CrashReport(val summary: String, val details: String) {
    companion object {
        private val bootstrapSites = mapOf(
            "com.jtn0123.audiowhisper.rust.NativeRuntime" to setOf("load", "requireLoaded"),
            "com.jtn0123.audiowhisper.rust.RustLibrary" to setOf("load"),
            "java.lang.System" to setOf("loadLibrary"),
            "java.lang.Runtime" to setOf("loadLibrary0", "nativeLoad"),
            "dev.patrickgold.florisboard.FlorisApplication" to setOf("onCreate"),
        )
        fun nativeLoad(failure: Throwable): CrashReport {
            val category = when (failure) {
                is UnsatisfiedLinkError -> "UnsatisfiedLinkError"
                is LinkageError -> "LinkageError"
                is SecurityException -> "SecurityException"
                else -> "NativeLoadFailure"
            }
            // Both class AND method must be compiled bootstrap identifiers. Even a forged
            // stack element cannot export an arbitrary valid-looking user string.
            val frames = failure.stackTrace.take(32).mapNotNull { frame ->
                if (bootstrapSites[frame.className]?.contains(frame.methodName) == true) {
                    "${frame.className}.${frame.methodName}:${frame.lineNumber.coerceAtLeast(0)}"
                } else null
            }
            return CrashReport("AudioWhisper could not load its keyboard engine. Reinstall the app, then open it again.",
                (listOf("AudioWhisper native engine unavailable", "library=awnative", "category=$category") + frames).joinToString("\n"))
        }
    }
}
