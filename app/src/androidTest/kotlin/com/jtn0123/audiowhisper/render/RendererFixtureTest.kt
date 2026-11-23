package com.jtn0123.audiowhisper.render

import android.app.Instrumentation
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.golden.GoldenTape
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue

/**
 * GIF search typing with no GIPHY key, through the production [GifSearch]. FixtureInstrumentation
 * runs it with `-e gifSearch true`, in a rendererFixture build: the debug build is shrunk, and only
 * a fixture build keeps what a test APK calls, so no JUnit runner can run it there.
 *
 * The query it types is recorded on the golden tape [TAPE] and checked against the answers of the
 * SearchText reference it replaced (docs/phase6-kotlin-references.md).
 */
object RendererFixtureTest {
    const val TAPE = "gif-search-typing"

    @JvmStatic fun searchStateWorksWithoutAKey(instrumentation: Instrumentation, goldens: GoldenTape.Store) {
        val tape = GoldenTape(TAPE, goldens)
        var failure: Throwable? = null
        instrumentation.runOnMainSync {
            val original=GifSearch.source
            try {
                GifSearch.source=null
                assertFalse(GifSearch.available)
                GifSearch.startTyping()
                assertTrue(GifSearch.typing.value)
                val before=GifSearch.query.value
                GifSearch.type("aw-fixture")
                tape.answer("before", before)
                tape.answer("typed", GifSearch.query.value)
                GifSearch.stopTyping()
                assertFalse(GifSearch.typing.value)
            } catch (thrown: Throwable) {
                failure = thrown // Thrown on the main thread, it would end the process instead of failing the run.
            } finally{GifSearch.reset();GifSearch.source=original}
        }
        failure?.let { throw it }
        tape.finish()
    }
}
