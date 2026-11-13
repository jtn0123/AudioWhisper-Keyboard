package dev.patrickgold.florisboard.ime.keyboard

import androidx.compose.ui.unit.LayoutDirection
import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.lib.io.DefaultJsonConfig
import kotlinx.serialization.json.*
import java.io.File
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class KeyModelsMatchTest {
    @Test fun stateMatches() {
        val random = Random(20261112)
        // All low registers, including invalid enum regions, then arbitrary upper bits.
        for (raw in (0L..65535L).asSequence() + generateSequence { random.nextLong() }.take(20_000)) {
            val expected = ReferenceKeyboardState.new(raw.toULong())
            val actual = KeyboardState.new(raw.toULong())
            assertEquals(state(expected), state(actual))
            val field = random.nextInt(15)
            val value = random.nextInt()
            val changed = KeyModels.write(raw, field, value).toULong()
            when (field) {
                0 -> expected.keyboardMode = KeyboardMode.fromInt(value and 15)
                1 -> expected.keyVariation = KeyVariation.fromInt(value and 15)
                2 -> expected.inputShiftState = InputShiftState.fromInt(value and 3)
                3 -> expected.imeUiMode = ImeUiMode.fromInt(value and 7)
                else -> Unit
            }
            // Raw JNI setters mask the supplied integer before enum fallback. Test the public
            // setters separately, and raw fields against the original register arithmetic.
            val masks = longArrayOf(15, 15, 3, 7, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1)
            val offsets = intArrayOf(0, 4, 8, 24, 27, 10, 11, 12, 13, 15, 16, 17, 32, 20, 21)
            val mask = masks[field].toULong(); val offset = offsets[field]
            val original = (raw.toULong() and (mask shl offset).inv()) or ((value.toULong() and mask) shl offset)
            assertEquals(original, changed)
            actual.keyboardMode = expected.keyboardMode
            actual.keyVariation = expected.keyVariation
            actual.inputShiftState = expected.inputShiftState
            actual.imeUiMode = expected.imeUiMode
            assertEquals(state(expected), state(actual))
        }
        assertFailsWith<IllegalArgumentException> { KeyModels.read(0, -1) }
        assertFailsWith<IllegalArgumentException> { KeyModels.write(0, 15, 0) }
    }

    @Test fun publicStateSettersMatch() {
        val random = Random(90112)
        repeat(2_000) {
            val raw = random.nextLong().toULong()
            val a = KeyboardState.new(raw); val e = ReferenceKeyboardState.new(raw)
            repeat(40) {
                val value = random.nextBoolean()
                when (random.nextInt(19)) {
                    0 -> { val v = KeyboardMode.entries.random(random); a.keyboardMode = v; e.keyboardMode = v }
                    1 -> { val v = KeyVariation.entries.random(random); a.keyVariation = v; e.keyVariation = v }
                    2 -> { val v = InputShiftState.entries.random(random); a.inputShiftState = v; e.inputShiftState = v }
                    3 -> { val v = ImeUiMode.entries.random(random); a.imeUiMode = v; e.imeUiMode = v }
                    4 -> { val v = LayoutDirection.entries.random(random); a.layoutDirection = v; e.layoutDirection = v }
                    5 -> { a.isSelectionMode = value; e.isSelectionMode = value }
                    6 -> { a.isManualSelectionMode = value; e.isManualSelectionMode = value }
                    7 -> { a.isManualSelectionModeStart = value; e.isManualSelectionModeStart = value }
                    8 -> { a.isManualSelectionModeEnd = value; e.isManualSelectionModeEnd = value }
                    9 -> { a.isIncognitoMode = value; e.isIncognitoMode = value }
                    10 -> { a.isActionsOverflowVisible = value; e.isActionsOverflowVisible = value }
                    11 -> { a.isActionsEditorVisible = value; e.isActionsEditorVisible = value }
                    12 -> { a.isSubtypeSelectionVisible = value; e.isSubtypeSelectionVisible = value }
                    13 -> { a.isComposingEnabled = value; e.isComposingEnabled = value }
                    14 -> { a.isCharHalfWidth = value; e.isCharHalfWidth = value }
                    15 -> { a.isCursorMode = value; e.isCursorMode = value }
                    16 -> { a.rawValue = raw; e.rawValue = raw }
                    17 -> assertEquals(e.snapshot().rawValue, a.snapshot().rawValue)
                    18 -> { assertEquals(e.hashCode(), a.hashCode()); assertEquals(e.toString(), a.toString()) }
                }
                assertEquals(e.rawValue, a.rawValue)
                assertEquals(state(e), state(a))
            }
        }
    }

    @Test fun keyRulesMatch() {
        val random = Random(20261112)
        assertContentEquals(ReferenceKeyRules.currencyDefaults, KeyModels.currencyDefaults())
        ReferenceKeyCodes::class.java.declaredFields.forEach { field ->
            if (field.type == Int::class.javaPrimitiveType) {
                val actual = dev.patrickgold.florisboard.ime.text.key.KeyCode::class.java.declaredFields
                    .firstOrNull { it.name == field.name }
                    ?: dev.patrickgold.florisboard.ime.text.key.KeyCode.Spec::class.java.getDeclaredField(field.name)
                actual.isAccessible = true
                assertEquals(field.getInt(null), actual.getInt(null))
            }
        }
        val records = KeyModels.predefined().toList()
        assertEquals(ReferenceKeyMetadata.entries.map { "${it.code},${it.type.ordinal},${it.label}" }, records)
        ReferenceKeyMetadata.entries.forEach { entry ->
            val data = TextKeyData::class.java.getDeclaredField(entry.name).let { field ->
                field.isAccessible = true; field.get(null) as TextKeyData
            }
            assertEquals(entry.code, data.code); assertEquals(entry.type, data.type); assertEquals(entry.label, data.label)
        }
        val codes = (-10_000..13_000).asSequence() + sequenceOf(Int.MIN_VALUE, Int.MAX_VALUE) +
            generateSequence { random.nextInt() }.take(100_000)
        for (code in codes) {
            assertEquals(ReferenceKeyRules.currency(code), CurrencySet.isCurrencySlot(code))
            assertEquals(ReferenceKeyRules.currencySlot(code), KeyModels.currencySlot(code))
            val expected = if (code <= 0) ReferenceKeyMetadata.entries.firstOrNull { it.discoverable && it.code == code }
                ?.let { Triple(it.type, it.code, it.label) } else Triple(KeyType.CHARACTER, code,
                    buildString { try { appendCodePoint(code) } catch (_: Throwable) { } })
            val actual = TextKeyData.getCodeInfoAsTextKeyData(code)?.let { Triple(it.type, it.code, it.label) }
            assertEquals(expected, actual)
            assertEquals(ReferenceKeyRules.phoneHint(code), KeyModels.phoneHint(code))
            val type = KeyType.entries.random(random)
            assertEquals(ReferenceKeyRules.space(type, code), TextKeyData(type, code).isSpaceKey())
            val label = if (random.nextBoolean()) "◌label" else "label\uD800"
            val display = random.nextBoolean(); val mark = random.nextBoolean()
            assertEquals(ReferenceKeyRules.text(code, label, display, mark), KeyModels.text(code, label, display, mark))
            val points = intArrayOf(code, random.nextInt(0x110000), 0xd800, -1)
            assertEquals(ReferenceKeyRules.multi(points, label, display), KeyModels.multiText(points, label, display))
        }
        for (mode in -4..20) assertEquals(ReferenceKeyRules.popupMode(mode), KeyModels.popupMode(mode))
        for (group in -10..10) assertEquals(ReferenceKeyRules.popupGroup(group), KeyModels.popupGroup(group))
        for (variation in 0..4) for (mask in 0..1023) {
            assertEquals(ReferenceKeyRules.popupSource(variation, mask), KeyModels.popupSource(variation, mask))
        }
    }

    @Test fun availabilityMatches() {
        var cases = 0
        for (code in -10_000..13_000) {
            for (a in 0..2) for (b in 0..1) {
                assertEquals(ReferenceKeyRules.enabled(code, a, b), KeyModels.enabled(code, a, b))
                cases++
            }
            for (enabled in listOf(false, true)) for (action in 0..4) for (language in listOf(false, true)) {
                assertEquals(ReferenceKeyRules.visible(code, enabled, action, language), KeyModels.visible(code, enabled, action, language))
                cases++
            }
        }
        println("Key availability: $cases cases")
    }

    @Test fun currencyObjectsMatch() {
        val random = Random(98112)
        repeat(100) { length ->
            val slots = List(length) { TextKeyData(code = random.nextInt(), label = "slot$it") }
            val set = CurrencySet("test", "test", slots)
            for (code in -1000..1000) assertEquals(slots.getOrNull(ReferenceKeyRules.currencySlot(code)), set.getSlot(code))
            for (code in listOf(Int.MIN_VALUE, Int.MAX_VALUE))
                assertEquals(slots.getOrNull(ReferenceKeyRules.currencySlot(code)), set.getSlot(code))
        }
    }

    @Test fun bundledCurrencySetsMatch() {
        val json = DefaultJsonConfig.parseToJsonElement(File(
            "src/main/assets/ime/keyboard/org.florisboard.currencysets/extension.json").readText()).jsonObject
        val sets = json.getValue("currencySets").jsonArray
        for (node in sets) {
            val set = DefaultJsonConfig.decodeFromJsonElement<CurrencySet>(node)
            val slots = node.jsonObject.getValue("slots").jsonArray.map {
                DefaultJsonConfig.decodeFromJsonElement<TextKeyData>(it)
            }
            for (code in -1000..1000) assertEquals(slots.getOrNull(ReferenceKeyRules.currencySlot(code)), set.getSlot(code))
        }
        for (type in KeyType.entries) for (code in listOf(0, 31, 32, 33, 1600, 8204, 12288, -801))
            assertEquals(ReferenceKeyRules.space(type, code), TextKeyData(type, code).isSpaceKey())
        println("Bundled currency sets: ${sets.size}, ${sets.size * 2001} lookups; 80 space/type cases")
    }

    private fun state(s: KeyboardState): List<Any> = listOf(s.keyboardMode, s.keyVariation, s.inputShiftState,
        s.imeUiMode, s.layoutDirection, s.isSelectionMode, s.isManualSelectionMode, s.isManualSelectionModeStart,
        s.isManualSelectionModeEnd, s.isIncognitoMode, s.isActionsOverflowVisible, s.isActionsEditorVisible,
        s.isSubtypeSelectionVisible, s.isComposingEnabled, s.isCharHalfWidth, s.isLowercase, s.isUppercase, s.isCursorMode)
    private fun state(s: ReferenceKeyboardState): List<Any> = listOf(s.keyboardMode, s.keyVariation, s.inputShiftState,
        s.imeUiMode, s.layoutDirection, s.isSelectionMode, s.isManualSelectionMode, s.isManualSelectionModeStart,
        s.isManualSelectionModeEnd, s.isIncognitoMode, s.isActionsOverflowVisible, s.isActionsEditorVisible,
        s.isSubtypeSelectionVisible, s.isComposingEnabled, s.isCharHalfWidth, s.isLowercase, s.isUppercase, s.isCursorMode)
}
