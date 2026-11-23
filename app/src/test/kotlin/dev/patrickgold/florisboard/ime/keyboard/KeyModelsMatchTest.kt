package dev.patrickgold.florisboard.ime.keyboard

import androidx.compose.ui.unit.LayoutDirection
import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.lib.io.DefaultJsonConfig
import kotlinx.serialization.json.*
import java.io.File
import java.lang.reflect.Modifier
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

/**
 * Key state, codes, predefined keys, key rules, availability and currency slots in Rust (through
 * [KeyModels], [KeyboardState], [KeyCode], [TextKeyData] and [CurrencySet]) give the answers the
 * Kotlin they replaced gave (`ReferenceKeyboardState`, `ReferenceKeyCodes`, `ReferenceKeyMetadata`
 * and `ReferenceKeyRules`), checked against golden digests of them (docs/phase6-kotlin-references.md).
 */
class KeyModelsMatchTest {
    @Test fun stateMatches() {
        GoldenTape.check("key-state-registers") { tape -> registers(tape) }
        assertFailsWith<IllegalArgumentException> { KeyModels.read(0, -1) }
        assertFailsWith<IllegalArgumentException> { KeyModels.write(0, 15, 0) }
    }

    private fun registers(tape: GoldenTape) {
        val random = Random(20261112)
        // All low registers, including invalid enum regions, then arbitrary upper bits.
        for (raw in (0L..65535L).asSequence() + generateSequence { random.nextLong() }.take(20_000)) {
            val actual = KeyboardState.new(raw.toULong())
            tape.answer("state", state(actual))
            val field = random.nextInt(15)
            val value = random.nextInt()
            val changed = KeyModels.write(raw, field, value).toULong()
            when (field) {
                0 -> actual.keyboardMode = KeyboardMode.fromInt(value and 15)
                1 -> actual.keyVariation = KeyVariation.fromInt(value and 15)
                2 -> actual.inputShiftState = InputShiftState.fromInt(value and 3)
                3 -> actual.imeUiMode = ImeUiMode.fromInt(value and 7)
                else -> Unit
            }
            // Raw JNI setters mask the supplied integer before enum fallback. Test the public
            // setters separately, and raw fields against the original register arithmetic.
            val masks = longArrayOf(15, 15, 3, 7, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1)
            val offsets = intArrayOf(0, 4, 8, 24, 27, 10, 11, 12, 13, 15, 16, 17, 32, 20, 21)
            val mask = masks[field].toULong(); val offset = offsets[field]
            val original = (raw.toULong() and (mask shl offset).inv()) or ((value.toULong() and mask) shl offset)
            assertEquals(original, changed)
            tape.answer("changed", state(actual))
        }
    }

    @Test fun publicStateSettersMatch() = GoldenTape.check("key-state-setters") { tape ->
        val random = Random(90112)
        repeat(2_000) {
            val raw = random.nextLong().toULong()
            val a = KeyboardState.new(raw)
            repeat(40) {
                val value = random.nextBoolean()
                when (random.nextInt(19)) {
                    0 -> a.keyboardMode = KeyboardMode.entries.random(random)
                    1 -> a.keyVariation = KeyVariation.entries.random(random)
                    2 -> a.inputShiftState = InputShiftState.entries.random(random)
                    3 -> a.imeUiMode = ImeUiMode.entries.random(random)
                    4 -> a.layoutDirection = LayoutDirection.entries.random(random)
                    5 -> a.isSelectionMode = value
                    6 -> a.isManualSelectionMode = value
                    7 -> a.isManualSelectionModeStart = value
                    8 -> a.isManualSelectionModeEnd = value
                    9 -> a.isIncognitoMode = value
                    10 -> a.isActionsOverflowVisible = value
                    11 -> a.isActionsEditorVisible = value
                    12 -> a.isSubtypeSelectionVisible = value
                    13 -> a.isComposingEnabled = value
                    14 -> a.isCharHalfWidth = value
                    15 -> a.isCursorMode = value
                    16 -> a.rawValue = raw
                    17 -> tape.answer("snapshot", a.snapshot().rawValue)
                    18 -> { tape.answer("hashCode", a.hashCode()); tape.answer("toString", a.toString()) }
                }
                tape.answer("rawValue", a.rawValue); tape.answer("state", state(a))
            }
        }
    }

