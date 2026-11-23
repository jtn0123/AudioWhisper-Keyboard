package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.LayoutCache
import dev.patrickgold.florisboard.ime.core.SubtypeLayoutMap
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class LayoutAssemblyMatchTest {
    private val fixtures = LayoutFixtures()
    private var cases = 0
    private var evaluatedKeys = 0
    private val defaultEvaluator = fixtures.evaluator(0)

    @Test fun everyBundledLayoutModeAndRetainedPresetMatches() {
        val random = Random(20261113)
        assertEquals(46, fixtures.presets.size)
        assertEquals(fixtures.bundledFiles, fixtures.indexedFiles)
        assertTrue(fixtures.layouts.size >= 93)
        fixtures.presets.forEach { preset ->
            val prepared = LayoutFixtures.Prepared(preset.locale)
            for (layout in fixtures.layouts) for (mode in KeyboardMode.entries) for (numberRow in listOf(false, true)) {
                compare(preset, prepared, mode, numberRow, true, layout, preset.layouts, random)
            }
        }
        println("Layout assembly: ${fixtures.bundledFiles.size} bundled files, ${fixtures.layouts.size} indexed layouts, ${fixtures.presets.size} presets, $cases complete keyboards, $evaluatedKeys evaluated keys")
    }

    @Test fun seededMissingReferencesEmptyRowsSelectorsAndHintSettingsMatch() {
        val random = Random(2098)
        repeat(4_000) {
            val preset = fixtures.presets.random(random)
            val prepared = LayoutFixtures.Prepared(preset.locale)
            val type = LayoutType.entries.random(random)
            val missing = ExtensionComponentName("missing-extension", if (it % 2 == 0) "" else "missing-layout")
            val map = preset.layouts.copy(type, missing) ?: preset.layouts
            compare(preset, prepared, KeyboardMode.entries.random(random), random.nextBoolean(), random.nextBoolean(),
                null, map, random)
            fun rows(): LayoutArrangement? = if (random.nextInt(5) == 0) null else List(random.nextInt(6)) {
                List(random.nextInt(12)) {
                    when (random.nextInt(5)) {
                        0 -> TextKeyData(code = 0)
                        1 -> CaseSelector(TextKeyData(code = 0), TextKeyData(code = 42))
                        2 -> VariationSelector()
                        else -> TextKeyData(code = random.nextInt(-900, 13000), label = "fixture")
                    }
                }
            }
            val main = rows(); val mod = rows(); val extension = rows()
            val expected = ReferenceLayoutAssembly.merge(main, mod, extension)
            val actual = LayoutAssembly.merge(main, mod, extension)
            assertEquals(expected, actual, "seeded merge $it")
            val symbols = rows().orEmpty()
            assertEquals(ReferenceLayoutAssembly.hints(KeyboardMode.CHARACTERS, true, expected, symbols, defaultEvaluator),
                LayoutAssembly.hints(KeyboardMode.CHARACTERS, true, actual, symbols, defaultEvaluator))
        }
        println("Layout assembly random: $cases complete keyboards, $evaluatedKeys evaluated keys, 4000 malformed/empty arrangement sets")
    }

    private fun compare(preset: LayoutFixtures.Preset, prepared: LayoutFixtures.Prepared, mode: KeyboardMode,
                        numberRow: Boolean, numberHints: Boolean, candidate: LayoutFixtures.Layout?, map: SubtypeLayoutMap, random: Random) {
        val getLayout = { type: LayoutType -> if (candidate?.type == type && map[type] != null) candidate.name else map[type] }
        fun assemble(mode: KeyboardMode, reference: Boolean): LayoutArrangement {
            val plan = if (reference) ReferenceLayoutAssembly.plan(mode, numberRow, getLayout) else LayoutAssembly.plan(mode, numberRow, getLayout)
            assertEquals(ReferenceLayoutAssembly.plan(mode, numberRow, getLayout), plan)
            val main = fixtures.load(plan[0], candidate)
            val modifier = main?.meta?.modifier?.let {
                val type = if (reference) ReferenceLayoutAssembly.modifier(main.type) else LayoutAssembly.modifier(main.type)
                type to it
            } ?: plan[1]
            val mod = fixtures.load(modifier, candidate)
            val extension = fixtures.load(plan[2], candidate)
            val a = prepared.rows(main?.rows); val b = prepared.rows(mod?.rows); val c = prepared.rows(extension?.rows)
            return if (reference) ReferenceLayoutAssembly.merge(a, b, c) else LayoutAssembly.merge(a, b, c)
        }
        val expected = assemble(mode, true); val actual = assemble(mode, false)
        assertEquals(expected, actual, "arrangement case $cases")
        val symbolsExpected = assemble(KeyboardMode.SYMBOLS, true)
        val symbolsActual = assemble(KeyboardMode.SYMBOLS, false)
        val hintsExpected = ReferenceLayoutAssembly.hints(mode, numberHints, expected, symbolsExpected, defaultEvaluator)
        val hintsActual = LayoutAssembly.hints(mode, numberHints, actual, symbolsActual, defaultEvaluator)
        assertEquals(hintsExpected, hintsActual, "hints case $cases")
        val raw = (random.nextInt(4).toLong() shl 8) or (random.nextInt(5).toLong() shl 4) or
            (random.nextInt(2).toLong() shl 27) or (random.nextInt(2).toLong() shl 21)
        val evaluator = fixtures.evaluator(raw, preset.currency)
        val mappings = listOf(fixtures.popupMappings[preset.popups.componentId], fixtures.popupMappings["default"])
        val config = KeyHintConfiguration(KeyHintMode.entries.random(random), KeyHintMode.entries.random(random), random.nextBoolean())
        val width = if (random.nextBoolean()) 1080f else 2340f
        val height = random.nextInt(250, 1000).toFloat()
        val a = LayoutSnapshot.evaluate(expected, hintsExpected, mode, evaluator, mappings, prepared, preset.locale, config, true, width, height)
        val b = LayoutSnapshot.evaluate(actual, hintsActual, mode, evaluator, mappings, prepared, preset.locale, config, false, width, height)
        assertEquals(a, b, "evaluated keyboard case $cases")
        evaluatedKeys += a.size; cases++
    }

    @Test fun cacheOperationsAndHashCollisionsMatch() {
        // The reference's answers also go on a golden tape (docs/phase6-kotlin-references.md).
        val tape = GoldenTape.ofReference("layout-cache")
        val random = Random(20261112)
        var checks = 0
        repeat(200) {
            val reference = ReferenceTextKeyboardCache<Int>()
            val actual = LayoutCache<Int>()
            val hashes = intArrayOf(Int.MIN_VALUE, Int.MAX_VALUE, -1, 0, 1, random.nextInt())
            repeat(200) { value ->
                val mode = KeyboardMode.entries.random(random); val hash = hashes.random(random)
                when (random.nextInt(8)) {
                    0 -> { reference.clear(); actual.clear() }
                    1 -> { reference.clear(mode); actual.clearMode(mode.value) }
                    2 -> { reference.clear(hash); actual.clearSubtype(hash) }
                    3 -> { reference.clear(mode, hash); actual.clear(mode.value, hash) }
                    else -> { reference.set(mode, hash, value); actual.set(mode.value, hash, value) }
                }
                for (m in KeyboardMode.entries) for (h in hashes) {
                    tape.answer("get", reference.get(m, h))
                    assertEquals(reference.get(m, h), actual.get(m.value, h)); checks++
                }
            }
        }
        tape.finish()
        println("Layout cache: 40000 operations, $checks lookups, including signed hashes and collisions")
    }
}

/** [this] with [componentName] for [layoutType], or null for a type a subtype doesn't name. */
private fun SubtypeLayoutMap.copy(layoutType: LayoutType, componentName: ExtensionComponentName): SubtypeLayoutMap? =
    when (layoutType) {
        LayoutType.CHARACTERS -> copy(characters = componentName)
        LayoutType.SYMBOLS -> copy(symbols = componentName)
        LayoutType.SYMBOLS2 -> copy(symbols2 = componentName)
        LayoutType.NUMERIC -> copy(numeric = componentName)
        LayoutType.NUMERIC_ADVANCED -> copy(numericAdvanced = componentName)
        LayoutType.NUMERIC_ROW -> copy(numericRow = componentName)
        LayoutType.PHONE -> copy(phone = componentName)
        LayoutType.PHONE2 -> copy(phone2 = componentName)
        else -> null
    }
