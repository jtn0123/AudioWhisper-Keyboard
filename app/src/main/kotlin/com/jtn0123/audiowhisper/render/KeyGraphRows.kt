package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyGraphAsk
import com.jtn0123.audiowhisper.rust.KeyGraphBuild
import com.jtn0123.audiowhisper.rust.KeyGraphFacts
import com.jtn0123.audiowhisper.rust.KeyGraphFile
import com.jtn0123.audiowhisper.rust.LayoutCache
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.ime.core.SubtypeFacts
import dev.patrickgold.florisboard.ime.keyboard.KEY_LABEL_RESOURCES
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.LayoutAssembly
import dev.patrickgold.florisboard.ime.keyboard.LayoutType
import dev.patrickgold.florisboard.ime.keyboard.extCorePopupMapping
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import java.util.Locale

/** A layout by its type and name, as `LayoutAssembly.plan` names one. */
internal typealias LayoutName = Pair<LayoutType, ExtensionComponentName>

/**
 * Where a key graph's files come from: the subtype's [layouts], the modifier a layout's index
 * entry names ([namedModifier]), each layout file's text ([layout], null when it didn't read)
 * and the popup mappings' ([mapping], the subtype's, and [fallback], the default one).
 */
internal class KeyGraphFiles(
    val layouts: (LayoutType) -> ExtensionComponentName?,
    val namedModifier: (LayoutName) -> ExtensionComponentName?,
    val layout: (LayoutName) -> String?,
    val mapping: String?,
    val fallback: String?,
)

/**
 * The files of the subtype Rust says names [facts]: its layouts and popup mapping, the modifier a
 * layout's index entry names, each layout file's text, and each popup mapping's ([mapping]).
 */
internal fun subtypeFiles(facts: SubtypeFacts, namedModifier: (LayoutName) -> ExtensionComponentName?,
                          layout: (LayoutName) -> String?, mapping: (ExtensionComponentName) -> String?) =
    KeyGraphFiles(facts::layout, namedModifier, layout, mapping(facts.popupMapping), mapping(extCorePopupMapping("default")))

/** The keyboard [this] keeps for [mode] and the subtype [facts] names, [build] the first time it is asked for. */
internal fun <T : Any> LayoutCache<T>.subtypeKeyboard(mode: KeyboardMode, facts: SubtypeFacts, build: () -> T): T =
    get(mode.value, facts.key) ?: build().also { set(mode.value, facts.key, it) }

/**
 * A `KeyGraphBuild` row for [mode]: its layouts' files, its symbols' for a characters keyboard,
 * and the popup mappings.
 */
internal fun keyGraphBuild(mode: KeyboardMode, numberRow: Boolean, numberHints: Boolean,
                           files: KeyGraphFiles): Array<String> {
    val fields = Array(KeyGraphBuild.SIZE) { "" }
    var loaded = 0
    fun put(bit: Int, field: Int, text: String?) = text?.let { loaded = loaded or bit; fields[field] = it }
    for ((layoutMode, set) in listOf(mode to MAIN_FILES, KeyboardMode.SYMBOLS to SYMBOLS_FILES)) {
        if (layoutMode != mode && mode != KeyboardMode.CHARACTERS) continue
        val (main, modifier, extension) = LayoutAssembly.plan(layoutMode, numberRow, files.layouts)
        val own = main?.let { name ->
            files.namedModifier(name)?.let { LayoutAssembly.modifier(name.first) to it }
        }
        if (own != null) loaded = loaded or set.names
        val named = listOf(set.main to main, set.modifier to modifier, set.own to own, set.extension to extension)
        for ((file, name) in named) {
            put(file.first, file.second, name?.let(files.layout))
        }
    }
    put(KeyGraphFile.MAPPING, KeyGraphBuild.MAPPING, files.mapping)
    put(KeyGraphFile.FALLBACK, KeyGraphBuild.FALLBACK, files.fallback)
    fields[KeyGraphBuild.MODE] = "${mode.value}"
    fields[KeyGraphBuild.NUMBER_HINTS] = WireFlag.encode(numberHints)
    fields[KeyGraphBuild.LOADED] = "$loaded"
    return fields
}

/**
 * What one computation of a key graph reads, as Android reports it: the evaluator's [state] and
 * editor, the settings and system facts the key evaluator read, the language names, the hint
 * settings, the subtype's currency [slots] and each of [KEY_LABEL_RESOURCES]' label.
 */