    @Test fun keyRulesMatch() = GoldenTape.check("key-rules") { tape ->
        val random = Random(20261112)
        tape.answer("currencyDefaults", KeyModels.currencyDefaults().toList())
        // By name, so the questions don't hang on the order reflection lists fields in.
        val codeFields = (KeyCode::class.java.declaredFields + KeyCode.Spec::class.java.declaredFields)
            .filter { it.type == Int::class.javaPrimitiveType && '$' !in it.name }.sortedBy { it.name }
        tape.answer("key codes", codeFields.map { it.name })
        for (field in codeFields) {
            field.isAccessible = true
            tape.answer(field.name, field.getInt(null))
        }
        tape.answer("predefined", KeyModels.predefined().toList())
        val keys = TextKeyData::class.java.declaredFields
            .filter { it.type == TextKeyData::class.java && Modifier.isStatic(it.modifiers) }.sortedBy { it.name }
        tape.answer("predefined keys", keys.map { it.name })
        for (field in keys) {
            field.isAccessible = true
            val data = field.get(null) as TextKeyData
            tape.answer(field.name, Triple(data.code, data.type, data.label))
        }
        val codes = (-10_000..13_000).asSequence() + sequenceOf(Int.MIN_VALUE, Int.MAX_VALUE) +
            generateSequence { random.nextInt() }.take(100_000)
        for (code in codes) {
            tape.answer("currency", CurrencySet.isCurrencySlot(code))
            tape.answer("currencySlot", KeyModels.currencySlot(code))
            tape.answer("codeInfo", TextKeyData.getCodeInfoAsTextKeyData(code)?.let { Triple(it.type, it.code, it.label) })
            tape.answer("phoneHint", KeyModels.phoneHint(code))
            val type = KeyType.entries.random(random)
            tape.answer("space", TextKeyData(type, code).isSpaceKey())
            val label = if (random.nextBoolean()) "◌label" else "label\uD800"
            val display = random.nextBoolean(); val mark = random.nextBoolean()
            tape.answer("text", KeyModels.text(code, label, display, mark))
            val points = intArrayOf(code, random.nextInt(0x110000), 0xd800, -1)
            tape.answer("multi", KeyModels.multiText(points, label, display))
        }
        for (mode in -4..20) tape.answer("popupMode", KeyModels.popupMode(mode))
        for (group in -10..10) tape.answer("popupGroup", KeyModels.popupGroup(group))
        for (variation in 0..4) for (mask in 0..1023) tape.answer("popupSource", KeyModels.popupSource(variation, mask))
    }

    @Test fun availabilityMatches() {
        var cases = 0
        GoldenTape.check("key-availability") { tape ->
            for (code in -10_000..13_000) {
                for (a in 0..2) for (b in 0..1) {
                    tape.answer("enabled", KeyModels.enabled(code, a, b))
                    cases++
                }
                for (enabled in listOf(false, true)) for (action in 0..4) for (language in listOf(false, true)) {
                    tape.answer("visible", KeyModels.visible(code, enabled, action, language))
                    cases++
                }
            }
        }
        println("Key availability: $cases cases")
    }

    @Test fun currencyObjectsMatch() = GoldenTape.check("key-currency-objects") { tape ->
        val random = Random(98112)
        repeat(100) { length ->
            val slots = List(length) { TextKeyData(code = random.nextInt(), label = "slot$it") }
            val set = CurrencySet("test", "test", slots)
            for (code in (-1000..1000) + listOf(Int.MIN_VALUE, Int.MAX_VALUE)) {
                val slot = KeyModels.currencySlot(code)
                tape.answer("slot", slot)
                assertEquals(slots.getOrNull(slot), set.getSlot(code))
            }
        }
    }

    @Test fun bundledCurrencySetsMatch() {
        val json = DefaultJsonConfig.parseToJsonElement(File(
            "src/main/assets/ime/keyboard/org.florisboard.currencysets/extension.json").readText()).jsonObject
        val sets = json.getValue("currencySets").jsonArray
        GoldenTape.check("key-currency-bundled") { tape ->
            for (node in sets) {
                val set = DefaultJsonConfig.decodeFromJsonElement<CurrencySet>(node)
                val slots = node.jsonObject.getValue("slots").jsonArray.map {
                    DefaultJsonConfig.decodeFromJsonElement<TextKeyData>(it)
                }
                for (code in -1000..1000) {
                    val slot = KeyModels.currencySlot(code)
                    tape.answer("slot", slot)
                    assertEquals(slots.getOrNull(slot), set.getSlot(code))
                }
            }
            for (type in KeyType.entries) for (code in listOf(0, 31, 32, 33, 1600, 8204, 12288, -801)) {
                tape.answer("space", TextKeyData(type, code).isSpaceKey())
            }
        }
        println("Bundled currency sets: ${sets.size}, ${sets.size * 2001} lookups; 80 space/type cases")
    }

    private fun state(s: KeyboardState): List<Any> = listOf(s.keyboardMode, s.keyVariation, s.inputShiftState,
        s.imeUiMode, s.layoutDirection, s.isSelectionMode, s.isManualSelectionMode, s.isManualSelectionModeStart,
        s.isManualSelectionModeEnd, s.isIncognitoMode, s.isActionsOverflowVisible, s.isActionsEditorVisible,
        s.isSubtypeSelectionVisible, s.isComposingEnabled, s.isCharHalfWidth, s.isLowercase, s.isUppercase, s.isCursorMode)
}
