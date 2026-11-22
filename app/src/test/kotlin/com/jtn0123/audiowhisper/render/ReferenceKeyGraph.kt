package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.InputProtocol
import com.jtn0123.audiowhisper.rust.InputRules
import com.jtn0123.audiowhisper.rust.KeyGeometry
import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.CaseSelector
import dev.patrickgold.florisboard.ime.keyboard.CharWidthSelector
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.KeyboardState
import dev.patrickgold.florisboard.ime.keyboard.LayoutArrangement
import dev.patrickgold.florisboard.ime.keyboard.LayoutDirectionSelector
import dev.patrickgold.florisboard.ime.keyboard.ShiftStateSelector
import dev.patrickgold.florisboard.ime.keyboard.VariationSelector
import dev.patrickgold.florisboard.ime.popup.PopupMapping
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.popup.ReferenceMutablePopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.ime.text.keyboard.AutoTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.MultiTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import java.util.Locale

/*
 * Main's Kotlin key graph at b10bee9b1, frozen to test the Rust graph (lib/rust/crates/keygraph)
 * against: `TextKey.compute` and `computeLabelsAndDrawables`, each key data's `compute`,
 * `ComputingEvaluatorImpl`, `computeLabel` and `computeImageVector`, and the key half of
 * `LayoutManager.mergeLayouts`. Decisions, order and object identity are main's; what main read
 * from Android (prefs, clipboard, keyguard, ICU casing, resources) arrives as plain facts and
 * functions, the same ones the Rust graph is given. The merge, the modifier choice and popup
 * ordering are the frozen ReferenceLayoutAssembly and ReferenceMutablePopupSet.
 */

/** What main's evaluator answered from: the facts, plus ICU's casing and mark predicate. */
internal class ReferencePlatform(val codeText: (Int) -> String, val mark: (Int) -> Boolean, val string: (Int) -> String)

/**
 * `ComputingEvaluatorImpl` (or, with [slots] off, `DefaultComputingEvaluator`'s answers) on a
 * keyboard of [mode] for a subtype in [locale], whose language is [language].
 */
