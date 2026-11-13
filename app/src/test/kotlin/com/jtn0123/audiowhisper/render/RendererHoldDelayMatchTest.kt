package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import kotlinx.serialization.json.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Frozen InputEventDispatcher.determineLongPressDelay; never called by production. */
private fun referenceHoldDelay(code: Int, delay: Long): Long {
    val factor = when (code) {
        KeyCode.SPACE, KeyCode.CJK_SPACE, KeyCode.SHIFT -> 2.5f
        KeyCode.LANGUAGE_SWITCH -> 2.0f
        else -> 1.0f
    }
    return (delay * factor).toLong()
}

class RendererHoldDelayMatchTest {
    @Test fun semanticHoldClocksMatchComposeIncluding400msShiftRegression() {
        val codes = listOf(KeyCode.SHIFT, KeyCode.SPACE, KeyCode.CJK_SPACE, KeyCode.LANGUAGE_SWITCH,
            KeyCode.DELETE, KeyCode.ENTER, 101)
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",
                emptyArray(), intArrayOf(), floatArrayOf(1f, 1f, 1f))
            for (base in listOf(1L, 100L, 299L, 300L, 301L, 500L, 1000L)) {
                r.barSettings(base * 1_000_000, intArrayOf())
                for (code in codes) {
                    r.update(floatArrayOf(200f, 200f, 40f), arrayOf("characters", "unshifted", "modifier"),
                        intArrayOf(), floatArrayOf(0f, 0f, 100f, 100f, 0f, 0f, 100f, 100f, 1f, code.toFloat()),
                        arrayOf("", "", ""), emptyArray())
                    val node = Json.parseToJsonElement(r.semanticSnapshot()).jsonObject.getValue("nodes").jsonArray[0].jsonObject
                    val actual = node.getValue("holdDelayMillis").jsonPrimitive.long
                    assertEquals(referenceHoldDelay(code, base), actual, "code=$code base=$base")
                    if (code == KeyCode.SHIFT && base == 300L) assertTrue(400 < actual)
                }
            }
        }
    }
}
