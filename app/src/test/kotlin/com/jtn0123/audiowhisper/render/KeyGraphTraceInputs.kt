package com.jtn0123.audiowhisper.render

import androidx.compose.ui.unit.LayoutDirection
import com.jtn0123.audiowhisper.rust.InputProtocol
import com.jtn0123.audiowhisper.rust.InputRules
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.core.SubtypeLayoutMap
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.keyboard.KEY_LABEL_RESOURCES
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.KeyboardState
import dev.patrickgold.florisboard.ime.keyboard.LayoutArrangementComponent
import dev.patrickgold.florisboard.ime.keyboard.LayoutAssembly
import dev.patrickgold.florisboard.ime.keyboard.LayoutFixtures
import dev.patrickgold.florisboard.ime.keyboard.LayoutType
import dev.patrickgold.florisboard.ime.keyboard.extCoreLayout
import dev.patrickgold.florisboard.ime.keyboard.extCorePopupMapping
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.lib.io.DefaultJsonConfig
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.io.File
import java.util.Locale
import kotlin.random.Random

/** The bundled layouts and popup mappings as Android reads them: each file's index entry and text. */
internal object KeyGraphAssets {
    private val root = File("src/main/assets/ime/keyboard")
    val fixtures = LayoutFixtures()

    /** Each indexed layout's index entry and text. */
    val layouts: Map<LayoutName, Pair<LayoutArrangementComponent, String>> = fixtures.layouts.associate {
        val text = File(root, "org.florisboard.layouts/${it.meta.arrangementFile(it.type)}").readText()
        (it.type to it.name) to (it.meta to text)
    }

    /** Each layout type's indexed names. */
    val names: Map<LayoutType, List<ExtensionComponentName>> = layouts.keys.groupBy({ it.first }, { it.second })

    /** Each indexed popup mapping's text, by its name. */
    val mappings: Map<ExtensionComponentName, String> = File(root, "org.florisboard.localization").let { dir ->
        DefaultJsonConfig.parseToJsonElement(File(dir, "extension.json").readText()).jsonObject
            .getValue("popupMappings").jsonArray.map { it.jsonObject.getValue("id").jsonPrimitive.content }
            .associate { extCorePopupMapping(it) to File(dir, "popupMappings/$it.json").readText() }
    }
}

/**
 * One keyboard as a subtype and the settings ask for it, and what goes wrong with its files: the
 * [broken] layout's text is cut in half, the [misnamed] layout's index entry names a modifier that
 * isn't indexed, and the [unnamed] one's names none. Main kept a keyboard per mode and subtype;
 * the trace keeps one per build, which includes the subtype's [locale].
 */
internal data class TraceBuild(
    val mode: KeyboardMode, val numberRow: Boolean, val numberHints: Boolean, val locale: Locale,
    val layouts: SubtypeLayoutMap, val mapping: ExtensionComponentName, val fallback: Boolean,
    val broken: LayoutName?, val misnamed: LayoutName?, val unnamed: LayoutName?,
    val synthetic: Synthetic? = null,
) {
    /** The files as Android reads them for this build. */
    fun files() = KeyGraphFiles(
        layouts::get,
        { name ->
            when (name) {
                misnamed -> extCoreLayout("not_indexed")
                unnamed -> null
                else -> synthetic?.modifiers?.get(name) ?: KeyGraphAssets.layouts[name]?.first?.modifier
            }
        },
        { name ->
            (synthetic?.layouts?.get(name) ?: KeyGraphAssets.layouts[name]?.second)
                ?.let { if (name == broken) it.substring(0, it.length / 2) else it }
        },
        synthetic?.mapping ?: KeyGraphAssets.mappings[mapping],
        if (fallback) KeyGraphAssets.mappings[extCorePopupMapping("default")] else null,
    )
}

/**
 * Files no bundled extension has: [layouts] by name, the modifier each main layout's entry names,
 * and a popup [mapping].
 */
internal data class Synthetic(
    val layouts: Map<LayoutName, String>,
    val modifiers: Map<LayoutName, ExtensionComponentName>,
    val mapping: String?,
)

/** Seeded builds and facts, each drawn from what Android can report and a little past it. */
internal class TraceInputs(private val random: Random) {
    private val presets = KeyGraphAssets.fixtures.presets
    private val currencies = KeyGraphAssets.fixtures.currencySets
    private val generator = SyntheticLayouts(random)

