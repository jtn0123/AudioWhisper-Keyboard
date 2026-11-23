package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode

/** The incognito watermark's icon, and whether incognito is on and shown behind the keys. */
internal fun rendererWatermark(r: RootReads): Pair<String, IntArray> = iconPath(r.incognito) to intArrayOf(
    if (r.keyboard.activeEvaluator.value.state.isIncognitoMode) 1 else 0,
    if (r.prefs.keyboard.incognitoDisplayMode.get() == IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD) 1 else 0)
