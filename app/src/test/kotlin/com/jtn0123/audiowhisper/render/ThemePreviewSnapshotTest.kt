package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.ime.theme.ThemeManager
import org.florisboard.lib.snygg.SnyggStylesheet
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ThemePreviewSnapshotTest {
    @Test fun copyingTheEditorDraftUpdatesTheActualNativeJsonAndPixels() {
        val original=ThemeManager.ThemeInfo.DEFAULT
        val old=original.stylesheetJson
        var preview=original
        for (color in listOf("#ff0000", "#0000ff")) {
            val stylesheet=SnyggStylesheet.fromJson("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key":{"background":"$color"}}""").getOrThrow()
            preview=preview.copy(stylesheet=stylesheet)
            assertEquals(stylesheet.toJson().getOrThrow(),preview.stylesheetJson)
            KeyboardRenderer(emptyArray()).use { renderer ->
                renderer.theme(preview.stylesheetJson,emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
                val pixels=renderer.keyPreview(97,arrayOf("character","",""))
                val wanted=if(color=="#ff0000") 0xffff0000.toInt() else 0xff0000ff.toInt()
                assertTrue(pixels.drop(2).any {it==wanted})
            }
        }
        assertEquals(old,original.stylesheetJson)
    }
}
