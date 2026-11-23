package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.golden.GoldenTape
import dev.patrickgold.florisboard.ime.text.keyboard.AutoTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.MultiTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.io.DefaultJsonConfig
import kotlinx.serialization.json.*
import java.io.File
import java.lang.reflect.Proxy
import java.util.Locale
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** The reference's answers also go on golden tapes (docs/phase6-kotlin-references.md). */
class KeySelectorsMatchTest {
    @Test fun bundledLayoutsAndPopupsMatchInEverySelectorState() {
        val tape = GoldenTape.ofReference("key-selectors-bundled")
        val root = File("src/main/assets/ime/keyboard")
        val files = root.walkTopDown().filter { it.isFile && it.extension == "json" }.sortedBy { it.path }.toList()
        assertTrue(files.isNotEmpty())
        var nodes = 0; var checks = 0; var layouts = 0
        files.forEach { file ->
            if ("/layouts/" in file.path) layouts++
            keyNodes(DefaultJsonConfig.parseToJsonElement(file.readText())).forEach { node ->
                val data = prepare(DefaultJsonConfig.decodeFromJsonElement<AbstractKeyData>(node))
                val reference = reference(data)
                nodes++
                for (shift in 0..3) for (variation in 0..4) for (direction in 0..1) for (width in 0..1) {
                    val raw = (shift.toLong() shl 8) or (variation.toLong() shl 4) or
                        (direction.toLong() shl 27) or (width.toLong() shl 21)
                    val evaluator = evaluator(raw)
                    tape.answer("compute", content(reference.compute(evaluator)))
                    assertEquals(signature(reference.compute(evaluator)), signature(data.compute(evaluator)),
                        "bundled key $nodes state $raw")
                    checks++
                }
            }
        }
        tape.finish()
        assertTrue(layouts > 50); assertTrue(nodes > 1_000)
        println("Key selectors: $layouts bundled layouts, ${files.size} JSON files, $nodes nodes, $checks state cases")
    }

    @Test fun seededNestedSelectorsAndNullFallbacksMatch() {
        val tape = GoldenTape.ofReference("key-selectors-seeded")
        val random = Random(20261112)
        repeat(40_000) {
            val data = randomNode(random, 3)
            val evaluator = evaluator(random.nextLong())
            tape.answer("compute", content(reference(data).compute(evaluator)))
            assertEquals(signature(reference(data).compute(evaluator)), signature(data.compute(evaluator)))
        }
        tape.finish()
    }

    private fun keyNodes(value: JsonElement): Sequence<JsonObject> = sequence {
        when (value) {
            is JsonObject -> {
                val kind = value["$"]?.jsonPrimitive?.contentOrNull
                if ("code" in value || "codePoints" in value || kind in setOf("text_key", "auto_text_key",
                        "multi_text_key", "case_selector", "shift_state_selector", "variation_selector",
                        "layout_direction_selector", "char_width_selector")) yield(value)
                value.values.forEach { yieldAll(keyNodes(it)) }
            }
            is JsonArray -> value.forEach { yieldAll(keyNodes(it)) }
            else -> Unit
        }
    }

    // Android ICU casing/cache remains unchanged in Kotlin. Supply both evaluators the same
    // casing snapshots on the JVM; emulator smoke checks the actual Android AutoTextKeyData path.
    private fun prepare(data: AbstractKeyData): AbstractKeyData = when (data) {
        is AutoTextKeyData -> CaseSelector(
            TextKeyData(data.type, data.code, data.label.lowercase(Locale.ROOT), data.groupId, data.popup),
            TextKeyData(data.type, data.code, data.label.uppercase(Locale.ROOT), data.groupId, data.popup))
        is CaseSelector -> CaseSelector(prepare(data.lower), prepare(data.upper))
        is ShiftStateSelector -> ShiftStateSelector(data.unshifted?.let(::prepare), data.shifted?.let(::prepare),
            data.shiftedManual?.let(::prepare), data.shiftedAutomatic?.let(::prepare), data.capsLock?.let(::prepare), data.default?.let(::prepare))
        is VariationSelector -> VariationSelector(data.default?.let(::prepare), data.email?.let(::prepare),
            data.uri?.let(::prepare), data.normal?.let(::prepare), data.password?.let(::prepare))
        is LayoutDirectionSelector -> LayoutDirectionSelector(prepare(data.ltr), prepare(data.rtl))
        is CharWidthSelector -> CharWidthSelector(data.full?.let(::prepare), data.half?.let(::prepare))
        else -> data
    }

