package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SmartbarRules
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class RendererSearchTransportTest {
    @Test fun searchRowKeepsTheEvaluatedActionCohortAndOverridesSmartbarVisibility() {
        for(enabled in 0..1) for(layout in 0..3) for(expanded in 0..1) for(dynamic in 0..8) {
            val original=floatArrayOf(enabled.toFloat(),layout.toFloat(),1f,expanded.toFloat(),expanded.toFloat(),2f,1f,1f,dynamic.toFloat(),1f,1f,48f,1f,1f)
            val config=gifSearchBarConfig(original,48f)
            val measurement=SmartbarRules.layout(config,400f,floatArrayOf())
            assertEquals(48f,measurement[0]);assertEquals(48f,measurement[1]);assertEquals(dynamic.toFloat(),config[8])
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
