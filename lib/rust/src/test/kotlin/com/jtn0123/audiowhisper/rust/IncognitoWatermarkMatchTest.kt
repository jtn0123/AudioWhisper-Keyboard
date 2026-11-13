package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals

/** TextInputLayout's showIncognitoIcon rule. */
class IncognitoWatermarkMatchTest {
    @Test fun activeIncognitoAndBehindKeyboardDisplayAreBothRequired() {
        for (incognito in 0..1) for (behind in 0..1) {
            assertEquals(incognito != 0 && behind != 0,ImeChromeRules.watermark(intArrayOf(incognito,behind)))
        }
    }
}