internal class ReferenceEvaluator(
    val mode: KeyboardMode, val facts: KeyGraphFactsInput, val locale: Locale, val language: String,
    val platform: ReferencePlatform, private val slots: Boolean = true,
) {
    val state: KeyboardState = KeyboardState.new(facts.state)

    fun evaluateEnabled(data: KeyData): Boolean {
        fun bit(value: Boolean) = if (value) 1 else 0
        val (a, b) = when (data.code) {
            KeyCode.CLIPBOARD_COPY, KeyCode.CLIPBOARD_CUT -> bit(state.isSelectionMode) to bit(facts.richEditor)
            KeyCode.CLIPBOARD_PASTE -> bit(facts.locked) to bit(!facts.locked && facts.pastable)
            KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP -> bit(facts.pastable) to 0
            KeyCode.CLIPBOARD_SELECT_ALL -> bit(facts.richEditor) to 0
            KeyCode.TOGGLE_INCOGNITO_MODE -> facts.incognito to bit(facts.noLearning)
            KeyCode.LANGUAGE_SWITCH -> facts.subtypes to 0
            KeyCode.VOICE_INPUT -> bit(facts.dictation) to 0
            else -> 0 to 0
        }
        return KeyModels.enabled(data.code, a, b)
    }

    fun evaluateVisible(data: KeyData): Boolean =
        KeyModels.visible(data.code, facts.utility, facts.utilityAction, facts.languageSwitch)

    fun isSlot(data: KeyData): Boolean = slots && KeyModels.isCurrency(data.code)

    fun slotData(data: KeyData): KeyData? = facts.slots.getOrNull(KeyModels.currencySlot(data.code))

    /** Each `AutoTextKeyData`'s `AutoLetterState`, cased once for this locale. */
    private val cased = java.util.IdentityHashMap<AutoTextKeyData, Pair<TextKeyData, TextKeyData>>()

    /** `KeyData.compute` and each selector's, as main computed them. */
    fun compute(data: AbstractKeyData): KeyData? = when (data) {
        is TextKeyData -> if (isSlot(data)) slotData(data)?.let { slot(data, it) } else data
        is AutoTextKeyData -> if (isSlot(data)) {
            slotData(data)?.let { slot(data, it) }
        } else {
            val (lower, upper) = cased.getOrPut(data) { case(data) }
            if (KeyModels.select(0, state.rawValue.toLong(), 3) == 0) lower else upper
        }
        is MultiTextKeyData -> data
        is CaseSelector -> select(0, data.lower, data.upper)
        is ShiftStateSelector -> with(data) {
            select(1, unshifted, shifted, shiftedManual, shiftedAutomatic, capsLock, default)
        }
        is VariationSelector -> select(2, data.default, data.email, data.uri, data.normal, data.password)
        is LayoutDirectionSelector -> select(3, data.ltr, data.rtl)
        is CharWidthSelector -> select(4, data.full, data.half)
        else -> error("no such key data in a layout: $data")
    }

    /** A currency slot's key: [data]'s type, group and popup set with [slot]'s code and label. */
    private fun slot(data: KeyData, slot: KeyData) =
        TextKeyData(data.type, slot.code, slot.label, data.groupId, data.popup)

    private fun case(data: AutoTextKeyData) = with(platform) {
        val lower = codeText(data.code).lowercase(locale).codePointAt(0)
        val upper = codeText(data.code).uppercase(locale).codePointAt(0)
        TextKeyData(data.type, lower, data.label.lowercase(locale), data.groupId, data.popup) to
            TextKeyData(data.type, upper, data.label.uppercase(locale), data.groupId, data.popup)
    }

    private fun select(kind: Int, vararg children: AbstractKeyData?): KeyData? {
        val present = children.indices.fold(0) { bits, i -> if (children[i] != null) bits or (1 shl i) else bits }
        return children.getOrNull(KeyModels.select(kind, state.rawValue.toLong(), present))?.let(::compute)
    }

    /** `asString(isForDisplay = true)`. */
    fun display(data: KeyData): String = when (data) {
        is MultiTextKeyData -> KeyModels.multiText(data.codePoints, data.label, true)
        else -> KeyModels.text(data.code, data.label, true, platform.mark(data.code))
    }

    fun isSpace(data: KeyData) = KeyModels.isSpace(data.type.ordinal, data.code)

    /** `computeLabel`, with main's resource strings. */
    fun label(data: KeyData): String? {
        val fields = longArrayOf(data.type.ordinal.toLong(), data.code.toLong(), mode.toInt().toLong())
        return labelOf(data, InputRules.visual(fields))
    }

    private fun labelOf(data: KeyData, token: Int): String? = when (token) {
        0 -> null
        1 -> display(data)
        2 -> language
        else -> platform.string(referenceLabelResource(token))
    }

    /** `computeImageVector`, as the token of the icon it drew, 0 for none. */
    fun icon(data: KeyData): Int = InputRules.visual(longArrayOf(data.code.toLong(), state.rawValue.toLong(),
        facts.input.toLong(), facts.options.toLong(), mode.toInt().toLong())).takeIf { it in DRAWN } ?: 0

    private companion object {
        /** The tokens `computeImageVector`'s `when` drew an icon for. */
        val DRAWN = setOf(KeyCode.ARROW_LEFT, KeyCode.ARROW_RIGHT, KeyCode.ARROW_UP, KeyCode.ARROW_DOWN,
            KeyCode.CLIPBOARD_COPY, KeyCode.CLIPBOARD_CUT, KeyCode.CLIPBOARD_PASTE, KeyCode.CLIPBOARD_SELECT_ALL,
            KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP, KeyCode.COMPACT_LAYOUT_TO_LEFT, KeyCode.COMPACT_LAYOUT_TO_RIGHT,
            KeyCode.TOGGLE_COMPACT_LAYOUT, KeyCode.VOICE_INPUT, KeyCode.READ_ALOUD, KeyCode.IME_HIDE_UI, KeyCode.DELETE,
            InputProtocol.ICON_RETURN, InputProtocol.ICON_DONE, InputProtocol.ICON_ARROW, InputProtocol.ICON_SEARCH,
            InputProtocol.ICON_SEND, KeyCode.FORWARD_DELETE, KeyCode.IME_UI_MODE_MEDIA, KeyCode.IME_UI_MODE_CLIPBOARD,
            KeyCode.LANGUAGE_SWITCH, KeyCode.SETTINGS, InputProtocol.ICON_CAPS, KeyCode.SHIFT, KeyCode.SPACE,
            KeyCode.UNDO, KeyCode.REDO, KeyCode.TOGGLE_ACTIONS_OVERFLOW, KeyCode.TOGGLE_INCOGNITO_MODE,
            InputProtocol.ICON_INCOGNITO_OFF, KeyCode.TOGGLE_AUTOCORRECT, KeyCode.CHAR_WIDTH_FULL,
            KeyCode.CHAR_WIDTH_HALF, KeyCode.NOOP)
    }
}

