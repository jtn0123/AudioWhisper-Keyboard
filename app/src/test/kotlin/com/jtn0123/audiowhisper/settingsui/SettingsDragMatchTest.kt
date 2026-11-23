package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import kotlinx.serialization.json.*
import kotlin.math.abs
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Rust's settings drag and fling against Compose's tracker and decay (ReferenceSettingsDrag). */
class SettingsDragMatchTest {
    private sealed interface Step
    private data class Down(val ms: Long, val y: Float) : Step
    private data class Move(val samples: List<Pair<Long, Float>>, val x: Float = 20f) : Step
    private data class Up(val ms: Long) : Step

    private fun frames(start: Long, y: Float, step: Long, deltas: List<Float>): List<Step> {
        var at = y
        return deltas.mapIndexed { i, d -> at += d; Move(listOf(start + step * (i + 1) to at)) }
    }

    private val gestures: Map<String, List<Step>> = mapOf(
        "fast flick at 120 Hz" to listOf(Down(1000, 1800f)) + frames(1000, 1800f, 8, List(8) { -70f }) + Up(1068),
        "slow drag, then a 60 ms rest" to listOf(Down(3000, 1600f)) + frames(3000, 1600f, 16, List(10) { -20f }) + Up(3220),
        "downward flick slowing" to listOf(Down(5000, 600f)) +
            frames(5000, 600f, 8, listOf(90f, 70f, 50f, 30f, 15f)) + Up(5042),
        // History batched three to an event; the first event crosses the slop on its middle sample.
        "batched history" to listOf(
            Down(7000, 1500f),
            Move(listOf(7004L to 1490f, 7008L to 1470f, 7012L to 1430f)),
            Move(listOf(7016L to 1380f, 7020L to 1320f, 7024L to 1250f)),
            Move(listOf(7028L to 1170f, 7032L to 1080f, 7036L to 980f)),
            Up(7038),
        ),
        "up exactly 40 ms after the last move" to listOf(Down(9000, 1700f)) +
            frames(9000, 1700f, 10, List(6) { -40f }) + Up(9100),
        "up 41 ms after the last move" to listOf(Down(11000, 900f)) +
            frames(11000, 900f, 10, List(6) { 40f }) + Up(11101),
        "samples sharing a millisecond" to listOf(
            Down(13000, 1600f),
            Move(listOf(13008L to 1560f, 13008L to 1540f)),
            Move(listOf(13016L to 1480f)),
            Move(listOf(13016L to 1450f, 13024L to 1400f)),
            Up(13026),
        ),
        "reversal" to listOf(Down(15000, 1200f)) + frames(15000, 1200f, 8, listOf(-60f, -60f, -30f, 20f, 60f, 80f)) + Up(15050),
        "long drag past the 20-sample ring" to listOf(Down(17000, 1900f)) + frames(17000, 1900f, 4, List(30) { -12f - it }) + Up(17122),
        // Unmoved events never reach the tracker, so the up is 58 ms after the last change.
        "hold before lifting" to listOf(Down(19000, 1500f)) + frames(19000, 1500f, 8, List(6) { -60f }) +
            (1..5).map { Move(listOf(19048L + 10 * it to 1140f)) } + Up(19106),
        // A sideways move with no vertical change still counts: Compose tracks it.
        "sideways wiggle while holding" to listOf(Down(21000, 1500f)) + frames(21000, 1500f, 8, List(6) { -60f }) +
            (1..5).map { Move(listOf(21048L + 10 * it to 1140f), if (it % 2 == 1) 24f else 20f) } + Up(21106),
    )

    @Test fun dragsAndFlingsLandWhereComposeScrollStateDoes() {
        KeyboardRenderer(emptyArray()).use { native ->
            fun state(reply: Array<String>) = Json.parseToJsonElement(reply[0]).jsonObject
            fun call(op: SettingsUiOp, vararg args: String) = state(native.settings(op, arrayOf(*args)))
            val density = 2.625f
            call(SettingsUiOp.CONFIG, "1080", "2340", "$density", "1", "4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814", "true", "true")
            var now = call(SettingsUiOp.NAVIGATE, "settings/keyboard")
            val max = now.getValue("maxScroll").jsonPrimitive.float
            assertTrue(max > 2000f, "the keyboard screen scrolls: $max")
            // A varying sub-millisecond phase: MotionEvent nanoseconds truncate to Compose's millis.
            fun ns(ms: Long) = ms * 1_000_000 + ms % 7 * 130_000 + 1_123
            fun touch(action: Int, samples: List<Pair<Long, Float>>, downMs: Long, x: Float = 20f) = state(native.settingsTouch(
                intArrayOf(action, 0, 0),
                longArrayOf(ns(downMs)) + samples.map { ns(it.first) },
                samples.flatMap { listOf(x, it.second) }.toFloatArray(),
            ))
            val tape = GoldenTape.ofReference("settings-drag")
            var flings = 0
            var moved = 0
            for ((name, steps) in gestures) {
                // The scroller's viewport: the screen less the 64 dp app bar and 56 dp preview field.
                val extent = 2340 - Math.round(64 * density) - Math.round(56 * density).toFloat()
                val reference = ReferenceSettingsDrag(density, now.getValue("scroll").jsonPrimitive.float, max, extent)
                val downMs = (steps.first() as Down).ms
                var lastX = 20f
                var lastY = 0f
                for (step in steps) when (step) {
                    is Down -> { reference.down(step.ms, 20f, step.y); lastX = 20f; lastY = step.y; now = touch(0, listOf(step.ms to step.y), downMs) }
                    is Move -> {
                        reference.move(step.samples, step.x); lastX = step.x; lastY = step.samples.last().second
                        now = touch(2, step.samples, downMs, step.x)
                        assertEquals(reference.position, now.getValue("scroll").jsonPrimitive.float, "$name drag")
                        tape.answer("$name drag", reference.position)
                    }
                    is Up -> {
                        val before = reference.position
                        val velocity = reference.up(step.ms)
                        if (reference.position != before) moved++
                        if (velocity != null && abs(velocity) > 1f) flings++
                        touch(1, listOf(step.ms to lastY), downMs, lastX)
                        call(SettingsUiOp.FRAME, ns(step.ms + 1).toString())
                        now = call(SettingsUiOp.FRAME, ns(step.ms + 30_000).toString())
                        val rust = now.getValue("scroll").jsonPrimitive.float
                        assertEquals(reference.position, rust, .01f, "$name fling at $velocity px/s")
                        assertEquals(reference.value, Math.round(rust), "$name shown offset")
                        tape.answer("$name fling", reference.position)
                        tape.answer("$name shown", reference.value)
                    }
                }
            }
            // The 60 ms rest, the 41 ms pause and the hold stop; every other gesture flings.
            assertEquals(gestures.size - 3, flings)
            // Six move the content: "batched history" and "up exactly 40 ms" fling forward from the end.
            assertEquals(6, moved)
            tape.finish()
        }
    }
}
