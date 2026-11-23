package dev.patrickgold.florisboard.ime.editor

import com.ibm.icu.text.BreakIterator
import java.util.Locale

/** ICU's boundary results are platform data, which [NativeEditorDriver] answers Rust's queries with. */
class ReferenceBreaks {
    suspend fun measureUChars(text: String, n: Int, locale: Locale = Locale.ENGLISH): Int = measure(text,n,locale,false,false)
    suspend fun measureLastUChars(text: String, n: Int, locale: Locale = Locale.ENGLISH): Int = measure(text,n,locale,true,false)
    fun measure(text: String, count: Int, locale: Locale, backwards: Boolean, words: Boolean): Int {
        val iterator = if (words) BreakIterator.getWordInstance(locale) else BreakIterator.getCharacterInstance(locale)
        iterator.setText(text)
        val edge = if (backwards) iterator.last() else iterator.first()
        var position: Int
        var n = 0
        do {
            if (backwards && words && iterator.ruleStatus != BreakIterator.WORD_NONE) n++
            position = if (backwards) iterator.previous() else iterator.next()
            if (!words || !backwards && iterator.ruleStatus != BreakIterator.WORD_NONE) n++
        } while (position != BreakIterator.DONE && n < count)
        val boundary = if (position == BreakIterator.DONE) { if (backwards) 0 else text.length } else position
        return (if (backwards) edge-boundary else boundary-edge).coerceIn(0,text.length)
    }
}
object ReferencePlatform {
    fun word(text: String): ReferenceEditorRange {
        val iterator = BreakIterator.getWordInstance(Locale.ENGLISH)
        iterator.setText(text)
        val end = iterator.last()
        if (iterator.ruleStatus == BreakIterator.WORD_NONE) return ReferenceEditorRange.Unspecified
        var start = iterator.previous()
        if (text.getOrNull(start-1) == ':') start--
        return ReferenceEditorRange(start,end)
    }
}