/** One `TextKey`: its layout entry, and what main's compute left on it. */
internal class ReferenceKey(val data: AbstractKeyData) {
    var computedData: KeyData = TextKeyData.UNSPECIFIED
    var popups = ReferenceMutablePopupSet()
    var symbolHint: KeyData? = null
    var numberHint: KeyData? = null
    var isEnabled = true
    var isVisible = true
    /** Width factor, shrink and grow, as `TextKeyboard.layout` packs them. */
    var flex = FloatArray(3)
    /** Touch bounds, then visible bounds, as the last layout left them. */
    val bounds = FloatArray(8)
    var label: String? = null
    var hintedLabel: String? = null
    var icon = 0

    /** `TextKey.compute`. */
    fun compute(e: ReferenceEvaluator, keyboard: ReferenceKeyboard) {
        val computed = e.compute(data)
        if (computed == null || !e.evaluateVisible(computed)) {
            computedData = TextKeyData.UNSPECIFIED
            popups = ReferenceMutablePopupSet()
            isEnabled = false
            isVisible = false
            flex = FloatArray(3)
            return
        }
        computedData = computed
        val lists = PopupLists()
        computed.popup?.let { lists.merge(it, e, lists.relevant, true) }
        if (KeyModels.popupMode(keyboard.mode.value)) extended(computed, e, keyboard, lists)
        popups = lists.set()
        isEnabled = e.evaluateEnabled(computed)
        isVisible = true
        val f = KeyGeometry.flex(keyboard.mode.value, computed.code, computed.type == KeyType.NUMERIC)
        flex = floatArrayOf(f.widthFactor, f.shrink, f.grow)
    }

    private fun extended(computed: KeyData, e: ReferenceEvaluator, keyboard: ReferenceKeyboard, lists: PopupLists) {
        val computedLabel = computed.label.lowercase(e.locale)
        val extLabel = KeyModels.popupGroup(computed.groupId) ?: computedLabel
        val sources = KeyVariation.entries.flatMap { variation ->
            listOf(keyboard.mapping?.get(variation)?.get(extLabel), keyboard.fallback?.get(variation)?.get(extLabel))
        }
        val present = sources.indices.fold(0) { bits, i -> if (sources[i] != null) bits or (1 shl i) else bits }
        val popupSet = sources.getOrNull(KeyModels.popupSource(e.state.keyVariation.value, present))
        val keySpecific = if (extLabel != computedLabel) keyboard.all(computedLabel) else null
        keySpecific?.let { lists.merge(it, e, lists.relevant, true) }
        popupSet?.let { lists.merge(it, e, lists.relevant, true) }
        if (computed.type == KeyType.CHARACTER) {
            symbolHint?.let { hint ->
                lists.symbolHint = lists.hint(hint, computed.code, e, keyboard, lists.symbolPopups) ?: lists.symbolHint
            }
            numberHint?.let { hint ->
                lists.numberHint = lists.hint(hint, computed.code, e, keyboard, lists.numberPopups) ?: lists.numberHint
            }
        }
    }

