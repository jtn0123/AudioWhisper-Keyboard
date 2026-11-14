package dev.patrickgold.florisboard.ime.input

import android.os.SystemClock
import android.util.Log
import dev.patrickgold.florisboard.BuildConfig
import dev.patrickgold.florisboard.ime.text.key.KeyCode

/** Opt-in debug timing probe. Only timestamps, never editor data or key labels. */
internal object DeleteTiming {
    private const val TAG = "aw-delete"
    private fun enabled(code: Int) = BuildConfig.DEBUG && code == KeyCode.DELETE && Log.isLoggable(TAG, Log.DEBUG)

    fun event(code: Int, down: Long, phase: String) {
        if (enabled(code)) Log.d(TAG, "$phase down_ms=$down at_ms=${SystemClock.uptimeMillis()}")
    }

    fun wait(code: Int, down: Long, phase: String, delay: Long, referenceDelay: Long) {
        if (!enabled(code)) return
        val now = SystemClock.uptimeMillis()
        Log.d(TAG, "$phase down_ms=$down at_ms=$now due_ms=${now + delay} reference_due_ms=${now + referenceDelay}")
    }
}
