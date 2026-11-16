package com.jtn0123.audiowhisper.rust

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The hit test leans on the letter odds the suggester last worked out, through the calls the app
 * makes: [LatinSession.suggest] works them out, [TapPositions.retarget] leans on them while they
 * hold for the word being typed.
 */
class KeyResizeTest {
    private val model = File("../../app/src/main/assets/dicts/main_en-US.lm")
    private val personal = PersonalWordStore(emptyList(), IntArray(0), emptyList())

    /** The key a tap at [x] types on e and r side by side, each 100 wide and 150 high. */
    private fun TapPositions.at(x: Float): Int {
        val hit = if (x < 100f) 0 else 1
        val layout = intArrayOf(hit, 1, 2, 'e'.code, 'r'.code)
        val bounds = floatArrayOf(0f, 0f, 100f, 150f, 100f, 150f, 100f, 0f, 200f, 150f, 100f, 150f)
        return retarget(layout, bounds, x, 75f, false)
    }

    private fun LatinSession.typing(typed: String) {
        suggest(personal, null, typed, listOf("think", "I"), false, false, true, FloatArray(0), 3,
            SuggestRules.modestThreshold)
    }

    /** A trail that typed "th". */
    private fun th() = TapPositions().apply {
        for (letter in "th") {
            pressed(letter.code, false, 0f, 0f)
            typed(letter.toString())
        }
    }

    @Test fun aTapNearTheBorderTypesTheLetterLikelierToComeNext() {
        requireNotNull(LatinSession.open(TestDictionaries.shipped.readBytes())).use { session ->
            val trail = th()
            // Without the model there are no odds: 3 px inside r is r.
            session.typing("th")
            assertEquals(1, trail.at(103f))
            check(session.attachModel(model.path, 0, model.length()))
            session.typing("th")
            // After "I think th" an e is far likelier than an r: the border moves into r.
            assertEquals(0, trail.at(103f))
            assertEquals(1, trail.at(120f), "well inside r it stays r")
            assertEquals(0, trail.at(97f))
            // Once the next key is down the odds are for a word no longer being typed.
            trail.pressed('e'.code, false, 0f, 0f)
            assertEquals(1, trail.at(103f))
        }
    }

    @Test fun theRendererHitTestLeansOnThemToo() {
        requireNotNull(LatinSession.open(TestDictionaries.shipped.readBytes())).use { session ->
            check(session.attachModel(model.path, 0, model.length()))
            session.typing("th")
            val trail = th()
            KeyboardRenderer(emptyArray()).use { renderer ->
                renderer.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key":{"font-size":"20sp"}}""",
                    emptyArray(), intArrayOf(), floatArrayOf(1f, 1f, 1f))
                // e and r, 50 wide, under a 20-high strip.
                renderer.update(floatArrayOf(100f, 80f, 20f),
                    arrayOf("characters", "unshifted", "character", "character"), intArrayOf(),
                    floatArrayOf(0f, 0f, 50f, 60f, 0f, 0f, 50f, 60f, 1f, 101f,
                        50f, 0f, 100f, 60f, 50f, 0f, 100f, 60f, 1f, 114f),
                    arrayOf("e", "", "", "r", "", ""), emptyArray())
                // The key going down 2 px inside r, and its release.
                fun down(trail: TapPositions?): List<Int> {
                    val key = renderer.touch(intArrayOf(0, 0, 7), longArrayOf(1, 2), floatArrayOf(52f, 40f), trail)
                    renderer.touch(intArrayOf(1, 0, 7), longArrayOf(1, 3), floatArrayOf(52f, 40f), trail)
                    return key.asList()
                }
                assertEquals(listOf(RenderProtocol.KEY_DOWN, 1, 7), down(null))
                assertEquals(listOf(RenderProtocol.KEY_DOWN, 0, 7), down(trail))
            }
        }
    }
}
