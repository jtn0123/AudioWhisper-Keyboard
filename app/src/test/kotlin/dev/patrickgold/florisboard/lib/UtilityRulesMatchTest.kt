package dev.patrickgold.florisboard.lib

import dev.patrickgold.florisboard.lib.util.VersionName
import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.WireFlag
import org.florisboard.lib.kotlin.UtilityOp
import org.florisboard.lib.kotlin.UtilityRules
import kotlin.random.Random
import kotlin.test.Test

class UtilityRulesMatchTest {
    private fun outcome(f: () -> Any?): Any? = try { f() } catch (e: Exception) { e.javaClass.name to e.message }
    @Test fun urlShapesAndSeededReferencePlans() {
        val values = mutableListOf("", "assets:", "assets:a", "assets:/a", "internal://a?x#y", "assets:a%20b", "INTERNAL:a", "example.com", "http://a", "HTTPS://a", "mailto:a", "//a/b", "\ud800", "\u00a0")
        // The shapes of reference and URL a source file holds, made up: the files' own URLs would
        // tie the golden to every licence header and link in src/main.
        values += URL_SHAPES
        val tape = GoldenTape("utility-references")
        val random = Random(181919)
        repeat(10_000) { values += listOf("assets:", "internal:", "https://", "", "mailto:").random(random) +
            String(CharArray(random.nextInt(20)) { random.nextInt(65536).toChar() }) }
        for (value in values) {
            tape.answer("normalize", UtilityRules.call(UtilityOp.NORMALIZE_REFERENCE, value))
            tape.answer("url", UtilityRules.call(UtilityOp.URL, value)[0])
        }
        repeat(10_000) {
            val scheme = listOf("florisboard", "FLORISBOARD", "content", "", " ").random(random)
            val authority = listOf("app-ui", "assets", "cache", "internal", "", "\u00a0", "other").random(random)
            val path = values.random(random)
            tape.answer("facts", UtilityRules.call(UtilityOp.REFERENCE_FACTS, scheme, authority, path))
        }
        tape.finish()
    }
    @Test fun fixedAndSeededTagsAndDisplayLayout() {
        val random = Random(181919)
        val seeded = listOf("en-US-posix-extra", "-", "en_", "__", "a_b_c_d", "", "en--x") +
            List(10_000) { List(random.nextInt(1, 6)) { listOf("en", "US", "variant", " ", "", "\ud800").random(random) }.joinToString(listOf("-", "_").random(random)) }
        // The golden holds the fixed and seeded tags, never the JDK's installed locales or the names
        // it gives real ones, which a JDK update may change. Seeded tags are named only in English
        // and United States, or by their own codes.
        val tape = GoldenTape("utility-locales")
        for (tag in REAL_TAGS + seeded) {
            val locale = FlorisLocale.fromTag(tag)
            tape.answer("parts", listOf(locale.language, locale.country, locale.variant))
            tape.answer("tags", listOf(locale.languageTag(), locale.localeTag()))
            tape.answer("capitalizes", locale.supportsCapitalization)
            if (tag !in REAL_TAGS) tape.answer("name", locale.displayName(FlorisLocale.ENGLISH))
        }
        tape.finish()
    }
    @Test fun versionParsingComparisonFormattingAndChangelog() {
        val random = Random(191818)
        val versions = listOf("0.0.0", "1.2.3", "001.02.03", "1.2.3-4", "1.2.3-alpha", "1.2.3-alpha1", "2147483648.1.1", "1.2.2147483648", "", "undefined", "1.2.3.4", "+1.2.3") +
            List(10_000) { "${random.nextInt(50)}.${random.nextInt(50)}.${random.nextInt(50)}" + listOf("", "-1", "-alpha2", ".beta").random(random) }
        val tape = GoldenTape("utility-versions")
        for (raw in versions) tape.answer("parsed", outcome { VersionName.fromString(raw)?.toString() })
        repeat(20_000) {
            val numbers = List(6) { random.nextInt() }
            val name = listOf(null, "", "alpha", "β").random(random)
            val value = listOf(null, 0, 1, -1, Int.MAX_VALUE).random(random)
            val a = VersionName(numbers[0], numbers[1], numbers[2], name, value)
            val b = VersionName(numbers[3], numbers[4], numbers[5], name, value)
            tape.answer("text", a.toString())
            tape.answer("order", a.compareTo(b))
            tape.answer("order without extra", a.compareTo(a.copy(extraName = null, extraValue = null)))
        }
        for (install in listOf("0.0.0", "1.1.0", "2.0.0", "undefined")) for (last in listOf("0.0.0", "1.1.0", "2.0.0")) {
            val current = "2.0.0"
            tape.answer("changelog $install $last", WireFlag.decode(UtilityRules.call(UtilityOp.SHOWS_CHANGELOG, install, last, current)[0]))
        }
        tape.finish()
    }
    @Test fun dictionaryValidationAllUtf16AndSeededValues() {
        val random = Random(1819)
        val values = (0..65535).map { it.toChar().toString() } + listOf("1", "255", "256", "-1", "2147483648", "١٢", " a b ", " a,b ", "a;b") +
            List(10_000) { String(CharArray(random.nextInt(16)) { random.nextInt(65536).toChar() }) }
        val tape = GoldenTape("utility-validation")
        for (value in values) for (rule in listOf("word", "freq", "shortcut")) {
            tape.record(UtilityRules.call(UtilityOp.VALIDATE, rule, value, "1", "255")[0])
        }
        tape.finish()
    }

    private companion object {
        val URL_SHAPES = listOf(
            "https://example.org/a/b?c=d#e", "http://example.org:8080/", "https://user@example.org", "https://[::1]/",
            "https://EXAMPLE.org/A%20B", "https://example.org/", "https://", "http://", "https:/a", "https:a", "ftp://a",
            "file:///a", "content://a/b", "florisboard://app-ui/settings", "mailto:", "MAILTO:a", "mailto:a@example.org",
            "assets:ime/keyboard/org.florisboard.layouts/extension.json", "assets:/", "assets:a/b.json", "internal:/a/b",
            "internal://a", "internal:", "Assets:a", "assets:\ud800", "internal:a\u00a0b",
        )

        /** Real tags, the capitalization rule's languages and the old codes among them. */
        val REAL_TAGS = listOf(
            "en", "en_US", "de_DE", "fr_CA", "pt-BR", "es_419", "zh", "zh_TW", "ko", "th", "bn", "hi", "ja_JP", "jp",
            "iw", "he", "in", "id", "ji", "yi", "sr_RS", "nb_NO", "no_NO_NY", "ar_EG", "fa", "ur_PK", "ZH", "Ko_kr",
        )
    }
}