    fun build(): TraceBuild {
        val preset = presets.random(random)
        var layouts = preset.layouts
        repeat(random.nextInt(4)) {
            val type = MAPPED.random(random)
            val name = if (random.nextInt(25) == 0) {
                extCoreLayout("not_indexed")
            } else {
                KeyGraphAssets.names.getValue(type).random(random)
            }
            layouts = layouts.with(type, name)
        }
        val synthetic = if (random.nextInt(4) == 0) synthetic() else null
        synthetic?.layouts?.keys?.forEach { (type, name) -> if (type in MAPPED) layouts = layouts.with(type, name) }
        val named = { types: List<LayoutType> ->
            types.random(random).let { it to (layouts[it] ?: extCoreLayout("default")) }
        }
        return TraceBuild(
            MODES.random(random), random.nextBoolean(), random.nextBoolean(),
            if (random.nextInt(5) == 0) LOCALES.random(random) else preset.locale, layouts,
            when (random.nextInt(20)) {
                0 -> extCorePopupMapping("not_indexed")
                1, 2 -> KeyGraphAssets.mappings.keys.random(random)
                else -> preset.popups
            },
            random.nextInt(20) != 0,
            named(BROKEN).takeIf { random.nextInt(25) == 0 },
            named(NAMING).takeIf { random.nextInt(25) == 0 },
            named(NAMING).takeIf { random.nextInt(25) == 0 },
            synthetic,
        )
    }

    /** One to three synthetic layouts, sometimes naming a synthetic modifier, sometimes a synthetic popup mapping. */
    private fun synthetic(): Synthetic {
        val layouts = LinkedHashMap<LayoutName, String>()
        val modifiers = LinkedHashMap<LayoutName, ExtensionComponentName>()
        repeat(1 + random.nextInt(3)) {
            val type = if (random.nextBoolean()) LayoutType.CHARACTERS else MAPPED.random(random)
            val name = type to extCoreLayout("synthetic_${random.nextInt(1_000_000)}")
            layouts[name] = generator.layout()
            if (type in NAMING && random.nextBoolean()) {
                val modifier = extCoreLayout("synthetic_mod_${random.nextInt(1_000_000)}")
                modifiers[name] = modifier
                layouts[LayoutAssembly.modifier(type) to modifier] = generator.modifier()
            }
        }
        return Synthetic(layouts, modifiers, if (random.nextBoolean()) generator.mapping() else null)
    }

    fun facts(build: TraceBuild): KeyGraphFactsInput {
        val preset = presets.firstOrNull { it.locale == build.locale }
        val slots = when (random.nextInt(20)) {
            0 -> emptyList()
            1 -> currencies.values.random(random).take(random.nextInt(6))
            2, 3 -> currencies.values.random(random)
            else -> currencies[preset?.currency ?: "dollar"] ?: currencies.values.first()
        }
        return KeyGraphFactsInput(
            state(build.mode), input(), options(), random.nextBoolean(), random.nextInt(5), random.nextBoolean(),
            random.nextBoolean(), random.nextInt(4) == 0, random.nextBoolean(), random.nextInt(3),
            random.nextInt(4) == 0, random.nextInt(4), random.nextBoolean(), LANGUAGES.random(random),
            DEFAULT_LANGUAGES.random(random), hints(),
            slots, KEY_LABEL_RESOURCES.map { TRACE_PLATFORM.string(referenceLabelResource(it)) },
        )
    }

    fun hints() = KeyHintConfiguration(
        KeyHintMode.entries.random(random), KeyHintMode.entries.random(random), random.nextBoolean(),
    )

    /** A keyboard state as the evaluator got it: the active state with its shift reset for non-character modes. */
    private fun state(mode: KeyboardMode): ULong {
        val state = KeyboardState.new()
        if (random.nextInt(10) == 0) {
            state.rawValue = random.nextLong().toULong()
        } else {
            state.keyboardMode = if (random.nextInt(10) == 0) KeyboardMode.entries.random(random) else mode
            state.keyVariation = KeyVariation.entries.random(random)
            state.inputShiftState = InputShiftState.entries.random(random)
            state.imeUiMode = ImeUiMode.entries.random(random)
            state.layoutDirection = if (random.nextInt(4) == 0) LayoutDirection.Rtl else LayoutDirection.Ltr
            state.isSelectionMode = random.nextBoolean()
            state.isManualSelectionMode = random.nextInt(4) == 0
            state.isIncognitoMode = random.nextInt(4) == 0
            state.isActionsOverflowVisible = random.nextInt(4) == 0
            state.isComposingEnabled = random.nextBoolean()
            state.isCharHalfWidth = random.nextInt(4) == 0
        }
        val fields = longArrayOf(InputProtocol.CONTROL_EVALUATOR.toLong(), state.rawValue.toLong())
        return InputRules.control(fields)[0].toULong()
    }