    /** `computeLabelsAndDrawables`, for the hint settings [config]. */
    fun computeLabels(e: ReferenceEvaluator, config: KeyHintConfiguration) {
        label = e.label(computedData)
        hintedLabel = null
        icon = e.icon(computedData)
        val data = computedData
        if (data.type == KeyType.NUMERIC && e.mode == KeyboardMode.PHONE) {
            hintedLabel = KeyModels.phoneHint(data.code)
        } else if (!e.isSpace(data) || data.type == KeyType.NUMERIC) {
            val hint = popups.getPopupKeys(config).hint
            hintedLabel = if (hint != null && !e.isSpace(hint)) e.display(hint) else null
        }
    }
}

/** `MutablePopupSet`'s lists while a key computes, merged as `mergeInternal` merged them. */
internal class PopupLists {
    var main: KeyData? = null
    val relevant = arrayListOf<KeyData>()
    var symbolHint: KeyData? = null
    var numberHint: KeyData? = null
    val symbolPopups = arrayListOf<KeyData>()
    val numberPopups = arrayListOf<KeyData>()

    fun merge(
        other: PopupSet<AbstractKeyData>,
        e: ReferenceEvaluator,
        target: MutableList<KeyData>,
        useMain: Boolean = false,
    ) {
        other.relevant.forEach { e.compute(it)?.let(target::add) }
        other.main?.let { e.compute(it) }?.let { if (useMain && main == null) main = it else target.add(it) }
    }

    /** `addComputedHints` for one hint: the hint to set, or null to leave it. */
    fun hint(
        hint: KeyData,
        code: Int,
        e: ReferenceEvaluator,
        keyboard: ReferenceKeyboard,
        target: MutableList<KeyData>,
    ): KeyData? {
        val evaluated = e.compute(hint)
        if (hint.code == code) return null
        evaluated?.popup?.let { merge(it, e, target) }
        keyboard.all(hint.label)?.let { merge(it, e, target) }
        return evaluated
    }

    fun set() = ReferenceMutablePopupSet(main, relevant, symbolHint, numberHint, symbolPopups, numberPopups)
}