    private fun reference(data: AbstractKeyData): AbstractKeyData = when (data) {
        is CaseSelector -> ReferenceCaseSelector(reference(data.lower), reference(data.upper))
        is ShiftStateSelector -> ReferenceShiftStateSelector(data.unshifted?.let(::reference), data.shifted?.let(::reference),
            data.shiftedManual?.let(::reference), data.shiftedAutomatic?.let(::reference), data.capsLock?.let(::reference), data.default?.let(::reference))
        is VariationSelector -> ReferenceVariationSelector(data.default?.let(::reference), data.email?.let(::reference),
            data.uri?.let(::reference), data.normal?.let(::reference), data.password?.let(::reference))
        is LayoutDirectionSelector -> ReferenceLayoutDirectionSelector(reference(data.ltr), reference(data.rtl))
        is CharWidthSelector -> ReferenceCharWidthSelector(data.full?.let(::reference), data.half?.let(::reference))
        is TextKeyData -> object : AbstractKeyData {
            override fun asString(isForDisplay: Boolean) = ""
            override fun compute(evaluator: ComputingEvaluator): KeyData? = if (evaluator.isSlot(data)) {
                evaluator.slotData(data)?.let { TextKeyData(data.type, it.code, it.label, data.groupId, data.popup) }
            } else data
        }
        else -> data
    }

    private fun randomNode(random: Random, depth: Int): AbstractKeyData {
        if (depth == 0 || random.nextInt(3) == 0) return TextKeyData(code = random.nextInt(-808, 13000),
            label = "leaf${random.nextInt()}", groupId = random.nextInt(5))
        fun node() = randomNode(random, depth - 1)
        fun optional() = if (random.nextBoolean()) null else node()
        return when (random.nextInt(5)) {
            0 -> CaseSelector(node(), node())
            1 -> ShiftStateSelector(optional(), optional(), optional(), optional(), optional(), optional())
            2 -> VariationSelector(optional(), optional(), optional(), optional(), optional())
            3 -> LayoutDirectionSelector(node(), node())
            else -> CharWidthSelector(optional(), optional())
        }
    }

    // Proxy avoids constructing Android editor/context/subtype objects in JVM tests. Unexpected
    // Android access is an error, rather than a default stub value masking a new dependency.
    private fun evaluator(raw: Long): ComputingEvaluator {
        val state = KeyboardState.new(raw.toULong())
        return Proxy.newProxyInstance(ComputingEvaluator::class.java.classLoader, arrayOf(ComputingEvaluator::class.java)) { _, method, args ->
            when (method.name) {
                "getState" -> state
                "isSlot" -> ReferenceKeyRules.currency((args!![0] as KeyData).code)
                "slotData" -> {
                    val slot = ReferenceKeyRules.currencySlot((args!![0] as KeyData).code)
                    ReferenceKeyRules.currencyDefaults.getOrNull(slot)?.takeIf { raw and 1L == 0L }
                        ?.let { TextKeyData(code = it, label = "slot$slot") }
                }
                else -> error("Unexpected evaluator access: ${method.name}")
            }
        } as ComputingEvaluator
    }

    private fun signature(data: KeyData?): List<Any?>? = data?.let {
        listOf(it.type, it.code, it.label, it.groupId, it.popup, (it as? MultiTextKeyData)?.codePoints?.toList())
    }

    /** [signature] with the popup's keys for the popup, which has no `toString` of its content. */
    private fun content(data: KeyData?): List<Any?>? = data?.let {
        listOf(it.type, it.code, it.label, it.groupId, it.popup?.let { popup -> listOf(popup.main, popup.relevant) },
            (it as? MultiTextKeyData)?.codePoints?.toList())
    }
}
