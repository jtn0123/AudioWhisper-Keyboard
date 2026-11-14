package dev.patrickgold.florisboard.lib

import dev.patrickgold.florisboard.lib.reference.ReferenceFlorisLocale
import dev.patrickgold.florisboard.lib.reference.ReferenceUtilityRules as Reference
import dev.patrickgold.florisboard.lib.reference.ReferenceVersionName
import dev.patrickgold.florisboard.lib.util.VersionName
import org.florisboard.lib.kotlin.UtilityRules
import java.io.File
import java.util.Locale
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class UtilityRulesMatchTest {
    private fun outcome(f: () -> Any?): Any? = try { f() } catch (e: Exception) { e.javaClass.name to e.message }
    @Test fun realAndSeededReferencePlans() {
        val values = mutableListOf("", "assets:", "assets:a", "assets:/a", "internal://a?x#y", "assets:a%20b", "INTERNAL:a", "example.com", "http://a", "HTTPS://a", "mailto:a", "//a/b", "\ud800", "\u00a0")
        File("src/main").walkTopDown().filter { it.extension in listOf("kt", "json") }.forEach { file ->
            Regex("(?:assets:|internal:|https?://)[^\\\"\\s]+")
                .findAll(file.readText()).forEach { values += it.value }
        }
        val random = Random(181919)
        repeat(10_000) { values += listOf("assets:", "internal:", "https://", "", "mailto:").random(random) +
            String(CharArray(random.nextInt(20)) { random.nextInt(65536).toChar() }) }
        for (value in values) {
            assertEquals(Reference.normalize(value), UtilityRules.call(2, value))
            assertEquals(Reference.url(value), UtilityRules.call(3, value)[0])
        }
        repeat(10_000) {
            val scheme = listOf("florisboard", "FLORISBOARD", "content", "", " ").random(random)
            val authority = listOf("app-ui", "assets", "cache", "internal", "", "\u00a0", "other").random(random)
            val path = values.random(random)
            assertEquals(Reference.facts(scheme, authority, path), UtilityRules.call(4, scheme, authority, path))
        }
    }
    @Test fun installedLocalesSeededTagsAndDisplayLayout() {
        val random = Random(181919)
        val tags = Locale.getAvailableLocales().map { it.toString() } + listOf("en-US-posix-extra", "-", "en_", "__", "a_b_c_d", "", "en--x") +
            List(10_000) { List(random.nextInt(1, 6)) { listOf("en", "US", "variant", " ", "", "\ud800").random(random) }.joinToString(listOf("-", "_").random(random)) }
        for (tag in tags) {
            val reference = ReferenceFlorisLocale.fromTag(tag)
            val actual = FlorisLocale.fromTag(tag)
            assertEquals(reference.base, actual.base)
            assertEquals(reference.languageTag(), actual.languageTag())
            assertEquals(reference.localeTag(), actual.localeTag())
            assertEquals(reference.supportsCapitalization, actual.supportsCapitalization)
            assertEquals(reference.displayName(ReferenceFlorisLocale.ENGLISH), actual.displayName(FlorisLocale.ENGLISH))
        }
    }
    @Test fun versionParsingComparisonFormattingAndChangelog() {
        val random = Random(191818)
        val versions = listOf("0.0.0", "1.2.3", "001.02.03", "1.2.3-4", "1.2.3-alpha", "1.2.3-alpha1", "2147483648.1.1", "1.2.2147483648", "", "undefined", "1.2.3.4", "+1.2.3") +
            List(10_000) { "${random.nextInt(50)}.${random.nextInt(50)}.${random.nextInt(50)}" + listOf("", "-1", "-alpha2", ".beta").random(random) }
        for (raw in versions) assertEquals(outcome { ReferenceVersionName.fromString(raw)?.toString() }, outcome { VersionName.fromString(raw)?.toString() })
        repeat(20_000) {
            val numbers = List(6) { random.nextInt() }
            val name = listOf(null, "", "alpha", "β").random(random)
            val value = listOf(null, 0, 1, -1, Int.MAX_VALUE).random(random)
            val a = VersionName(numbers[0], numbers[1], numbers[2], name, value)
            val ar = ReferenceVersionName(numbers[0], numbers[1], numbers[2], name, value)
            val b = VersionName(numbers[3], numbers[4], numbers[5], name, value)
            val br = ReferenceVersionName(numbers[3], numbers[4], numbers[5], name, value)
            assertEquals(ar.toString(), a.toString())
            assertEquals(ar.compareTo(br), a.compareTo(b))
            assertEquals(ar.compareTo(ar.copy(extraName = null, extraValue = null)), a.compareTo(a.copy(extraName = null, extraValue = null)))
        }
        for (install in listOf("0.0.0", "1.1.0", "2.0.0", "undefined")) for (last in listOf("0.0.0", "1.1.0", "2.0.0")) {
            val current = "2.0.0"
            val i = ReferenceVersionName.fromString(install) ?: ReferenceVersionName.DEFAULT
            val l = ReferenceVersionName.fromString(last) ?: ReferenceVersionName.DEFAULT
            val c = ReferenceVersionName.fromString(current)!!
            assertEquals(l < c && i != c, UtilityRules.call(12, install, last, current)[0] == "1")
        }
    }
    @Test fun dictionaryValidationAllUtf16AndSeededValues() {
        val random = Random(1819)
        val values = (0..65535).map { it.toChar().toString() } + listOf("1", "255", "256", "-1", "2147483648", "١٢", " a b ", " a,b ", "a;b") +
            List(10_000) { String(CharArray(random.nextInt(16)) { random.nextInt(65536).toChar() }) }
        for (value in values) for (rule in listOf("word", "freq", "shortcut")) assertEquals(
            Reference.validate(rule, value, 1, 255), UtilityRules.call(14, rule, value, "1", "255")[0])
    }
}
