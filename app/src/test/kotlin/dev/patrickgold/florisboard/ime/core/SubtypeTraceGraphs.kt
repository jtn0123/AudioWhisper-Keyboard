package dev.patrickgold.florisboard.ime.core

import com.jtn0123.audiowhisper.render.KeyGraphFactsInput
import com.jtn0123.audiowhisper.render.RustKeyboard
import com.jtn0123.audiowhisper.render.TRACE_PLATFORM
import com.jtn0123.audiowhisper.render.referenceLabelResource
import com.jtn0123.audiowhisper.render.traceAnswer
import com.jtn0123.audiowhisper.rust.InputProtocol
import com.jtn0123.audiowhisper.rust.InputRules
import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.KeyGraphOp
import com.jtn0123.audiowhisper.rust.LayoutCache
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.keyboard.KEY_LABEL_RESOURCES
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.KeyboardState
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import kotlin.random.Random

/**
 * Each keyboard's key graphs, kept in a `LayoutCache` under its subtype's cache key as
 * `RustKeyGraphs` keeps them (main's under `hashCode()`, the branch's under Rust's key), and
 * computed for facts that take the subtype count, language and currency set from that keyboard.
 */
internal class SubtypeTraceGraphs(private val random: Random, private val saw: (String) -> Unit) {
    private class Served(val row: Array<String>, val keyboard: RustKeyboard)

    private var main = LayoutCache<Served>()
    private var branch = LayoutCache<Served>()

    /** Resources changed: every keyboard is dropped, on both. */
    fun clear() { main.clear(); branch.clear() }

    /** The number row changed: the characters keyboards are dropped. */
    fun clearCharacters() { main.clearMode(KeyboardMode.CHARACTERS.value); branch.clearMode(KeyboardMode.CHARACTERS.value) }

    /** The process ended: the caches went with it. */
    fun reset() { main = LayoutCache(); branch = LayoutCache() }

    /**
     * Serves the active subtype's keyboard for [mode] on both and, when [compute], computes both;
     * null when they match: both found or both built it, from the same row, computing the same keys.
     */
    fun compare(mode: KeyboardMode, numberRow: Boolean, numberHints: Boolean, compute: Boolean,
                mainSide: TracedSubtypes, branchSide: TracedSubtypes): String? {
        val (mainHit, mainServed) = serve(main, mode, numberRow, numberHints, mainSide)
        val (branchHit, branchServed) = serve(branch, mode, numberRow, numberHints, branchSide)
        if (mainHit != branchHit) return "main ${hit(mainHit)} its keyboard, the branch ${hit(branchHit)}"
        saw(if (mainHit) "keyboard from the cache" else "keyboard built")
        if (!mainServed.row.contentEquals(branchServed.row)) {
            val field = mainServed.row.indices.firstOrNull { mainServed.row[it] != branchServed.row.getOrNull(it) }
            return "the build rows differ at field $field"
        }
        if (!compute) return null
        val base = facts(mode)
        val mainKeys = keys(mainServed.keyboard, base, mainSide)
        val branchKeys = keys(branchServed.keyboard, base, branchSide)
        saw("keyboard computed")
        return when {
            mainKeys.isFailure || branchKeys.isFailure -> exceptions(mainKeys, branchKeys)
            !mainKeys.getOrThrow().contentEquals(branchKeys.getOrThrow()) -> "the computed keys differ"
            else -> null
        }
    }

    private fun serve(cache: LayoutCache<Served>, mode: KeyboardMode, numberRow: Boolean, numberHints: Boolean,
                      side: TracedSubtypes): Pair<Boolean, Served> {
        var built = false
        val served = side.cached(cache, mode) {
            built = true
            val row = side.row(mode, numberRow, numberHints)
            Served(row, RustKeyboard(KeyGraph.build(*row), side.active))
        }
        return !built to served
    }

    private fun keys(keyboard: RustKeyboard, base: KeyGraphFactsInput, side: TracedSubtypes) = runCatching {
        val active = side.active
        val facts = base.copy(subtypes = side.list.size, language = active.primaryLocale.displayName(),
            slots = side.currencySet(active).slots)
        keyboard.compute(facts.row(), traceAnswer(active.primaryLocale.base))
        keyboard.graph.call(KeyGraphOp.KEYS)
    }

    private fun exceptions(main: Result<Array<String>>, branch: Result<Array<String>>): String? {
        val mainError = main.exceptionOrNull()?.javaClass
        val branchError = branch.exceptionOrNull()?.javaClass
        return if (mainError == branchError) null.also { saw("computing crashed alike") }
            else "computing: main ${mainError ?: "computed"}, the branch ${branchError ?: "computed"}"
    }

    /** The facts no subtype decides, for [mode]. */
    private fun facts(mode: KeyboardMode): KeyGraphFactsInput {
        val state = KeyboardState.new().also {
            it.keyboardMode = mode
            it.inputShiftState = InputShiftState.entries.random(random)
        }
        val evaluated = InputRules.control(longArrayOf(InputProtocol.CONTROL_EVALUATOR.toLong(), state.rawValue.toLong()))
        return KeyGraphFactsInput(
            evaluated[0].toULong(), 1, random.nextInt(8), random.nextBoolean(), random.nextInt(5), random.nextBoolean(),
            random.nextBoolean(), false, random.nextBoolean(), 0, false, 0, random.nextBoolean(), "", "English",
            KeyHintConfiguration(KeyHintMode.entries.random(random), KeyHintMode.entries.random(random), random.nextBoolean()),
            emptyList(), KEY_LABEL_RESOURCES.map { TRACE_PLATFORM.string(referenceLabelResource(it)) },
        )
    }

    private fun hit(hit: Boolean) = if (hit) "found" else "built"
}