/** A `TextKeyboard`: its keys by row, and the popup mappings it was built with. */
internal class ReferenceKeyboard(val mode: KeyboardMode, val rows: List<List<ReferenceKey>>, val mapping: PopupMapping?,
                                 val fallback: PopupMapping?) {
    /** `TextKeyboardIterator`: the keys up to the first empty row. */
    val listed: List<ReferenceKey> = rows.takeWhile { it.isNotEmpty() }.flatten()

    fun all(label: String): PopupSet<AbstractKeyData>? =
        mapping?.get(KeyVariation.ALL)?.get(label) ?: fallback?.get(KeyVariation.ALL)?.get(label)

    /** Every key, listed or not, in row order: what `TextKeyboard.layout` lays out. */
    private val everyKey: List<ReferenceKey> = rows.flatten()
    private val rowSizes = IntArray(rows.size) { rows[it].size }

    /** `updateActiveEvaluators`' loop: each listed key computed, then its labels and drawables. */
    fun compute(e: ReferenceEvaluator, config: KeyHintConfiguration) {
        for (key in listed) {
            key.compute(e, this)
            key.computeLabels(e, config)
        }
    }

    /**
     * The renderer's `sizeDesiredKey` and `layout` for a keyboard [width] by [height]: the desired
     * key's visible size, then each listed key's touch and visible bounds, enabled state and code.
     * A layout that has nothing to lay out leaves each key's bounds as they were.
     */
    fun layout(width: Float, height: Float, rowHeight: Float, marginH: Float, marginV: Float): FloatArray {
        val desired = KeyGeometry.desiredKey(mode.value, width, height, rows.size, rowHeight, marginH, marginV)
        val flex = FloatArray(3 * everyKey.size)
        for ((i, key) in everyKey.withIndex()) key.flex.copyInto(flex, 3 * i)
        KeyGeometry.layout(rowSizes, flex, desired, width, height, true)?.let { bounds ->
            for ((i, key) in everyKey.withIndex()) bounds.copyInto(key.bounds, 0, 8 * i, 8 * i + 8)
        }
        val out = FloatArray(2 + 10 * listed.size)
        out[0] = desired[6] - desired[4]
        out[1] = desired[7] - desired[5]
        for ((i, key) in listed.withIndex()) {
            key.bounds.copyInto(out, 2 + 10 * i)
            out[2 + 10 * i + 8] = if (key.isEnabled) 1f else 0f
            out[2 + 10 * i + 9] = key.computedData.code.toFloat()
        }
        return out
    }

    companion object {
        /**
         * `mergeLayouts`' keys: [rows] merged, each a key, and on a characters keyboard each key's
         * number and symbol hints from [symbols], computed by [default] (`DefaultComputingEvaluator`).
         */
        fun build(mode: KeyboardMode, rows: LayoutArrangement, symbols: LayoutArrangement?, numberHints: Boolean,
                  default: ReferenceEvaluator, mapping: PopupMapping?, fallback: PopupMapping?): ReferenceKeyboard {
            val keys = rows.map { row -> row.map(::ReferenceKey) }
            if (mode == KeyboardMode.CHARACTERS && rows.isNotEmpty()) {
                val hints = hints(rows, symbols.orEmpty(), numberHints, default).iterator()
                for (key in keys.flatten()) {
                    val (number, symbol) = hints.next()
                    key.numberHint = number
                    key.symbolHint = symbol
                }
            }
            return ReferenceKeyboard(mode, keys, mapping, fallback)
        }

        /**
         * `ReferenceLayoutAssembly.hints`, with every symbols key computed by [default] first, as
         * main's `LayoutAssembly.hints` computed them, whether or not a hint uses it.
         */
        private fun hints(rows: LayoutArrangement, symbols: LayoutArrangement, numberHints: Boolean,
                          default: ReferenceEvaluator): List<Pair<KeyData?, KeyData?>> {
            val numbers = rows.map { arrayOfNulls<KeyData>(it.size) }
            val hints = rows.map { arrayOfNulls<KeyData>(it.size) }
            val evaluated = symbols.map { row -> row.map(default::compute) }
            fun addRow(r: Int, hintRow: List<KeyData?>, hintType: KeyType) {
                for (k in rows[r].indices) {
                    val hintKey = hintRow.getOrNull(k)
                    if (hintKey?.type != hintType) continue
                    if (hintType == KeyType.CHARACTER) hints[r][k] = hintKey else numbers[r][k] = hintKey
                }
            }
            if (numberHints && symbols.isNotEmpty()) addRow(0, evaluated[0], KeyType.NUMERIC)
            val offset = rows.size - symbols.size
            for (r in rows.indices) {
                if (r < offset) continue
                evaluated.getOrNull(r - offset)?.let { addRow(r, it, KeyType.CHARACTER) }
            }
            return rows.indices.flatMap { r -> rows[r].indices.map { k -> numbers[r][k] to hints[r][k] } }
        }
    }
}

/** Main's string resource for label resource [token], as `computeLabel` named it. */
internal fun referenceLabelResource(token: Int): Int = when (token) {
    KeyCode.PHONE_PAUSE -> R.string.key__phone_pause
    KeyCode.PHONE_WAIT -> R.string.key__phone_wait
    KeyCode.IME_UI_MODE_TEXT, KeyCode.VIEW_CHARACTERS -> R.string.key__view_characters
    KeyCode.VIEW_NUMERIC, KeyCode.VIEW_NUMERIC_ADVANCED -> R.string.key__view_numeric
    KeyCode.VIEW_PHONE -> R.string.key__view_phone
    KeyCode.VIEW_PHONE2 -> R.string.key__view_phone2
    KeyCode.VIEW_SYMBOLS -> R.string.key__view_symbols
    KeyCode.VIEW_SYMBOLS2 -> R.string.key__view_symbols2
    KeyCode.HALF_SPACE -> R.string.key__view_half_space
    KeyCode.KESHIDA -> R.string.key__view_keshida
    else -> error("Unknown native label resource")
}
