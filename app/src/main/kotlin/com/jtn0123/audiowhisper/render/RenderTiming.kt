package com.jtn0123.audiowhisper.render

import android.util.Log
import android.view.View

/** Shared synthetic strip for visual comparisons, opt-in via log.tag.aw-emoji=DEBUG. */
internal fun rendererEmojiSamples(): List<String> = if (Log.isLoggable("aw-emoji", Log.DEBUG)) {
    listOf("👨‍👩‍👧‍👦", "🇺🇸", "👍🏽 1️⃣ ❤️")
} else emptyList()

/** Opt-in measurement only: adb setprop log.tag.aw-compose DEBUG. No text or action data. */
internal fun traceComposeFrame(view: View, touch: Long) {
    if (!Log.isLoggable("aw-compose", Log.DEBUG)) return
    view.viewTreeObserver.registerFrameCommitCallback {
        Log.d("aw-compose", "frame touch_ns=$touch post_ns=${System.nanoTime()}")
    }
}
