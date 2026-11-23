package org.florisboard.lib.kotlin

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * MIME filters as FlorisBoard's Kotlin matched them. What its frozen copy answered is a golden digest
 * now (docs/phase6-kotlin-references.md); the filter is Rust's.
 */
class MimePolicyMatchTest {
    private fun outcome(f: () -> Any?): Any? = try { f() } catch (e: Exception) { e.javaClass.name to e.message }
    @Test fun constructorRetainsCompiledSnapshotWhileTypesKeepsOriginalList() {
        val input = mutableListOf("image/png")
        val actual = MimeTypeFilter(input)
        input[0] = "image/jpeg"
        assertEquals(listOf("image/jpeg"), actual.types)
        assertTrue(actual.matches("image/png"))
        assertFalse(actual.matches("image/jpeg"))
    }
    @Test fun realFiltersSeededValuesCollectionModesAndErrors() {
        val filters = mutableSetOf<List<String>>()
        // The literal filters the app's sources held when the golden was written, listed rather than
        // read from app/src/main, which would tie the golden to every MIME type added there.
        for (filter in APP_FILTERS) filters += listOf(filter)
        filters += listOf("*/*", "font/*", "application/x-font-*", "application/vnd.ms-fontobject")
        filters += emptyList<String>()
        for (filter in listOf("image/", "/png", "a/b/c", "", "image/[", "a/(", "a/*")) filters += listOf(filter)
        for (filter in listOf("image/pn.", "image/(png|jpeg)", "a/[a-z]+", "a/foo?", "a/a{2}", "a/\\d+", "a/a+b", "a/x*-*", "a/\\Qx.y\\E", "a/\\p{L}+", "a/(?U)\\w+", "a/\ud800", "a/\ufffd")) filters += listOf(filter)
        val random = Random(1919)
        val values = listOf(null, "", "/", "image/png", "image/jpeg", "application/x-font-otf", "application/vndXms-fontobject", "a/aa", "a/123", "a/😀", "a/\ud800", "a/\u00a0", "a/\u2003", "a/x-", "a/x-y", " a/png", "image/png\n", "a/b/c", "a/\ufffd", "a/x\u0085", "a/x\u2028", "a/İ")
        repeat(100) {
            val a = listOf("a", "image", "application", "*", "x-*").random(random)
            val b = listOf("b", "png", "*", "x-*", "*-font-*", "x.y", "p(n|j)g").random(random)
            filters += listOf("$a/$b")
        }
        val tape = GoldenTape("mime-filters")
        for (types in filters) {
            val made = outcome { MimeTypeFilter(types); "ok" }
            tape.answer("filter $types", made)
            if (made != "ok") continue
            val actual = MimeTypeFilter(types)
            for (value in values) tape.answer("matches", actual.matches(value))
            for (cohort in listOf<List<String?>?>(null, emptyList(), values, listOf(null)) + List(200) {
                List(random.nextInt(8)) { values.random(random) }
            }) {
                tape.answer("all any one", listOf(actual.matchesAll(cohort), actual.matchesAny(cohort), actual.matchesOne(cohort)))
            }
        }
        tape.finish()
    }

    private companion object {
        val APP_FILTERS = listOf(
            "*/*", "application/octet-stream", "application/vnd.florisboard.extension+zip", "application/zip", "clips/images",
            "clips/videos", "image/*", "image/gif", "ime/keyboard", "ime/languagepack", "ime/theme", "text/plain", "video/*",
        )
    }
}
