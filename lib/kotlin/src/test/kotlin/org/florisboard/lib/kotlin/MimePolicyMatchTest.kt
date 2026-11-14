package org.florisboard.lib.kotlin

import java.nio.file.Files
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class MimePolicyMatchTest {
    private fun outcome(f: () -> Any?): Any? = try { f() } catch (e: Exception) { e.javaClass.name to e.message }
    @Test fun constructorRetainsCompiledSnapshotWhileTypesKeepsOriginalList() {
        val input = mutableListOf("image/png")
        val reference = ReferenceMimeTypeFilter(input)
        val actual = MimeTypeFilter(input)
        input[0] = "image/jpeg"
        assertEquals(reference.types, actual.types)
        assertEquals(reference.matches("image/png"), actual.matches("image/png"))
        assertEquals(reference.matches("image/jpeg"), actual.matches("image/jpeg"))
    }
    @Test fun realFiltersSeededValuesCollectionModesAndErrors() {
        val filters = mutableSetOf<List<String>>()
        // All literal filters used by the app and the font registry.
        Files.walk(java.nio.file.Path.of("../../app/src/main/kotlin")).use { paths ->
            paths.filter { it.toString().endsWith(".kt") }.forEach { path ->
                Regex("\"([A-Za-z*][A-Za-z0-9.*+-]*/[A-Za-z0-9.*+-]+)\"")
                    .findAll(Files.readString(path)).forEach { filters += listOf(it.groupValues[1]) }
            }
        }
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
        for (types in filters) {
            val expected = outcome { ReferenceMimeTypeFilter(types); "ok" }
            assertEquals(expected, outcome { MimeTypeFilter(types); "ok" })
            if (expected != "ok") continue
            val reference = ReferenceMimeTypeFilter(types)
            val actual = MimeTypeFilter(types)
            for (value in values) assertEquals(reference.matches(value), actual.matches(value))
            for (cohort in listOf<List<String?>?>(null, emptyList(), values, listOf(null)) + List(200) {
                List(random.nextInt(8)) { values.random(random) }
            }) {
                assertEquals(reference.matchesAll(cohort), actual.matchesAll(cohort))
                assertEquals(reference.matchesAny(cohort), actual.matchesAny(cohort))
                assertEquals(reference.matchesOne(cohort), actual.matchesOne(cohort))
            }
        }
    }
}
