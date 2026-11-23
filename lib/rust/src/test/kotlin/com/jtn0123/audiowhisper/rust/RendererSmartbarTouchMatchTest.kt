package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test

/**
 * Pointer schedules on the Smartbar as BarTouch and RendererSmartbarTouch answered them (7c0b8183):
 * the actions each event, tick and invalidation gives and the deadline after it, a golden digest of
 * their frozen copy's answers (docs/phase6-kotlin-references.md).
 */
class RendererSmartbarTouchMatchTest {
    /** A slot of the layout packet: an action's or a word's, its index and its horizontal span. */
    private data class Slot(val index: Int, val left: Float, val right: Float)

    @Test fun recordedAndSeededPointerSchedules() {
        val random = Random(0x3b0301)
        val tape = GoldenTape("smartbar-touch")
        for (layout in listOf(0,1,2,3)) KeyboardRenderer(emptyArray()).use { renderer ->
            val config = floatArrayOf(1f,layout.toFloat(),0f,1f,1f,0f,0f,0f,5f,0f,1f,40f)
            renderer.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",
                emptyArray(), intArrayOf(), floatArrayOf(1f,1f,1f))
            renderer.smartbar(config, IntArray(20) { if (it % 2 == 1) 1 else -1 }, Array(20) { "" })
            renderer.update(floatArrayOf(400f,200f,if (layout == 3) 80f else 40f), arrayOf("characters","unshifted"),
                IntArray(3), floatArrayOf(), emptyArray(), arrayOf("a","b","c"))
            // Use the independently matched layout packets for the stationary/leave schedules.
            val packet = SmartbarRules.layout(config,400f,floatArrayOf(10f,10f,10f))
            val slots = (8 until packet.size step 6).filter { packet[it+3] == 0f }.map {
                Slot(packet[it+1].toInt(),packet[it+2],packet[it+4])
            }
            for (run in 0 until 800) {
                renderer.touch(intArrayOf(3,0),longArrayOf(),floatArrayOf())
                val delay = listOf(100L,300L,750L)[run%3]*1_000_000
                renderer.barSettings(delay,intArrayOf())
                var time = 1_000_000_000L
                val ids = intArrayOf(0,7)
                val xy = FloatArray(4)
                val first = slots[random.nextInt(slots.size)]
                xy[0]=(first.left+first.right)/2; xy[1]=20f
                fun event(action: Int, index: Int = 0, count: Int = 1) {
                    val actual = renderer.touch(intArrayOf(action,index)+ids.copyOf(count),longArrayOf(0,time),xy.copyOf(count*2))
                    tape.answer("event $action", actual.toList())
                    tape.answer("deadline", renderer.nextDeadline())
                }
                fun tick(delta: Long) {
                    time += delta
                    tape.answer("tick", renderer.tick(time).toList())
                    tape.answer("deadline", renderer.nextDeadline())
                }
                event(0)
                if (run%4 == 0) {
                    val second = slots[random.nextInt(slots.size)]
                    xy[2]=(second.left+second.right)/2; xy[3]=20f
                    event(5,1,2); tick(delay-1); tick(1); event(6,1,2)
                } else {
                    tick(listOf(0,delay-1,delay,delay+1)[run%4])
                }
                if (run%5 == 0) {
                    val changed = intArrayOf(first.index)
                    tape.answer("invalidated", renderer.barSettings(delay,changed).toList())
                }
                if (run%3 == 0) { xy[0]=-1f; event(2) }
                event(if (run%7 == 0) 3 else 1)
                tick(delay*2)
            }
        }
        tape.finish()
    }
}
