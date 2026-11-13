package com.jtn0123.audiowhisper.render

import androidx.test.platform.app.InstrumentationRegistry
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.gif.SearchText
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** The long-lived external fixture controller is Java instrumentation; this is an ordinary test. */
class RendererFixtureTest {
    @Test fun searchStateWorksWithoutAKey() {
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
            val original=GifSearch.source
            try {
                GifSearch.source=null
                assertFalse(GifSearch.available)
                GifSearch.startTyping()
                assertTrue(GifSearch.typing.value)
                val before=GifSearch.query.value
                GifSearch.type("aw-fixture")
                assertEquals(SearchText.type(before,"aw-fixture"),GifSearch.query.value)
                GifSearch.stopTyping()
                assertFalse(GifSearch.typing.value)
            }finally{GifSearch.reset();GifSearch.source=original}
        }
    }
}
