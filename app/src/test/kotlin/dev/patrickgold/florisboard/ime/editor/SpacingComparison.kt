package dev.patrickgold.florisboard.ime.editor

import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.WordSuggestionCandidate
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.lib.FlorisLocale
import kotlin.random.Random

/**
 * Puts the keyboard's own spaces in Rust ([Spacing], the [EditorInstance.AutoSpaceState] and
 * [EditorInstance.PhantomSpaceState] wrappers, [PunctuationRule.Fallback] and
 * [FlorisLocale.supportsAutoSpace]) up against the Kotlin they replaced, [ReferenceSpacing],
 * wherever this runs: the JVM's tests and, copied in for a check, the phone, whose character rules
 * differ. Both sides see the same made-up editor, and must agree on every answer, every flag after
 * every step (to the bit), and which texts they looked up. It records each difference and how
 * often each outcome was seen, so a caller can tell the runs reached them all.
 */
class SpacingComparison {
    val differences = mutableListOf<String>()
    val seen = mutableMapOf<String, Int>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    private fun check(what: String, input: () -> String, kotlin: Any?, rust: Any?) {
        checks++
        if (kotlin != rust && differences.size < MAX_DIFFERENCES) differences += "$what(${input()}): kotlin $kotlin, rust $rust"
    }

    private fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    /** An editor both sides read, which notes which texts were looked up. */
    private class Editor : SpacingEditor {
        override var autoSpacePunctuation = true
        override var isRawInputEditor = false
        override var keyVariation = KeyVariation.NORMAL
        override var selection = EditorRange(3, 3)
        override var locale: FlorisLocale = FlorisLocale.from("en")
        override var punctuationRule = DEFAULT_RULE
        var before1 = ""
        var before3 = ""
        override var currentWordText = ""
        val asked = sortedSetOf<Int>()

        override fun textBeforeCursor(n: Int): String {
            asked += n
            return when (n) {
                1 -> before1
                3 -> before3
                else -> error("n = $n")
            }
        }

        override fun toString() =
            "pref=$autoSpacePunctuation raw=$isRawInputEditor $keyVariation $selection ${locale.language} " +
                "rule=[${punctuationRule.symbolsPrecedingAutoSpace.show()}|${punctuationRule.symbolsFollowingAutoSpace.show()}|" +
                "${punctuationRule.symbolsPrecedingPhantomSpace.show()}|${punctuationRule.symbolsFollowingPhantomSpace.show()}] " +
                "before1=${before1.show()} before3=${before3.show()} word=${currentWordText.show()}"
    }

    /** The two sides, each with its own flags, over one editor. */
    private class Sides(val editor: Editor) {
        val kotlin = ReferenceSpacing(editor)
        val autoSpace = EditorInstance.AutoSpaceState()
        val phantomSpace = EditorInstance.PhantomSpaceState()
    }

    /** Both sides' flags: raw, as read, and the suggestion kept. */
    private fun compareStates(sides: Sides, what: String, input: () -> String) {
        val kotlin = sides.kotlin
        val old = listOf(
            kotlin.autoSpace.raw, kotlin.autoSpace.isActive, kotlin.autoSpace.isInactive,
            kotlin.phantomSpace.raw, kotlin.phantomSpace.isActive, kotlin.phantomSpace.isInactive,
            kotlin.phantomSpace.showComposingRegion, kotlin.phantomSpace.candidateForRevert?.text,
        )
        val new = listOf(
            sides.autoSpace.bits, sides.autoSpace.isActive, sides.autoSpace.isInactive,
            sides.phantomSpace.bits, sides.phantomSpace.isActive, sides.phantomSpace.isInactive,
            sides.phantomSpace.showComposingRegion, sides.phantomSpace.candidateForRevert?.text,
        )
        check("$what: flags", input, old, new)
    }

    /** Committing [char] on both sides: the spaces, what each looked up, and the flags after. */
    private fun commitChar(sides: Sides, char: String): Spacing.CharSpaces {
        val editor = sides.editor
        val input = { "${char.show()}, $editor, auto=${sides.autoSpace.bits} phantom=${sides.phantomSpace.bits}" }
        editor.asked.clear()
        val old = sides.kotlin.commitChar(char)
        val oldAsked = editor.asked.toList()
        editor.asked.clear()
        val new = Spacing.commitChar(char, editor, sides.autoSpace, sides.phantomSpace)
        check("commitChar", input, old, new)
        check("commitChar: looked up", input, oldAsked, editor.asked.toList())
        compareStates(sides, "commitChar", input)
        see("char: none", old == NO_SPACES)
        see("char: delete previous space", old.deletePreviousSpace)
        see("char: space before", old.insertSpaceBeforeChar)
        see("char: space after", old.insertSpaceAfterChar)
        see("looked up: ${oldAsked.ifEmpty { "nothing" }}")
        return old
    }

