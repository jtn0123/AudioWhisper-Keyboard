package dev.patrickgold.florisboard.ime.editor

import com.ibm.icu.text.BreakIterator
import java.util.Locale

/** ICU's entire boundary result is platform data; both implementations receive the same data. */
class ReferenceBreaks {
    suspend fun measureUChars(text: String, n: Int, locale: Locale = Locale.ENGLISH): Int = measure(text,n,locale,false,false)
    suspend fun measureLastUChars(text: String, n: Int, locale: Locale = Locale.ENGLISH): Int = measure(text,n,locale,true,false)
    suspend fun measureUWords(text: String, n: Int, locale: Locale = Locale.ENGLISH): Int = measure(text,n,locale,false,true)
    suspend fun measureLastUWords(text: String, n: Int, locale: Locale = Locale.ENGLISH): Int = measure(text,n,locale,true,true)
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
class ReferenceComposer {
    val toRead = 1
    fun getActions(previous: String, char: String): Pair<Int,String> =
        if (previous.endsWith("a") && char == "\u0301") 1 to "á" else 0 to char
}
class ReferenceFlorisEditorInfo(
    val inputAttributes: ReferenceInputAttributes = ReferenceInputAttributes.wrap(0x4001),
    val initialSelection: ReferenceEditorRange = ReferenceEditorRange.cursor(0),
    val isRawInputEditor: Boolean = false,
) {
    val isRichInputEditor get() = !isRawInputEditor
    val initialCapsMode get() = inputAttributes.capsMode
    companion object { val Unspecified = ReferenceFlorisEditorInfo(isRawInputEditor = true) }
}

class ReferenceKeyboardManager { fun reevaluateInputShiftState() {} }
class ReferenceSubtypeManager { val activeSubtype = ReferenceSubtype() }
class ReferenceSubtype { val primaryLocale: Locale = Locale.ENGLISH; val composer = Unit }
class ReferenceNlpManager {
    suspend fun determineLocalComposing(text: CharSequence, breaks: ReferenceBreaks, last: Int): ReferenceEditorRange = ReferencePlatform.word(text.toString())
}
