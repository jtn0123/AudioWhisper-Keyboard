package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.SpacingRules
import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.lib.FlorisLocale

/**
 * What [Spacing] asks of the editor. [EditorInstance] answers from its content and settings; a
 * test, from made-up ones. The text lookups are only made when a rule gets far enough to need them.
 */
internal interface SpacingEditor {
    /** The setting to space after punctuation. */
    val autoSpacePunctuation: Boolean
    val isRawInputEditor: Boolean
    val keyVariation: KeyVariation
    val selection: EditorRange
    val locale: FlorisLocale
    val punctuationRule: PunctuationRule

    fun textBeforeCursor(n: Int): String

    val currentWordText: String
}

/**
 * The spaces [EditorInstance] types by itself, decided in Rust (lib/rust/crates/spacing): auto-spacing
 * after punctuation, and the phantom space owed after a picked or swiped word.
 */
internal object Spacing {
    /** What committing a character does with spaces. */
    data class CharSpaces(
        val deletePreviousSpace: Boolean,
        val insertSpaceBeforeChar: Boolean,
        val insertSpaceAfterChar: Boolean,
    )

    /**
     * What committing [char] does with spaces; sets [autoSpace] if a space is typed after it, and
     * clears [phantomSpace], which the character uses up.
     */
    fun commitChar(
        char: String,
        editor: SpacingEditor,
        autoSpace: EditorInstance.AutoSpaceState,
        phantomSpace: EditorInstance.PhantomSpaceState,
    ): CharSpaces {
        val spaces = SpacingRules.commitChar(char, typing(editor), autoSpace.bits, phantomSpace.bits, Lookup(editor))
        autoSpace.bits = spaces.autoSpace
        phantomSpace.reset(spaces.phantomSpace)
        return CharSpaces(spaces.deletePreviousSpace, spaces.spaceBefore, spaces.spaceAfter)
    }

    /** Whether to type the space [phantomSpace] owes, or one [forceActive], before [text]. */
    fun phantomSpaceBefore(
        phantomSpace: EditorInstance.PhantomSpaceState,
        text: String,
        forceActive: Boolean,
        editor: SpacingEditor,
    ): Boolean = SpacingRules.phantomSpaceBefore(text, typing(editor), phantomSpace.bits, forceActive, Lookup(editor))

    private fun typing(editor: SpacingEditor): SpacingRules.Typing {
        val selection = editor.selection
        return SpacingRules.Typing(
            autoSpacePunctuation = editor.autoSpacePunctuation,
            rawEditor = editor.isRawInputEditor,
            keyVariation = editor.keyVariation.value,
            selectionValid = selection.isValid,
            selectionStart = selection.start,
        )
    }

    private class Lookup(private val editor: SpacingEditor) : SpacingRules.Editor {
        override val rule: SpacingRules.Rule
            get() = editor.punctuationRule.let {
                SpacingRules.Rule(
                    precedingAutoSpace = it.symbolsPrecedingAutoSpace,
                    followingAutoSpace = it.symbolsFollowingAutoSpace,
                    precedingPhantomSpace = it.symbolsPrecedingPhantomSpace,
                    followingPhantomSpace = it.symbolsFollowingPhantomSpace,
                )
            }

        override val language: String get() = editor.locale.language

        override fun textBeforeCursor(n: Int): String = editor.textBeforeCursor(n)

        override val currentWordText: String get() = editor.currentWordText
    }
}
