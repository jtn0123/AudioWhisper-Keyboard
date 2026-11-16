package com.jtn0123.audiowhisper.rust

/**
 * The spaces the keyboard types by itself, in Rust (lib/rust/crates/spacing): a phantom space
 * owed after a picked or swiped word, auto-spacing after punctuation, and the two flags' states,
 * which the caller keeps as plain `Int`s.
 *
 * The rules look text up only once they get far enough to need it, as the Kotlin they replace
 * did: [Editor] is asked for the rule, the language and the text before the cursor only then.
 */
object SpacingRules {
    /** The symbols a punctuation rule lists for each kind of space. */
    class Rule(
        val precedingAutoSpace: String,
        val followingAutoSpace: String,
        val precedingPhantomSpace: String,
        val followingPhantomSpace: String,
    )

    /** What the rules look up when they need it. */
    interface Editor {
        val rule: Rule

        /** The subtype's primary language, as `Locale.getLanguage` gives it. */
        val language: String

        /** The last [n] characters before the cursor. */
        fun textBeforeCursor(n: Int): String

        val currentWordText: String
    }

    /** What is known before any text is looked up; [inputType] is the field's raw `EditorInfo.inputType`. */
    class Typing(
        val autoSpacePunctuation: Boolean,
        val rawEditor: Boolean,
        val keyVariation: Int,
        val selectionValid: Boolean,
        val selectionStart: Int,
        val inputType: Int,
    )

    /** What typing a character does with spaces, and the flags after it. */
    @JvmInline
    value class Spaces internal constructor(private val bits: Int) {
        val deletePreviousSpace: Boolean get() = bits and DELETE_PREVIOUS_SPACE != 0
        val spaceBefore: Boolean get() = bits and SPACE_BEFORE != 0
        val spaceAfter: Boolean get() = bits and SPACE_AFTER != 0
        val autoSpace: Int get() = (bits ushr AUTO_SPACE_SHIFT) and STATE_MASK
        val phantomSpace: Int get() = (bits ushr PHANTOM_SPACE_SHIFT) and STATE_MASK
        internal val needs: Int get() = (bits ushr NEEDS_SHIFT) and NEEDS_MASK
    }

    /** What typing the character [char] does, with the flags [autoSpace] and [phantomSpace]. */
    fun commitChar(char: String, typing: Typing, autoSpace: Int, phantomSpace: Int, editor: Editor): Spaces =
        ask(char, typing, forceActive = false, autoSpace, phantomSpace, editor, SpacingNative::commitChar)

    /** Whether to type the owed phantom space ([phantomSpace]), or one [forceActive], before [text]. */
    fun phantomSpaceBefore(text: String, typing: Typing, phantomSpace: Int, forceActive: Boolean, editor: Editor): Boolean =
        ask(text, typing, forceActive, INACTIVE, phantomSpace, editor, SpacingNative::phantomSpaceBefore).spaceBefore

    private fun ask(
        text: String,
        typing: Typing,
        forceActive: Boolean,
        autoSpace: Int,
        phantomSpace: Int,
        editor: Editor,
        rule: (String, IntArray, String?, IntArray?) -> Int,
    ): Spaces {
        var flags = if (typing.autoSpacePunctuation) AUTO_SPACE_PUNCTUATION else 0
        if (typing.rawEditor) flags = flags or RAW_EDITOR
        if (typing.selectionValid) flags = flags or SELECTION_VALID
        if (forceActive) flags = flags or FORCE_ACTIVE
        val known = intArrayOf(flags, typing.keyVariation, typing.selectionStart, autoSpace, phantomSpace, typing.inputType)
        val first = Spaces(rule(text, known, null, null))
        val needs = first.needs
        if (needs == 0) return first
        val symbols = editor.rule
        val before1 = if (needs and NEEDS_BEFORE_1 != 0) editor.textBeforeCursor(1) else null
        val before3 = if (needs and NEEDS_BEFORE_3 != 0) editor.textBeforeCursor(3) else null
        val pieces = listOf(
            symbols.precedingAutoSpace,
            symbols.followingAutoSpace,
            symbols.precedingPhantomSpace,
            symbols.followingPhantomSpace,
            editor.language,
            before1,
            before3,
            if (before3 != null) editor.currentWordText else null,
        )
        val lookedUp = buildString { pieces.forEach { if (it != null) append(it) } }
        val lengths = IntArray(pieces.size) { pieces[it]?.length ?: -1 }
        return Spaces(rule(text, known, lookedUp, lengths)).also { check(it.needs == 0) { "still needs ${it.needs}" } }
    }

    /** Whether a word in [language] is followed by a space (`FlorisLocale.supportsAutoSpace`). */
    fun supportsAutoSpace(language: String): Boolean = SpacingNative.supportsAutoSpace(language)

    /** `PunctuationRule.Fallback`'s id, label and five symbol strings, in its constructor's order. */
    val fallbackRule: List<String> = SpacingNative.fallbackRule().toList()

    /** Neither flag set. */
    val INACTIVE: Int = SpacingNative.inactive()

    /** An active auto-space. */
    fun autoSpaceActive(stayActiveNextUpdate: Boolean): Int = SpacingNative.autoSpaceActive(stayActiveNextUpdate)

    /** An active phantom space. */
    fun phantomSpaceActive(showComposingRegion: Boolean, stayActiveNextUpdate: Boolean): Int =
        SpacingNative.phantomSpaceActive(showComposingRegion, stayActiveNextUpdate)

    /** Whether the space is owed. */
    fun isActive(state: Int): Boolean = SpacingNative.isActive(state)

    /** Whether the word before a phantom space shows as being composed. */
    fun showsComposingRegion(state: Int): Boolean = SpacingNative.showsComposingRegion(state)

    /** The state after the editor's selection changed: a mark to stay active is used up, else it clears. */
    fun afterUpdate(state: Int): Int = SpacingNative.afterUpdate(state)

    /** Whether a phantom space in [state] before an update keeps the suggestion it came from. */
    fun keepsCandidate(state: Int): Boolean = SpacingNative.keepsCandidate(state)

    // The packing of awnative/src/spacing.rs.
    private const val AUTO_SPACE_PUNCTUATION = 1
    private const val RAW_EDITOR = 2
    private const val SELECTION_VALID = 4
    private const val FORCE_ACTIVE = 8
    private const val DELETE_PREVIOUS_SPACE = 1
    private const val SPACE_BEFORE = 2
    private const val SPACE_AFTER = 4
    private const val NEEDS_SHIFT = 3
    private const val NEEDS_MASK = 3
    private const val NEEDS_BEFORE_1 = 1
    private const val NEEDS_BEFORE_3 = 2
    private const val AUTO_SPACE_SHIFT = 8
    private const val PHANTOM_SPACE_SHIFT = 16
    private const val STATE_MASK = 0xFF
}
