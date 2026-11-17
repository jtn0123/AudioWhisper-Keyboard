package dev.patrickgold.florisboard.ime.editor

import android.text.InputType
import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.lib.FlorisLocale
import java.util.concurrent.atomic.AtomicInteger

/**
 * The Kotlin that decided the keyboard's own spaces before it moved to Rust
 * (lib/rust/crates/spacing), kept as [SpacingComparison]'s reference: `EditorInstance`'s
 * `shouldInsertAutoSpaceBefore`, `shouldInsertAutoSpaceAfter`, `commitChar`'s flags,
 * `PhantomSpaceState.determine`, `AutoSpaceState` and `PhantomSpaceState`, `PunctuationRule.Fallback`
 * and `FlorisLocale.supportsAutoSpace`. The only edits: what `EditorInstance` read from its
 * managers and content comes from [editor], and `commitChar` returns its flags instead of
 * committing. Each state class also shows its raw `Int`, for the comparison. Two deliberate
 * changes since: `determine` types no phantom space outside [KeyVariation.NORMAL], as AOSP spaces
 * no URL, e-mail address or password, except in a URL or e-mail field that asks for
 * auto-correction (`TYPE_TEXT_FLAG_AUTO_CORRECT`), as Gboard spaces a word picked in one
 * (docs/decisions.md, "Field kinds").
 */
internal class ReferenceSpacing(private val editor: SpacingEditor) {
    val autoSpace = AutoSpaceState()
    val phantomSpace = PhantomSpaceState()

    private fun shouldInsertAutoSpaceBefore(text: String): Boolean {
        if (!editor.autoSpacePunctuation || text.isEmpty()) return false
        if (editor.isRawInputEditor) return false
        if (editor.keyVariation != KeyVariation.NORMAL) return false

        val punctuationRule = editor.punctuationRule
        val textBefore = editor.textBeforeCursor(1)
        return textBefore.isNotEmpty() && !textBefore.last().isWhitespace() &&
            punctuationRule.symbolsFollowingAutoSpace.contains(text.first())
    }

    private fun shouldInsertAutoSpaceAfter(text: String): Boolean {
        if (!editor.autoSpacePunctuation || text.isEmpty()) return false
        if (editor.isRawInputEditor) return false
        if (editor.keyVariation != KeyVariation.NORMAL) return false

        val punctuationRule = editor.punctuationRule
        val content = editor
        val textBefore = content.textBeforeCursor(3).let { textBefore ->
            if (autoSpace.isActive && textBefore.isNotEmpty() && textBefore.last() == ' ') {
                textBefore.dropLast(1)
            } else {
                textBefore
            }
        }
        return textBefore.isNotEmpty() && !textBefore.last().isWhitespace() &&
            content.currentWordText.all { !it.isDigit() } &&
            punctuationRule.symbolsPrecedingAutoSpace.contains(text.first())
    }

    fun commitChar(char: String): Spacing.CharSpaces {
        val isInsertAutoSpaceBeforeChar = shouldInsertAutoSpaceBefore(char)
        val isInsertAutoSpaceAfterChar = shouldInsertAutoSpaceAfter(char)
        val isDeletePreviousSpace = isInsertAutoSpaceAfterChar && autoSpace.isActive
        if (isInsertAutoSpaceAfterChar) {
            autoSpace.setActive()
        } else {
            autoSpace.setInactive()
        }
        val isPhantomSpaceActive = phantomSpace.determine(char)
        phantomSpace.setInactive()
        return Spacing.CharSpaces(
            deletePreviousSpace = isDeletePreviousSpace,
            insertSpaceBeforeChar = isInsertAutoSpaceBeforeChar || isPhantomSpaceActive,
            insertSpaceAfterChar = isInsertAutoSpaceAfterChar,
        )
    }

