package com.jtn0123.audiowhisper.render

import com.ibm.icu.lang.UCharacter
import com.ibm.icu.lang.UCharacterCategory
import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.KeyGraphLocale
import com.jtn0123.audiowhisper.rust.KeyGraphOp
import com.jtn0123.audiowhisper.rust.KeyGraphQuestion
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.KeyboardState
import dev.patrickgold.florisboard.ime.keyboard.LayoutArrangement
import dev.patrickgold.florisboard.ime.keyboard.ReferenceLayoutAssembly
import dev.patrickgold.florisboard.ime.popup.PopupMapping
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import dev.patrickgold.florisboard.ime.text.keyboard.MultiTextKeyData
import dev.patrickgold.florisboard.lib.io.loadJsonAsset
import java.util.IdentityHashMap
import java.util.Locale
import kotlin.random.Random

/** ICU's casing and mark predicate (icu4j standing in for Android's), and resource strings by id. */
internal val TRACE_PLATFORM = ReferencePlatform(
    { UCharacter.toString(it) },
    { UCharacter.getType(it).toByte() == UCharacterCategory.NON_SPACING_MARK },
    { "string $it" },
)

/** `Subtype.DEFAULT`'s locale, which the smartbar and the symbol hints case in. */
internal val DEFAULT_LOCALE: Locale = Subtype.DEFAULT.primaryLocale.base

/** Answers the Rust graph's questions as Android does, through the production adapter. */
internal fun traceAnswer(subtype: Locale) = { ask: Int, locale: Int, value: String ->
    keyGraphAnswer(ask, value, if (locale == KeyGraphLocale.SUBTYPE) subtype else DEFAULT_LOCALE,
        TRACE_PLATFORM.codeText, TRACE_PLATFORM.mark)
}

/** Main's `mergeLayouts`, reading the same files: each parsed once, as main's layout cache kept them. */
internal class ReferenceBuilder {
    private val arrangements = HashMap<String, LayoutArrangement?>()
    private val mappings = HashMap<String, PopupMapping?>()

    fun keyboard(build: TraceBuild): ReferenceKeyboard {
        val files = build.files()
        val rows = rows(build.mode, build.numberRow, files)
        val symbols = if (build.mode == KeyboardMode.CHARACTERS && rows.isNotEmpty()) {
            rows(KeyboardMode.SYMBOLS, build.numberRow, files)
        } else {
            null
        }
        val default = ReferenceEvaluator(
            KeyboardMode.CHARACTERS, DEFAULT_FACTS, DEFAULT_LOCALE, "", TRACE_PLATFORM, slots = false,
        )
        return ReferenceKeyboard.build(build.mode, rows, symbols, build.numberHints, default,
            files.mapping?.let(::mapping), files.fallback?.let(::mapping))
    }

    private fun rows(mode: KeyboardMode, numberRow: Boolean, files: KeyGraphFiles): LayoutArrangement {
        val (main, modifier, extension) = ReferenceLayoutAssembly.plan(mode, numberRow, files.layouts)
        val mainLayout = main?.let { arrangement(files, it) }
        val named = main?.takeIf { mainLayout != null }?.let(files.namedModifier)
        val modifierName =
            if (main != null && named != null) ReferenceLayoutAssembly.modifier(main.first) to named else modifier
        return ReferenceLayoutAssembly.merge(mainLayout, modifierName?.let { arrangement(files, it) },
            extension?.let { arrangement(files, it) })
    }

    private fun arrangement(files: KeyGraphFiles, name: LayoutName): LayoutArrangement? =
        files.layout(name)?.let { text ->
            arrangements.getOrPut(text) { loadJsonAsset<LayoutArrangement>(text).getOrNull() }
        }

    private fun mapping(text: String) = mappings.getOrPut(text) { loadJsonAsset<PopupMapping>(text).getOrNull() }

    companion object {
        /** `DefaultComputingEvaluator`'s facts: a zero state, and nothing else it reads. */
        val DEFAULT_FACTS = KeyGraphFactsInput(
            0u, 0, 0, false, 0, false, false, false, false, 0, false, 0, false, "", "",
            KeyHintConfiguration(KeyHintMode.DISABLED, KeyHintMode.DISABLED, false), emptyList(), emptyList())
    }
}