    /** `determine` on both sides. */
    private fun determine(sides: Sides, text: String, forceActive: Boolean): Boolean {
        val editor = sides.editor
        val input = { "${text.show()}, force=$forceActive, $editor, phantom=${sides.phantomSpace.bits}" }
        editor.asked.clear()
        val old = with(sides.kotlin) { phantomSpace.determine(text, forceActive) }
        val oldAsked = editor.asked.toList()
        editor.asked.clear()
        val new = Spacing.phantomSpaceBefore(sides.phantomSpace, text, forceActive, editor)
        check("determine", input, old, new)
        check("determine: looked up", input, oldAsked, editor.asked.toList())
        see("determine: $old${if (forceActive) " forced" else ""}")
        see("determine: unspaced language", !ReferenceSpacing.supportsAutoSpace(editor.locale) && oldAsked.isNotEmpty())
        return old
    }

    fun constants() {
        check("Fallback", { "" }, ReferenceSpacing.Fallback, PunctuationRule.Fallback)
        val languages = LANGUAGES.map { FlorisLocale.fromTag(it) } +
            List(200) { i -> FlorisLocale.from(String(CharArray(i % 4) { (i * 7 + it * 13).toChar() })) }
        for (locale in languages) {
            check("supportsAutoSpace", { locale.language.show() }, ReferenceSpacing.supportsAutoSpace(locale), locale.supportsAutoSpace)
        }
    }

    /**
     * Every UTF-16 unit as the character typed, as the last character before the cursor, as the
     * one before an auto-space, in the current word, and in each of the rule's symbol lists.
     */
    fun everyUnit() {
        for (code in 0..0xFFFF) {
            val unit = code.toChar()
            val char = unit.toString()
            // As what is typed, after a word, with a phantom space owed.
            commitChar(sides(before1 = "a", before3 = "cat", phantom = 1), char)
            determine(sides(before1 = "a", before3 = "", phantom = 0), char, forceActive = true)
            // As what is before the cursor.
            commitChar(sides(before1 = char, before3 = "a$char", phantom = 5), ".")
            commitChar(sides(before1 = char, before3 = "ab$char", phantom = 5), "¿")
            commitChar(sides(before1 = char, before3 = "ab$char", phantom = 5), "b")
            // Before an auto-space, which is looked past.
            commitChar(sides(before1 = " ", before3 = "a$char ", auto = true), ",")
            // In the current word.
            commitChar(sides(before1 = "a", before3 = "ab", word = "x$char"), ".")
            // In the rule's symbols, as what is typed and as what is before the cursor.
            val rule = PunctuationRule("u", symbolsPrecedingAutoSpace = "x$char", symbolsFollowingAutoSpace = "$char",
                symbolsPrecedingPhantomSpace = "$char", symbolsFollowingPhantomSpace = "y$char", symbolsTerminatingSentence = "")
            commitChar(sides(before1 = "-", before3 = "-", phantom = 1, rule = rule), char)
            commitChar(sides(before1 = char, before3 = char, phantom = 1, rule = rule), "-")
        }
    }

    private fun sides(
        before1: String,
        before3: String,
        word: String = "",
        auto: Boolean = false,
        phantom: Int = 0,
        rule: PunctuationRule = DEFAULT_RULE,
    ): Sides {
        val editor = Editor().also {
            it.before1 = before1
            it.before3 = before3
            it.currentWordText = word
            it.punctuationRule = rule
        }
        return Sides(editor).also { sides ->
            if (auto) {
                sides.kotlin.autoSpace.setActive()
                sides.autoSpace.setActive()
            }
            if (phantom != 0) {
                sides.kotlin.phantomSpace.raw = phantom
                sides.phantomSpace.bits = phantom
            }
        }
    }

    /**
     * Random runs of what [EditorInstance] does with the flags: committing characters and words,
     * setting and clearing the flags, and selection updates, in random editors, and every state's
     * `Int` (not only those the keyboard sets) through an update.
     */
    fun runs(random: Random, runs: Int, steps: Int) = repeat(runs) {
        val sides = Sides(Editor())
        if (random.nextInt(4) == 0) {
            val state = STATES.random(random).takeIf { it != Int.MIN_VALUE } ?: random.nextInt()
            sides.kotlin.autoSpace.raw = state
            sides.autoSpace.bits = state
        }
        repeat(steps) { step ->
            randomize(random, sides.editor)
            val input = { "run step $step" }
            when (random.nextInt(10)) {
                0, 1, 2 -> commitChar(sides, text(random, 1 + random.nextInt(2)))
                3, 4 -> determine(sides, text(random, random.nextInt(4)), forceActive = random.nextInt(3) == 0)
                5 -> {
                    val show = random.nextBoolean()
                    val stay = random.nextBoolean()
                    val candidate = CANDIDATES.random(random)
                    sides.kotlin.phantomSpace.setActive(show, stay, candidate)
                    sides.phantomSpace.setActive(show, stay, candidate)
                }
                6 -> {
                    val stay = random.nextBoolean()
                    sides.kotlin.autoSpace.setActive(stay)
                    sides.autoSpace.setActive(stay)
                }
                7 -> {
                    sides.kotlin.autoSpace.setInactive()
                    sides.autoSpace.setInactive()
                    if (random.nextBoolean()) {
                        sides.kotlin.phantomSpace.setInactive()
                        sides.phantomSpace.setInactive()
                    }
                }
                8 -> {
                    // An arbitrary state, with a suggestion, through an update.
                    val state = STATES.random(random).takeIf { it != Int.MIN_VALUE } ?: random.nextInt()
                    val candidate = CANDIDATES.random(random)
                    sides.kotlin.phantomSpace.setActive(showComposingRegion = false, candidate = candidate)
                    sides.phantomSpace.setActive(showComposingRegion = false, candidate = candidate)
                    sides.kotlin.phantomSpace.raw = state
                    sides.phantomSpace.bits = state
                    update(sides)
                }
                else -> update(sides)
            }
            compareStates(sides, "step", input)
        }
    }

