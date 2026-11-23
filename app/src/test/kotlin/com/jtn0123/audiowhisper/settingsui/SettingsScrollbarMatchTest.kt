package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import kotlinx.serialization.json.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SettingsScrollbarMatchTest {
    /**
     * Alpha in ten-thousandths. Rust's cubic solve and Compose's CubicBezierEasing agree to 6.6e-7
     * but not bit for bit (48 of these 522 frames differ in the last place); rounded, every frame
     * agrees, 30 times finer than the .003 the comparison allows.
     */
    private fun alpha(value: Float) = Math.round(value * 10_000.0)

    @Test fun initialHoldAndPostScrollFadeMatchTheRealComposeEasing() {
        KeyboardRenderer(emptyArray()).use { native ->
            fun call(op: SettingsUiOp, vararg args: String) = Json.parseToJsonElement(native.settings(op, arrayOf(*args))[0]).jsonObject
            call(SettingsUiOp.CONFIG, "480", "640", "1", "1", "4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814", "true", "true")
            call(SettingsUiOp.NAVIGATE, "settings/keyboard")
            val tape = GoldenTape.ofReference("settings-scrollbar")
            val origin = 1_000_000_000L
            for (elapsed in 0..4000 step 10) {
                val state = call(SettingsUiOp.FRAME, (origin+elapsed*1_000_000L).toString())
                assertEquals(ReferenceSettingsScrollbar.alpha(elapsed, true), state.getValue("scrollbarAlpha").jsonPrimitive.float, .003f)
                tape.answer("initial $elapsed", alpha(ReferenceSettingsScrollbar.alpha(elapsed, true)))
            }
            assertTrue(call(SettingsUiOp.ACCESSIBILITY_SCROLL, "4", "4").getValue("scroll").jsonPrimitive.float > 0)
            for (elapsed in 0..1200 step 10) {
                val state = call(SettingsUiOp.FRAME, (origin+4_000_000_000L+elapsed*1_000_000L).toString())
                assertEquals(ReferenceSettingsScrollbar.alpha(elapsed, false), state.getValue("scrollbarAlpha").jsonPrimitive.float, .003f)
                tape.answer("after a scroll $elapsed", alpha(ReferenceSettingsScrollbar.alpha(elapsed, false)))
            }
            // Idle frames stop: a new screen starts its initial hold on the NEXT frame.
            call(SettingsUiOp.NAVIGATE, "settings/home"); call(SettingsUiOp.NAVIGATE, "settings/keyboard")
            assertEquals(1f, call(SettingsUiOp.FRAME, (origin+100_000_000_000L).toString()).getValue("scrollbarAlpha").jsonPrimitive.float)
            tape.finish()
        }
    }
}
