package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import kotlinx.serialization.json.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SettingsScrollbarMatchTest {
    @Test fun initialHoldAndPostScrollFadeMatchTheRealComposeEasing() {
        KeyboardRenderer(emptyArray()).use { native ->
            fun call(op: Int, vararg args: String) = Json.parseToJsonElement(native.settings(op, arrayOf(*args))[0]).jsonObject
            call(3, "480", "640", "1", "1", "4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814", "true", "true")
            call(4, "settings/keyboard")
            val origin = 1_000_000_000L
            for (elapsed in 0..4000 step 10) {
                val state = call(8, (origin+elapsed*1_000_000L).toString())
                assertEquals(ReferenceSettingsScrollbar.alpha(elapsed, true), state.getValue("scrollbarAlpha").jsonPrimitive.float, .003f)
            }
            assertTrue(call(17, "4", "4").getValue("scroll").jsonPrimitive.float > 0)
            for (elapsed in 0..1200 step 10) {
                val state = call(8, (origin+4_000_000_000L+elapsed*1_000_000L).toString())
                assertEquals(ReferenceSettingsScrollbar.alpha(elapsed, false), state.getValue("scrollbarAlpha").jsonPrimitive.float, .003f)
            }
            // Idle frames stop: a new screen starts its initial hold on the NEXT frame.
            call(4, "settings/home"); call(4, "settings/keyboard")
            assertEquals(1f, call(8, (origin+100_000_000_000L).toString()).getValue("scrollbarAlpha").jsonPrimitive.float)
        }
    }
}
