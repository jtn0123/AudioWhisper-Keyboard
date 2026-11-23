package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SubtypeRendererTest {
    @Test fun modalPixelsResizeAndSelectedSemanticActionUseTheWindowViewport() {
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","subtype-panel":{"background":"#ffffff"},"subtype-panel-header":{"padding":"12dp","font-size":"18sp"},"subtype-panel-list-item":{"padding":"16dp"},"subtype-panel-list-item-icon-leading":{"font-size":"24sp"}}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            for ((width,height) in listOf(400 to 300,600 to 200,320 to 480)) {
                r.update(floatArrayOf(width.toFloat(),height.toFloat(),0f,1f,1f,0f),arrayOf("characters","unshifted"),intArrayOf(),floatArrayOf(),emptyArray(),emptyArray())
                r.panels(floatArrayOf(7f,1f,48f,0f,50_000_000f,0f,500_000_000f),intArrayOf(-1,0,0),Array(6) {""})
                r.subtypes(Long.MAX_VALUE,longArrayOf(-1,Long.MAX_VALUE),arrayOf("Select subtype","English (United States)","English (United Kingdom)"))
                val nodes=r.semanticSnapshot()
                assertTrue(nodes.contains("\"popup\":true"))
                assertTrue(nodes.contains("English (United Kingdom)"))
                val pixels=r.pixels()
                assertEquals(width*height,pixels.size)
                assertEquals(82,pixels[0] ushr 24)
                assertContentEquals(intArrayOf(RenderProtocol.PANEL_SUBTYPE,1,1),r.semanticAction(2_700_001,RenderProtocol.ACCESS_CLICK))
                assertContentEquals(intArrayOf(RenderProtocol.PANEL_SUBTYPE,0,0),r.semanticAction(2_700_001,RenderProtocol.ACCESS_DISMISS))
            }
        }
    }
}
