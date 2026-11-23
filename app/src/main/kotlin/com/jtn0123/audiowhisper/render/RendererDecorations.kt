package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode

/** Whether incognito is on and shown behind the keys; Rust draws the watermark. */
internal fun rendererWatermark(r: RootReads): IntArray = intArrayOf(
    if (r.keyboard.activeEvaluator.value.state.isIncognitoMode) 1 else 0,
    if (r.prefs.keyboard.incognitoDisplayMode.get() == IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD) 1 else 0)