    fun PhantomSpaceState.determine(text: String, forceActive: Boolean = false): Boolean {
         val content = editor
         val selection = content.selection
         if (!(isActive || forceActive) || selection.isNotValid || selection.start <= 0 || text.isEmpty()) return false
         val asksCorrection = editor.inputType and InputType.TYPE_TEXT_FLAG_AUTO_CORRECT != 0
         val address = editor.keyVariation == KeyVariation.EMAIL_ADDRESS || editor.keyVariation == KeyVariation.URI
         if (editor.keyVariation != KeyVariation.NORMAL && !(address && asksCorrection)) return false
         val textBefore = content.textBeforeCursor(1)
         val punctuationRule = editor.punctuationRule
         if (!supportsAutoSpace(editor.locale)) return false;
         return textBefore.isNotEmpty() &&
             (punctuationRule.symbolsPrecedingPhantomSpace.contains(textBefore[textBefore.length - 1]) ||
                 textBefore[textBefore.length - 1].isLetterOrDigit()) &&
             (punctuationRule.symbolsFollowingPhantomSpace.contains(text[0]) || text[0].isLetterOrDigit())
    }

    class AutoSpaceState {
        companion object {
            private const val F_IS_ACTIVE = 0x1
            private const val F_STAY_ACTIVE_NEXT_UPDATE = 0x4
        }

        private val state = AtomicInteger(0)

        /** Added for the comparison: the state as it is. */
        var raw: Int
            get() = state.get()
            set(value) = state.set(value)

        val isActive: Boolean
            get() = state.get() and F_IS_ACTIVE != 0

        val isInactive: Boolean
            get() = !isActive

        fun setActive(stayActiveNextUpdate: Boolean = true) {
            state.set(F_IS_ACTIVE or (if (stayActiveNextUpdate) F_STAY_ACTIVE_NEXT_UPDATE else 0))
        }

        fun setInactive() {
            state.set(0)
        }

        fun setInactiveFromUpdate() {
            state.updateAndGet { state ->
                if ((state and F_STAY_ACTIVE_NEXT_UPDATE) != 0) (state and F_STAY_ACTIVE_NEXT_UPDATE.inv()) else 0
            }
        }
    }

    class PhantomSpaceState {
        companion object {
            private const val F_IS_ACTIVE = 0x1
            private const val F_SHOW_COMPOSING_REGION = 0x2
            private const val F_STAY_ACTIVE_NEXT_UPDATE = 0x4
        }

        private val state = AtomicInteger(0)
        var candidateForRevert: SuggestionCandidate? = null
            private set

        /** Added for the comparison: the state as it is. */
        var raw: Int
            get() = state.get()
            set(value) = state.set(value)

        val isActive: Boolean
            get() = state.get() and F_IS_ACTIVE != 0

        val isInactive: Boolean
            get() = !isActive

        val showComposingRegion: Boolean
            get() = state.get() and F_SHOW_COMPOSING_REGION != 0

        fun setActive(
            showComposingRegion: Boolean,
            stayActiveNextUpdate: Boolean = true,
            candidate: SuggestionCandidate? = null,
        ) {
            state.set(
                F_IS_ACTIVE
                    or (if (showComposingRegion) F_SHOW_COMPOSING_REGION else 0)
                    or (if (stayActiveNextUpdate) F_STAY_ACTIVE_NEXT_UPDATE else 0)
            )
            candidateForRevert = candidate
        }

        fun setInactive() {
            state.set(0)
            candidateForRevert = null
        }

        fun setInactiveFromUpdate() {
            val prevStateValue = state.getAndUpdate { state ->
                if ((state and F_STAY_ACTIVE_NEXT_UPDATE) != 0) (state and F_STAY_ACTIVE_NEXT_UPDATE.inv()) else 0
            }
            if ((prevStateValue and F_STAY_ACTIVE_NEXT_UPDATE) == 0) {
                candidateForRevert = null
            }
        }
    }

    companion object {
        /** `PunctuationRule.Fallback`. */
        val Fallback = PunctuationRule(
            id = "fallback",
            label = "Fallback",
            symbolsPrecedingAutoSpace = ".,?!",
            symbolsFollowingAutoSpace = "",
            symbolsPrecedingPhantomSpace = ".,?!",
            symbolsFollowingPhantomSpace = "",
            symbolsTerminatingSentence = ".?!",
        )

        /** `FlorisLocale.supportsAutoSpace`. */
        fun supportsAutoSpace(locale: FlorisLocale): Boolean =
            when (locale.language) {
                "zh", "ko", "jp", "th" -> false
                else -> true
            }
    }
}