    private fun input(): Int = when (random.nextInt(20)) {
        0 -> random.nextInt()
        else -> INPUTS.random(random) or (if (random.nextBoolean()) INPUT_FLAGS.random(random) else 0)
    }

    private fun options(): Int = when (random.nextInt(20)) {
        0 -> random.nextInt()
        else -> random.nextInt(8) or OPTION_FLAGS.filter { random.nextInt(5) == 0 }.fold(0, Int::or)
    }

    private fun SubtypeLayoutMap.with(type: LayoutType, name: ExtensionComponentName) = when (type) {
        LayoutType.CHARACTERS -> copy(characters = name)
        LayoutType.SYMBOLS -> copy(symbols = name)
        LayoutType.SYMBOLS2 -> copy(symbols2 = name)
        LayoutType.NUMERIC -> copy(numeric = name)
        LayoutType.NUMERIC_ADVANCED -> copy(numericAdvanced = name)
        LayoutType.NUMERIC_ROW -> copy(numericRow = name)
        LayoutType.PHONE -> copy(phone = name)
        else -> copy(phone2 = name)
    }

    companion object {
        /** The layout types a subtype names. */
        val MAPPED = listOf(LayoutType.CHARACTERS, LayoutType.SYMBOLS, LayoutType.SYMBOLS2, LayoutType.NUMERIC,
            LayoutType.NUMERIC_ADVANCED, LayoutType.NUMERIC_ROW, LayoutType.PHONE, LayoutType.PHONE2)
        val BROKEN = MAPPED + listOf(LayoutType.CHARACTERS_MOD, LayoutType.SYMBOLS_MOD, LayoutType.SYMBOLS2_MOD)
        val NAMING = listOf(LayoutType.CHARACTERS, LayoutType.SYMBOLS, LayoutType.SYMBOLS2)

        /** Characters most of the time, as typing is; every other mode, and the ones with no layouts. */
        val MODES = List(8) { KeyboardMode.CHARACTERS } + List(2) { KeyboardMode.SYMBOLS } +
            List(2) { KeyboardMode.SYMBOLS2 } + List(2) { KeyboardMode.NUMERIC } +
            List(2) { KeyboardMode.NUMERIC_ADVANCED } + List(2) { KeyboardMode.PHONE } +
            listOf(KeyboardMode.PHONE2, KeyboardMode.UNSPECIFIED, KeyboardMode.SMARTBAR_QUICK_ACTIONS,
                KeyboardMode.SMARTBAR_NUMBER_ROW)

        /** Locales with their own casing (Turkish, Azerbaijani, Lithuanian), right to left, and not Latin. */
        val LOCALES = listOf(
            "tr", "az", "lt", "ar", "he", "fa", "ur", "ru", "el", "hy", "ja", "zh-CN", "hi", "th", "nl", "de-CH",
        )
            .map(Locale::forLanguageTag) + Subtype.DEFAULT.primaryLocale.base

        val LANGUAGES = listOf("English (US)", "Deutsch", "Türkçe", "العربية", "עברית", "Русский", "日本語", "Ελληνικά",
            "हिन्दी", "", " ", "á", "😀")
        val DEFAULT_LANGUAGES = listOf("English (United States)", "English (US)", "Englisch (Vereinigte Staaten)")

        /** Text, email, URI, passwords, names, numbers, phone, dates and times, and no editor. */
        val INPUTS = listOf(0x1, 0x21, 0xd1, 0x11, 0x81, 0x91, 0xe1, 0x41, 0x61, 0x71, 0xa1, 0xb1, 0xc1,
            0x2, 0x1002, 0x2002, 0x3002, 0x12, 0x3, 0x4, 0x14, 0x24, 0x0)
        val INPUT_FLAGS = listOf(0x1000, 0x2000, 0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000)

        /**
         * No enter action, no personalized learning, navigate next and previous, no extract UI, no
         * fullscreen, force ASCII.
         */
        val OPTION_FLAGS = listOf(0x40000000, 0x1000000, 0x8000000, 0x4000000, 0x10000000, 0x2000000, Int.MIN_VALUE)
    }
}
