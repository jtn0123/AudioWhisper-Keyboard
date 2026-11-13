package org.florisboard.lib.snygg

import org.junit.jupiter.api.Test
import kotlin.random.Random
import kotlin.test.assertEquals

class SnyggGrammarMatchTest {
    @Test
    fun `schema generator grammar patterns stay identical`() {
        assertEquals(ReferenceSnyggAnnotationRule.Defines.pattern.pattern, SnyggAnnotationRule.Defines.pattern.pattern)
        assertEquals(ReferenceSnyggAnnotationRule.Font.pattern.pattern, SnyggAnnotationRule.Font.pattern.pattern)
        assertEquals(ReferenceSnyggAttributes.ATTRIBUTE_REGEX.pattern, SnyggAttributes.ATTRIBUTE_REGEX.pattern)
        assertEquals(ReferenceSnyggElementRule.pattern.pattern, SnyggElementRule.pattern.pattern)
        assertEquals(ReferenceSnyggStylesheet.SCHEMA_V2, SnyggStylesheet.SCHEMA_V2)
        assertEquals(SnyggStylesheet.SCHEMA_V2, SnyggGrammarAdapter.metadata[4])
    }

    @Test
    fun `seeded rules selectors attributes constructors and exact comparison match`() {
        val random = Random(0x5A723)
        val names = listOf("key", "unknown-element", "0", "-", "A-Z", "bad_name", "", "é", "key\n")
        val values = listOf("0", "-0", "01", "+1", "-2", "2", "10", "1..4", "4..1", "2147483648", "2147483648..1",
            "-2147483649..0", "`unknown`", "`a,b[]\n`", "`\uD800`", "``", "`1`", "`٣`", "1.2", "-1..1")
        val selectors = listOf("", ":pressed", ":focus", ":hover", ":disabled", ":none", ":unknown", ":pressed:focus")
        val rules = mutableListOf("@defines", "@defines\n", "@font `A B-12`", "@font ``", "@font `a_b`", "@font `x`\n")
        repeat(12000) {
            val attrs = List(random.nextInt(5)) {
                "[${listOf("code", "unknown", "code", "a", "bad_key").random(random)}=${List(random.nextInt(1, 4)) { values.random(random) }.joinToString(",")}]"
            }.joinToString("")
            rules.add(names.random(random) + attrs + selectors.random(random))
        }
        // Full UTF-16 corpus proves backtick strings are lossless and the JVM font \s table matches.
        for (unit in Char.MIN_VALUE..Char.MAX_VALUE) {
            checkRule("@font `x${unit}y`")
            checkRule("key[attr=`${unit}`]")
        }
        rules.forEach { input ->
            checkRule(input)
            sameResult(input, runCatching { ReferenceSnyggAttributes.from(input) }, runCatching { SnyggAttributes.from(input) },
                { it.toMap() to it.toString() }, { it.toMap() to it.toString() })
            sameResult(input, runCatching { ReferenceSnyggAnnotationRule.Defines.fromOrNull(input) },
                runCatching { SnyggAnnotationRule.Defines.fromOrNull(input) }, { it?.toString() }, { it?.toString() })
            sameResult(input, runCatching { ReferenceSnyggAnnotationRule.Font.fromOrNull(input) },
                runCatching { SnyggAnnotationRule.Font.fromOrNull(input) }, { it?.toString() }, { it?.toString() })
            sameResult(input, runCatching { ReferenceSnyggElementRule.fromOrNull(input) },
                runCatching { SnyggElementRule.fromOrNull(input) }, { it?.snapshot() }, { it?.snapshot() })
        }
        val valid = rules.mapNotNull { runCatching { ReferenceSnyggRule.fromOrNull(it) }.getOrNull() }
        repeat(20000) {
            val a = valid.random(random)
            val b = valid.random(random)
            assertEquals(a.compareTo(b), a.production().compareTo(b.production()), "$a vs $b")
        }
        assertEquals(valid.sorted().map { it.snapshot() }, valid.map { it.production() }.sorted().map { it.snapshot() })
        (selectors + listOf("xpressed", "xnone", ":", "\uD800focus")).forEach { s ->
            sameResult(s, runCatching { ReferenceSnyggSelector.from(s) }, runCatching { SnyggSelector.from(s) },
                { it.ordinal to it.toString() }, { it.ordinal to it.toString() })
        }
        names.forEach { name ->
            sameResult(name, runCatching { ReferenceSnyggElementRule(name) }, runCatching { SnyggElementRule(name) },
                { it.toString() }, { it.toString() })
        }
    }

    @Test
    fun `seeded attribute edits and arbitrary constructor maps match`() {
        val random = Random(23723)
        val texts = listOf("", "1", "10", "2", "2147483647", "-2147483648", "+01", "٣", "\uD800", "a`,b", "x")
        repeat(4000) {
            val pairs = Array(random.nextInt(6)) { listOf("a", "b", "unknown", "", "\uDFFF").random(random) to
                List(random.nextInt(5)) { texts.random(random) as Any } }
            var a = ReferenceSnyggAttributes.of(*pairs)
            var b = SnyggAttributes.of(*pairs)
            repeat(10) {
                assertEquals(a.toMap(), b.toMap())
                assertEquals(a.toString(), b.toString())
                val other = ReferenceSnyggAttributes.of(*Array(3) { "a" to List(3) { texts.random(random) as Any } })
                assertEquals(a.compareTo(other), b.compareTo(SnyggAttributes(other.toMap())))
                val edits = Array(random.nextInt(5)) { texts.random(random) to (texts.random(random) as Any) }
                when (random.nextInt(3)) {
                    0 -> { a = a.including(*edits); b = b.including(*edits) }
                    1 -> { a = a.excluding(*edits); b = b.excluding(*edits) }
                    else -> { a = a.toggling(*edits); b = b.toggling(*edits) }
                }
            }
            assertEquals(a.toMap(), b.toMap())
            assertEquals(a.toString(), b.toString())
        }
    }
}