/** Object identities that must pair one to one: main's objects on one side, the branch's on the other. */
internal class Identities {
    private val forward = IdentityHashMap<Any, Any>()
    private val backward = IdentityHashMap<Any, Any>()

    fun pair(main: Any?, branch: Any?): Boolean {
        if (main == null || branch == null) return main == null && branch == null
        val f = forward.putIfAbsent(main, branch)
        val b = backward.putIfAbsent(branch, main)
        return (f == null || f === branch) && (b == null || b === main)
    }
}

/** One build's keyboards: main's and the Rust graph, kept and recomputed as the keyboard cache kept them. */
internal class TracePair(val build: TraceBuild, val main: ReferenceKeyboard, val branch: RustKeyboard) {
    val popups = Identities()
    val multis = Identities()
}

/**
 * Compares a computed main keyboard with the Rust graph's, key by key and field by field: what
 * each key shows and sends, its popups for two hint settings, and the layout. Returns where they
 * first differ, or null.
 */
internal class TraceComparison(private val random: Random, private val seen: MutableMap<String, Int>) {
    private fun saw(what: String) { seen[what] = (seen[what] ?: 0) + 1 }

    fun compare(pair: TracePair, facts: KeyGraphFactsInput, second: KeyHintConfiguration): String? {
        val main = pair.main.listed
        val branch = pair.branch.keys
        if (main.size != branch.size) return "listed ${main.size} vs ${branch.size}"
        if (pair.main.mode != pair.branch.mode) return "mode ${pair.main.mode} vs ${pair.branch.mode}"
        for (i in main.indices) key(pair, main[i], branch[i])?.let { return "key $i: $it" }
        for (config in listOf(facts.hints, second)) popups(pair, config)?.let { return it }
        return layout(pair)
    }

    private fun key(pair: TracePair, main: ReferenceKey, branch: RustKey): String? {
        if (main.isVisible != branch.isVisible) return "visible ${main.isVisible} vs ${branch.isVisible}"
        if (main.isEnabled != branch.isEnabled) return "enabled ${main.isEnabled} vs ${branch.isEnabled}"
        data(pair, main.computedData, branch.computedData)?.let { return "data: $it" }
        if (main.label != branch.label) return "label"
        if (main.hintedLabel != branch.hintedLabel) return "hinted label"
        if (main.icon != branch.icon) return "icon ${main.icon} vs ${branch.icon}"
        val base = (main.data as? KeyData)?.code ?: KeyCode.UNSPECIFIED
        if (base != (branch.data as? KeyData)?.code) return "base code $base vs ${branch.data}"
        count(main)
        return null
    }

    private fun count(key: ReferenceKey) {
        if (!key.isVisible) saw("hidden key") else if (!key.isEnabled) saw("disabled key")
        if (key.label != null) saw("label")
        if (key.hintedLabel != null) saw("hinted label")
        if (key.icon != 0) saw("icon ${key.icon}")
        if (key.computedData is MultiTextKeyData) saw("multi-text key")
        if (key.computedData.popup != null) saw("own popup")
    }

    /** Where [main] and [branch] differ: class, type, code, label, group, code points, or popup set identity. */
    private fun data(pair: TracePair, main: KeyData, branch: KeyData): String? {
        if (main.javaClass != branch.javaClass) {
            return "class ${main.javaClass.simpleName} vs ${branch.javaClass.simpleName}"
        }
        if (main.type != branch.type || main.code != branch.code) {
            return "type/code ${main.type} ${main.code} vs ${branch.type} ${branch.code}"
        }
        if (main.label != branch.label) return "label"
        if (main.groupId != branch.groupId) return "group ${main.groupId} vs ${branch.groupId}"
        if (main is MultiTextKeyData) {
            if (!main.codePoints.contentEquals((branch as MultiTextKeyData).codePoints)) return "code points"
            if (!pair.multis.pair(main, branch)) return "multi-text identity"
        }
        if ((main.popup == null) != (branch.popup == null)) {
            return "popup set ${main.popup != null} vs ${branch.popup != null}"
        }
        if (!pair.popups.pair(main.popup, branch.popup)) return "popup set identity"
        return null
    }