    private fun update(sides: Sides) {
        val had = sides.kotlin.phantomSpace.candidateForRevert != null
        sides.kotlin.autoSpace.setInactiveFromUpdate()
        sides.kotlin.phantomSpace.setInactiveFromUpdate()
        sides.autoSpace.setInactiveFromUpdate()
        sides.phantomSpace.setInactiveFromUpdate()
        val kept = sides.kotlin.phantomSpace.candidateForRevert != null
        see("update: candidate kept", had && kept)
        see("update: candidate dropped", had && !kept)
        see("update: still active", sides.kotlin.phantomSpace.isActive)
    }

    private fun randomize(random: Random, editor: Editor) {
        editor.autoSpacePunctuation = random.nextInt(4) != 0
        editor.isRawInputEditor = random.nextInt(6) == 0
        editor.keyVariation = if (random.nextInt(3) == 0) KeyVariation.entries.random(random) else KeyVariation.NORMAL
        editor.selection = when (random.nextInt(8)) {
            0 -> EditorRange.Unspecified
            1 -> EditorRange(0, random.nextInt(-1, 3))
            2 -> EditorRange(random.nextInt(-2, 3), random.nextInt(-2, 3))
            3 -> EditorRange(Int.MAX_VALUE, Int.MAX_VALUE)
            else -> EditorRange(random.nextInt(1, 50), random.nextInt(1, 50))
        }
        editor.locale = if (random.nextInt(3) == 0) FlorisLocale.fromTag(LANGUAGES.random(random)) else ENGLISH
        editor.punctuationRule = when (random.nextInt(5)) {
            0 -> PunctuationRule.Fallback
            1 -> PunctuationRule("r", symbolsPrecedingAutoSpace = text(random, random.nextInt(5)),
                symbolsFollowingAutoSpace = text(random, random.nextInt(3)), symbolsPrecedingPhantomSpace = text(random, random.nextInt(5)),
                symbolsFollowingPhantomSpace = text(random, random.nextInt(3)), symbolsTerminatingSentence = text(random, random.nextInt(3)))
            else -> DEFAULT_RULE
        }
        editor.before3 = text(random, random.nextInt(5)).let { if (random.nextInt(3) == 0) "$it " else it }
        editor.before1 = if (random.nextBoolean()) editor.before3.takeLast(1 + random.nextInt(2)) else text(random, random.nextInt(3))
        editor.currentWordText = text(random, random.nextInt(4))
    }

    /** [length] units, mostly ones the rules care about. */
    private fun text(random: Random, length: Int) = String(CharArray(length) {
        if (random.nextInt(4) == 0) random.nextInt(0x10000).toChar() else UNITS[random.nextInt(UNITS.length)]
    })

    private companion object {
        const val MAX_DIFFERENCES = 20
        val NO_SPACES = Spacing.CharSpaces(deletePreviousSpace = false, insertSpaceBeforeChar = false, insertSpaceAfterChar = false)
        val DEFAULT_RULE = PunctuationRule(
            id = "default",
            symbolsPrecedingAutoSpace = ".,?‽!\"&%)]}»",
            symbolsFollowingAutoSpace = "¿⸘¡",
            symbolsPrecedingPhantomSpace = ".,;:?‽!&%)]}»©®™",
            symbolsFollowingPhantomSpace = "¿⸘¡([{",
            symbolsTerminatingSentence = ".?‽!",
        )
        val ENGLISH: FlorisLocale = FlorisLocale.from("en")
        val LANGUAGES = listOf("en", "zh", "ko", "jp", "ja", "th", "de", "ZH", "Th", "zh_TW", "ko-KR", "zhh", "z", "", "jpn", "en_US")
        val STATES = listOf(0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 15, -1, -4, -5, Int.MAX_VALUE, Int.MIN_VALUE)

        /** Spaces of every kind, the separators, digits and letters of other scripts, half emoji. */
        const val UNITS = "aZ3 .,?!¿¡(   \u001C\u001F\u0085\t\n 　٣²ªʰ😀'-"

        val CANDIDATES: List<SuggestionCandidate?> = listOf(null, WordSuggestionCandidate("one"), WordSuggestionCandidate("two"))
    }
}

private fun String.show() = map { if (it in ' '..'~') "$it" else "\\u%04X".format(it.code) }.joinToString("", "\"", "\"")
