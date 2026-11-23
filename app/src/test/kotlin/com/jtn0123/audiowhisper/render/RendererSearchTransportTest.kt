package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.ImeWindowFacts as F
import com.jtn0123.audiowhisper.rust.ImeWindowPlan as P
import com.jtn0123.audiowhisper.rust.ImeWindowRules
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SmartbarRules
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class RendererSearchTransportTest {
    /** A 400 × 900 portrait window at density 1 typing a GIF search, then the Smartbar's [config]. */
    private fun plan(config: FloatArray): FloatArray {
        val facts = FloatArray(F.SIZE)
        for ((i, v) in listOf(F.DENSITY to 1f, F.SYSTEM_DENSITY to 1f, F.WIDTH to 400f, F.HEIGHT_MODE to 2f, F.HEIGHT to 900f,
            F.ONE_HANDED_SCALE to 87f, F.HEIGHT_PORTRAIT to 100f, F.HEIGHT_LANDSCAPE to 100f, F.DISPLAY_WIDTH to 400f,
            F.DISPLAY_HEIGHT to 900f, F.MIN_FRACTION to .4f, F.MAX_FRACTION to .46f, F.BASE_HEIGHT to 206f, F.ROWS to 4f,
            F.CHARACTER_ROWS to 4f, F.SEARCH to 1f)) facts[i] = v
        return ImeWindowRules.layout(facts + config)
    }

    /** The search row's config, as Rust's plan answers it. */
    private fun searchBar(config: FloatArray) = plan(config).let { it.copyOfRange(P.SIZE, it.size) }

    @Test fun searchRowKeepsTheEvaluatedActionCohortAndOverridesSmartbarVisibility() {
        for(enabled in 0..1) for(layout in 0..3) for(expanded in 0..1) for(dynamic in 0..8) {
            val original=floatArrayOf(enabled.toFloat(),layout.toFloat(),1f,expanded.toFloat(),expanded.toFloat(),2f,1f,1f,dynamic.toFloat(),1f,0f,0f,1f,1f)
            val config=searchBar(original+floatArrayOf(0f,0f,0f))
            val row=plan(original+floatArrayOf(0f,0f,0f))[P.SMARTBAR_ROW]
            val measurement=SmartbarRules.layout(config,400f,floatArrayOf())
            // One row of the plan's Smartbar height, in whole pixels, whatever the settings' layout.
            assertTrue(kotlin.math.abs(row-measurement[0])<1f);assertEquals(measurement[0],measurement[1]);assertEquals(dynamic.toFloat(),config[8])
            // Words, dictation and reading facts never reach the search row's own layout.
            val roomy=searchBar(original+floatArrayOf(1f,1f,1f))
            assertEquals(List(5){0f},roomy.drop(12));assertTrue(!SmartbarRules.roomy(roomy))
            assertEquals(measurement.toList(),SmartbarRules.layout(roomy,400f,floatArrayOf()).toList())
            val font=java.io.File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
            KeyboardRenderer(arrayOf(font)).use { native ->
                native.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
                native.smartbar(config,IntArray((5+dynamic)*2),Array((5+dynamic)*2){""})
                native.update(floatArrayOf(400f,248f,48f,1f,1f),arrayOf("characters","unshifted"),intArrayOf(),floatArrayOf(),emptyArray(),emptyArray())
                native.search(1f,arrayOf("","Back to GIFs","Search GIPHY","Search","",""),0)
                val snapshot=native.semanticSnapshot()
                assertTrue(snapshot.contains("Search GIPHY"));assertTrue(snapshot.contains("Back to GIFs"))
                native.search(1f,emptyArray(),0);assertEquals(-1,native.nextDeadline())
            }
        }
    }
}