    private fun popups(pair: TracePair, config: KeyHintConfiguration): String? {
        val flags = pair.branch.popupFlags(config)
        for ((i, key) in pair.main.listed.withIndex()) {
            val main = key.popups.getPopupKeys(config)
            if (flags[i] != if (main.isNotEmpty()) 1 else 0) return "key $i popup flag for $config"
            if (main.isEmpty() && random.nextInt(8) != 0) continue
            val branch = pair.branch.popupKeys(i, config)
            if (main.prioritized.size != branch.prioritized.size || main.other.size != branch.other.size) {
                return "key $i popup sizes ${main.prioritized.size}+${main.other.size} vs " +
                    "${branch.prioritized.size}+${branch.other.size} for $config"
            }
            for ((j, pairOf) in main.toList().zip(branch.toList()).withIndex()) {
                data(pair, pairOf.first, pairOf.second)?.let { return "key $i popup $j for $config: $it" }
            }
            if (main.isNotEmpty()) saw("popup keys") else saw("no popup keys")
            if (main.hint != null) saw("popup hint")
        }
        return null
    }

    private fun layout(pair: TracePair): String? {
        val width = if (random.nextInt(40) == 0) listOf(0f, -1f).random(random) else 240f + random.nextInt(1400)
        val height = 150f + random.nextInt(900)
        val row = 30f + random.nextInt(150)
        val marginH = random.nextInt(12).toFloat()
        val marginV = random.nextInt(12).toFloat()
        val main = pair.main.layout(width, height, row, marginH, marginV)
        val branch = pair.branch.layout(width, height, row, marginH, marginV)
        if (!main.contentEquals(branch)) {
            val at = main.indices.firstOrNull { main.getOrNull(it)?.toRawBits() != branch.getOrNull(it)?.toRawBits() }
            return "layout at $at of ${main.size} vs ${branch.size}"
        }
        saw(if (width > 0f) "layout" else "layout of nothing")
        return null
    }
}

/**
 * A seeded run of key graph states: each picks a build (often one already computed, as the
 * keyboard cache reuses them) and facts, computes main's keyboard and the Rust graph, and compares
 * them. Some states first start a computation and abandon it with its questions open (a newer
 * state replaced it), and some drop every keyboard, as a recreated service does.
 */
internal class KeyGraphTrace(private val seed: Long) {
    private val random = Random(seed)
    private val inputs = TraceInputs(random)
    val seen = HashMap<String, Int>()
    private val comparison = TraceComparison(random, seen)
    private var builder = ReferenceBuilder()
    private val pairs = LinkedHashMap<TraceBuild, TracePair>()

    private fun saw(what: String) { seen[what] = (seen[what] ?: 0) + 1 }

    fun run(states: Int) {
        for (state in 0 until states) {
            step()?.let { throw AssertionError("seed $seed state $state: $it") }
        }
    }

    private fun step(): String? {
        if (random.nextInt(2_000) == 0) recreate("recreated")
        val build = if (pairs.isNotEmpty() && random.nextInt(5) != 0) pairs.keys.random(random) else inputs.build()
        val facts = inputs.facts(build)
        val pair = pairs[build] ?: pair(build, facts) ?: return null.also { count(build, facts) }
        while (pairs.size > 40) pairs.remove(pairs.keys.first())
        repeat(if (random.nextInt(20) == 0) 2 else 1) {
            val crashed = compute(pair, facts)
            if (crashed != null) {
                return crashed.takeIf { it.isNotEmpty() }?.let { "$it, in ${describe(build)}" }
                    .also { count(build, facts) }
            }
        }
        count(build, facts)
        return comparison.compare(pair, facts, inputs.hints())?.let { "$it, in ${describe(build)}" }
    }

    /** A failing build, its synthetic files named rather than printed. */
    private fun describe(build: TraceBuild) = "${build.copy(synthetic = null)} with synthetic layouts " +
        "${build.synthetic?.layouts?.keys}, modifiers ${build.synthetic?.modifiers} " +
        "and mapping ${build.synthetic?.mapping != null}"

