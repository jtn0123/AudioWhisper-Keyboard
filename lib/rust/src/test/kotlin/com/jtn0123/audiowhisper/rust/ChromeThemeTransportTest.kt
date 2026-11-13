package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals

class ChromeThemeTransportTest {
    @Test fun activeModeAndShiftReachNativeSideAndExtractThemeQueries() {
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","one-handed-panel":{"background":"#ff0000"},"one-handed-panel[mode=`symbols`][shiftstate=`shifted_manual`]":{"background":"#0000ff"}}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            for ((mode,shift,wanted) in listOf(Triple("characters","unshifted",0xffff0000.toInt()),Triple("symbols","shifted_manual",0xff0000ff.toInt()))) {
                r.chrome(floatArrayOf(80f,300f,1f,0f),arrayOf("Close","Move"),emptyArray(),arrayOf(mode,shift))
                assertEquals(wanted,r.pixels()[0])
            }
        }
    }
}