internal data class KeyGraphFactsInput(
    val state: ULong, val input: Int, val options: Int, val utility: Boolean, val utilityAction: Int,
    val languageSwitch: Boolean, val richEditor: Boolean, val locked: Boolean, val pastable: Boolean,
    val incognito: Int, val noLearning: Boolean, val subtypes: Int, val dictation: Boolean,
    val language: String, val defaultLanguage: String, val hints: KeyHintConfiguration,
    val slots: List<TextKeyData>, val resources: List<String?>,
) {
    /**
     * A `KeyGraphFacts` row, then each currency slot's code and label, then each label resource's
     * token and label.
     */
    fun row(): Array<String> = with(KeyGraphFacts) {
        fun flag(value: Boolean) = WireFlag.encode(value)
        row(STATE to "$state", INPUT to "$input", OPTIONS to "$options", UTILITY to flag(utility),
            UTILITY_ACTION to "$utilityAction", LANGUAGE_SWITCH to flag(languageSwitch),
            RICH_EDITOR to flag(richEditor), LOCKED to flag(locked), PASTABLE to flag(pastable),
            INCOGNITO to "$incognito", NO_LEARNING to flag(noLearning), SUBTYPES to "$subtypes",
            DICTATION to flag(dictation), LANGUAGE to language, DEFAULT_LANGUAGE to defaultLanguage,
            HINT_SYMBOL to "${hints.symbolHintMode.ordinal}", HINT_NUMBER to "${hints.numberHintMode.ordinal}",
            HINT_MERGE to flag(hints.mergeHintPopups), SLOTS to "${slots.size}",
            RESOURCES to "${KEY_LABEL_RESOURCES.size}") +
            slots.flatMap { listOf("${it.code}", it.label) } +
            KEY_LABEL_RESOURCES.indices.flatMap { listOf("${KEY_LABEL_RESOURCES[it]}", resources[it] ?: "") }
    }
}

/**
 * The platform's answer to a key graph's question: [value] cased in [locale], where a code point
 * becomes a text by [codeText], or whether a code point is a non-spacing [mark].
 */
internal fun keyGraphAnswer(ask: Int, value: String, locale: Locale, codeText: (Int) -> String,
                            mark: (Int) -> Boolean): String =
    when (ask) {
        KeyGraphAsk.LOWER_CODE -> "${codeText(value.toInt()).lowercase(locale).codePointAt(0)}"
        KeyGraphAsk.UPPER_CODE -> "${codeText(value.toInt()).uppercase(locale).codePointAt(0)}"
        KeyGraphAsk.LOWER_TEXT -> value.lowercase(locale)
        KeyGraphAsk.UPPER_TEXT -> value.uppercase(locale)
        else -> WireFlag.encode(mark(value.toInt()))
    }

/** One keyboard's layout files in a `KeyGraphBuild` row: each file's `KeyGraphFile` bit and its field. */
private class LayoutFiles(val main: Pair<Int, Int>, val modifier: Pair<Int, Int>, val names: Int,
                          val own: Pair<Int, Int>, val extension: Pair<Int, Int>)

private val MAIN_FILES = LayoutFiles(KeyGraphFile.MAIN to KeyGraphBuild.MAIN,
    KeyGraphFile.MODIFIER to KeyGraphBuild.MODIFIER, KeyGraphFile.NAMES_MODIFIER,
    KeyGraphFile.OWN_MODIFIER to KeyGraphBuild.OWN_MODIFIER,
    KeyGraphFile.EXTENSION to KeyGraphBuild.EXTENSION)

private val SYMBOLS_FILES = LayoutFiles(KeyGraphFile.SYMBOLS_MAIN to KeyGraphBuild.SYMBOLS_MAIN,
    KeyGraphFile.SYMBOLS_MODIFIER to KeyGraphBuild.SYMBOLS_MODIFIER, KeyGraphFile.SYMBOLS_NAMES_MODIFIER,
    KeyGraphFile.SYMBOLS_OWN_MODIFIER to KeyGraphBuild.SYMBOLS_OWN_MODIFIER,
    KeyGraphFile.SYMBOLS_EXTENSION to KeyGraphBuild.SYMBOLS_EXTENSION)