    /** A new build's pair, or null if main crashed building it, as the branch then must computing it. */
    private fun pair(build: TraceBuild, facts: KeyGraphFactsInput): TracePair? {
        val fields = keyGraphBuild(build.mode, build.numberRow, build.numberHints, build.files())
        val branch = RustKeyboard(KeyGraph.build(*fields))
        val main = runCatching { builder.keyboard(build) }.getOrElse { error ->
            val failure = runCatching { branch.compute(facts.row(), traceAnswer(build.locale)) }.exceptionOrNull()
                ?: throw AssertionError("seed $seed: main crashed building $build ($error), the branch didn't", error)
            if (failure.javaClass != error.javaClass) {
                throw AssertionError("seed $seed: main crashed with $error, the branch with $failure")
            }
            recreate("crashed alike building")
            return null
        }
        saw("built ${build.mode}")
        return TracePair(build, main, branch).also { fresh ->
            pairs[build] = fresh
            if (random.nextInt(3) == 0) runCatching { abandon(fresh) }
        }
    }

    /**
     * Computes both keyboards for [facts]: null if both did, "" if both crashed alike (and the
     * service, recreated, drops every keyboard), or how they differed.
     */
    private fun compute(pair: TracePair, facts: KeyGraphFactsInput): String? {
        var asked = ""
        val answer = traceAnswer(pair.build.locale)
        val branch = runCatching {
            pair.branch.compute(facts.row()) { ask, locale, value ->
                asked = "ask $ask locale $locale of $value"
                answer(ask, locale, value)
            }
        }.exceptionOrNull()
        val main = runCatching {
            val build = pair.build
            val evaluator = ReferenceEvaluator(build.mode, facts, build.locale, facts.language, TRACE_PLATFORM)
            pair.main.compute(evaluator, facts.hints)
        }.exceptionOrNull()
        if (main == null && branch == null) return null
        if (main?.javaClass != branch?.javaClass) {
            return "main ${main ?: "computed"}, the branch ${branch ?: "computed"} (last $asked)"
        }
        recreate("crashed alike computing")
        return ""
    }

    private fun recreate(why: String) {
        pairs.clear()
        builder = ReferenceBuilder()
        saw(why)
    }

    /** Starts a computation for other facts and leaves it, its questions open or answered only once. */
    private fun abandon(pair: TracePair) {
        val facts = inputs.facts(pair.build).row()
        val questions = pair.branch.graph.call(KeyGraphOp.COMPUTE, *facts)
        if (questions.isEmpty()) return
        saw("abandoned with questions")
        if (random.nextBoolean()) {
            val answer = traceAnswer(pair.build.locale)
            pair.branch.graph.call(KeyGraphOp.ANSWER, *Array(questions.size / KeyGraphQuestion.SIZE) {
                val at = it * KeyGraphQuestion.SIZE
                answer(questions[at + KeyGraphQuestion.ASK].toInt(), questions[at + KeyGraphQuestion.LOCALE].toInt(),
                    questions[at + KeyGraphQuestion.VALUE])
            })
            pair.branch.graph.call(KeyGraphOp.COMPUTE, *facts)
            saw("abandoned after an answer")
        }
    }

    private fun count(build: TraceBuild, facts: KeyGraphFactsInput) {
        saw("mode ${build.mode}")
        if (build.locale.language in RTL) saw("right-to-left locale")
        if (build.broken != null) saw("broken layout")
        if (build.misnamed != null) saw("misnamed modifier")
        if (build.synthetic != null) saw("synthetic layouts")
        if (build.synthetic?.mapping != null) saw("synthetic popup mapping")
        if (build.synthetic?.modifiers?.isNotEmpty() == true) saw("synthetic modifier")
        if (facts.slots.size < 6) saw("missing currency slots")
        val state = KeyboardState.new().also { it.rawValue = facts.state }
        saw("shift ${state.inputShiftState}")
        saw("variation ${state.keyVariation}")
        saw("ime action ${facts.options and 0xFF}")
        saw("input class ${facts.input and 0xF}")
    }

    private companion object {
        val RTL = setOf("ar", "he", "fa", "ur")
    }
}
