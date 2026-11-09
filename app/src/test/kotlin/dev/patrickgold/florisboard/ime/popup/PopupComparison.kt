package dev.patrickgold.florisboard.ime.popup

import com.jtn0123.audiowhisper.rust.PopupGrid
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.FlorisRect
import java.lang.reflect.Proxy
import kotlin.math.nextDown
import kotlin.math.nextUp
import kotlin.random.Random

/**
 * Puts the long-press popup's rules in Rust (the extended popup's layout, the key under the
 * finger and the popup keys' order, through [extendedPopupLayout], [extendedPopupIndex],
 * [extendedPopupElementOrNull] and [MutablePopupSet]) up against the Kotlin they replaced,
 * [ReferencePopupGrid] and [ReferenceMutablePopupSet], wherever this runs: the JVM's tests and,
 * copied in for a check, the phone. It records each difference and how often each outcome was
 * seen, so a caller can tell the runs reached them all.
 */
class PopupComparison {
    val differences = mutableListOf<String>()
    val seen = mutableMapOf<String, Int>()

    private fun check(what: String, input: () -> String, kotlin: Any?, rust: Any?) {
        if (kotlin != rust && differences.size < MAX_DIFFERENCES) differences += "$what(${input()}): kotlin $kotlin, rust $rust"
    }

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    private fun bits(value: Float) = value.toBits()

    private fun rect(r: FlorisRect) = listOf(bits(r.left), bits(r.top), bits(r.right), bits(r.bottom))

    private fun FlorisRect.text() = "[$left, $top, $right, $bottom]"

    /** A float for a bound: mostly a keyboard's, with zero, negative, fractional, huge, infinite and NaN ones. */
    private fun float(random: Random, typical: Float): Float = when (random.nextInt(20)) {
        0 -> ODD_FLOATS.random(random)
        1 -> -typical * random.nextFloat()
        2 -> typical * random.nextInt(-3, 12) + random.nextFloat()
        3 -> Float.fromBits(random.nextInt())
        else -> typical * random.nextInt(0, 11)
    }

    /** A key's bounds and its preview popup's, as a keyboard of about this width lays them out. */
    private fun bounds(random: Random): Pair<FlorisRect, FlorisRect> {
        val keyWidth = if (random.nextInt(8) == 0) float(random, 100f) else KEY_WIDTHS.random(random)
        val left = if (random.nextInt(8) == 0) float(random, keyWidth) else keyWidth * random.nextInt(0, 11) + EDGES.random(random)
        val top = if (random.nextInt(10) == 0) float(random, 160f) else 160f * random.nextInt(0, 4)
        val height = if (random.nextInt(10) == 0) float(random, 160f) else 160f
        val key = FlorisRect.new(left, top, left + keyWidth, top + height)
        val baseWidth = when (random.nextInt(4)) {
            0 -> float(random, keyWidth)
            1 -> keyWidth * 1.2f
            else -> keyWidth
        }
        val baseHeight = if (random.nextInt(6) == 0) float(random, height) else height * 1.5f + random.nextInt(3)
        val baseLeft = left - (baseWidth - keyWidth) / 2f
        val base = FlorisRect.new(baseLeft, top - baseHeight, baseLeft + baseWidth, top)
        return key to base
    }

    private fun keyboardWidth(random: Random, key: FlorisRect): Float = when (random.nextInt(10)) {
        0 -> float(random, 1080f)
        1 -> key.right * random.nextFloat()
        2 -> key.width * random.nextInt(0, 4)
        else -> KEYBOARD_WIDTHS.random(random)
    }

    /** Popup keys, `null` as a key that isn't a text key has, of these sizes. */
    private fun popupKeys(random: Random): PopupKeys<*>? {
        if (random.nextInt(20) == 0) return null
        val prioritized = if (random.nextInt(10) == 0) random.nextInt(4, 7) else random.nextInt(0, 4)
        val other = when (random.nextInt(4)) {
            0 -> random.nextInt(0, 3)
            1 -> random.nextInt(0, 40)
            else -> random.nextInt(0, 9)
        }
        return PopupKeys(null, List(prioritized) { it }, List(other) { it })
    }

    /** Random extended popups, laid out by both. */
    fun layouts(random: Random, count: Int) = repeat(count) {
        val (key, base) = bounds(random)
        val width = keyboardWidth(random, key)
        val keys = popupKeys(random)
        val kotlin = ReferencePopupGrid.extend(key, base, width, keys)
        val rust = extendedPopupLayout(key, base, width, keys)
        val input = { "key ${key.text()} base ${base.text()} width $width keys ${keys?.prioritized?.size}+${keys?.other?.size}" }
        check("extend", input, kotlin == null, rust == null)
        if (kotlin == null || rust == null) {
            see("extend: none")
            return@repeat
        }
        check("extend", input, listOf(kotlin.anchorLeft, kotlin.anchorRight, kotlin.anchorOffset, kotlin.row0count, kotlin.row1count, kotlin.initUiIndex),
            listOf(rust.anchorLeft, !rust.anchorLeft, rust.anchorOffset, rust.row0Count, rust.row1Count, rust.initialIndex))
        check("extend indices", input, kotlin.popupIndices.toList(), List(rust.count) { rust.popupIndex(it) })
        check("extend rows", input, kotlin.rows.toList(), List(rust.count) { rust.row(it) })
        check("extend bounds", input, rect(kotlin.bounds), listOf(bits(rust.left), bits(rust.top), bits(rust.right), bits(rust.bottom)))
        see(if (kotlin.row1count > 0) "extend: two rows ${if (kotlin.popupIndices.size % 2 == 1) "odd" else "even"}" else "extend: one row")
        see(if (kotlin.anchorLeft) "extend: left" else "extend: right")
        ReferencePopupGrid.branches.distinct().forEach { see("extend: $it") }
    }

    /** Fingers moving over random extended popups, often just either side of an edge. */
    fun pointers(random: Random, count: Int) = repeat(count) {
        val (key, base) = bounds(random)
        val ext = ext(random, key, base)
        val bw = base.width
        val diff = (key.width - bw) / 2f
        val from = (-ext.anchorOffset.toLong() - 3).coerceIn(-40L, 40L)
        val until = (ext.row0count.toLong() + 4).coerceIn(from + 1, 41L)
        val k = random.nextLong(from, until).toFloat() + NUDGES.random(random)
        var x = key.left + when (random.nextInt(4)) {
            0 -> diff + k * bw
            1 -> key.width - diff + k * bw
            2 -> k * bw
            else -> float(random, bw)
        }
        var y = key.top + when (random.nextInt(5)) {
            0 -> -base.height
            1 -> 0.9f * base.height
            2 -> 0f
            3 -> float(random, base.height)
            else -> base.height * (random.nextFloat() * 2.2f - 1.2f)
        }
        when (random.nextInt(6)) {
            0 -> x = x.nextUp()
            1 -> x = x.nextDown()
            2 -> y = y.nextUp()
            3 -> y = y.nextDown()
        }
        val kotlin = ReferencePopupGrid.propagateMotionEvent(key, ext, x, y)
        val rust = extendedPopupIndex(key, ext, x, y).let { if (it == PopupGrid.OFF_THE_POPUP) null else it.toInt() }
        check("pointer", { "key ${key.text()} base ${base.text()} $ext at $x, $y" }, kotlin, rust)
        ReferencePopupGrid.branches.forEach { see("pointer: $it") }
    }

    /** An extended popup's render info: mostly one laid out for the key, else any grid at all. */
    private fun ext(random: Random, key: FlorisRect, base: FlorisRect): PopupUiController.ExtRenderInfo {
        val laid = if (random.nextInt(5) == 0) null else ReferencePopupGrid.extend(key, base, keyboardWidth(random, key), popupKeys(random))
        val int = { range: IntRange -> if (random.nextInt(30) == 0) ODD_INTS.random(random) else range.random(random) }
        val anchorLeft = laid?.anchorLeft ?: random.nextBoolean()
        return PopupUiController.ExtRenderInfo(
            elements = emptyList(),
            baseBounds = base,
            bounds = FlorisRect.empty(),
            anchorLeft = anchorLeft,
            anchorRight = !anchorLeft,
            anchorOffset = laid?.anchorOffset ?: int(-2..6),
            row0count = laid?.row0count ?: int(0..25),
            row1count = laid?.row1count ?: int(0..25),
        )
    }

    /** Finding an extended popup's element by its index, in one or two rows. */
    fun elements(random: Random, count: Int) = repeat(count) {
        val rows = List(random.nextInt(0, 3)) { List(random.nextInt(0, 25)) { Any() } }
        val index = random.nextInt(-3, 60)
        val kotlin = ReferencePopupGrid.getElementOrNull(rows, index)
        val rust = extendedPopupElementOrNull(rows, index)
        check("element", { "rows ${rows.map { it.size }} index $index" }, true, kotlin === rust)
        see(
            when {
                kotlin == null -> "element: none"
                rows.last().contains(kotlin) -> "element: last row"
                else -> "element: first row"
            },
        )
    }

    private class Item(val computed: KeyData?) : AbstractKeyData {
        override fun compute(evaluator: ComputingEvaluator) = computed
        override fun asString(isForDisplay: Boolean) = ""
    }

    private var key = 0

    private fun keyData(): KeyData = TextKeyData(code = ++key, label = "k$key")

    private fun item(random: Random) = Item(if (random.nextInt(5) == 0) null else keyData())

    private fun popupSet(random: Random) = PopupSet<AbstractKeyData>(
        main = if (random.nextInt(3) == 0) null else item(random),
        relevant = List(if (random.nextInt(4) == 0) 0 else random.nextInt(0, 5)) { item(random) },
    )

    /** Random runs of filling, merging into and asking two popup sets, one of each. */
    fun popupSets(random: Random, runs: Int, steps: Int) = repeat(runs) { run ->
        val kotlin = ReferenceMutablePopupSet()
        val rust = MutablePopupSet<KeyData>()
        val asked = mutableSetOf<KeyHintConfiguration>()
        repeat(steps) { step ->
            val input = { "run $run step $step" }
            when (random.nextInt(14)) {
                0 -> { kotlin.clear(); rust.clear(); asked.clear() }
                1 -> (if (random.nextInt(2) == 0) null else keyData()).let { kotlin.main = it; rust.main = it }
                2 -> (if (random.nextInt(3) == 0) null else keyData()).let { kotlin.symbolHint = it; rust.symbolHint = it }
                3 -> (if (random.nextInt(3) == 0) null else keyData()).let { kotlin.numberHint = it; rust.numberHint = it }
                4, 5 -> popupSet(random).let { set ->
                    val hadMain = kotlin.main != null
                    kotlin.merge(set, EVALUATOR); rust.merge(set, EVALUATOR)
                    seeMerge(set, hadMain)
                }
                6 -> popupSet(random).let { kotlin.mergeSymbolHint(it, EVALUATOR); rust.mergeSymbolHint(it, EVALUATOR) }
                7 -> popupSet(random).let { kotlin.mergeNumberHint(it, EVALUATOR); rust.mergeNumberHint(it, EVALUATOR) }
                else -> {
                    val config = KeyHintConfiguration(MODES.random(random), MODES.random(random), random.nextBoolean())
                    if (asked.add(config)) see("order: ${branch(kotlin, config)}")
                    val k = kotlin.getPopupKeys(config)
                    val r = rust.getPopupKeys(config)
                    check("order hint", input, true, k.hint === r.hint)
                    check("order prioritized", input, k.prioritized.map { it.label }, r.prioritized.map { it.label })
                    check("order prioritized same", input, true, same(k.prioritized, r.prioritized))
                    check("order other", input, k.other.map { it.label }, r.other.map { it.label })
                    check("order other same", input, true, same(k.other, r.other))
                    check("order other is relevant", input, k.other === kotlin.relevant, r.other === rust.relevant)
                }
            }
            check("main", input, true, kotlin.main === rust.main)
            check("relevant", input, true, same(kotlin.relevant, rust.relevant))
        }
    }

    private fun same(a: List<Any?>, b: List<Any?>) = a.size == b.size && a.indices.all { a[it] === b[it] }

    private fun seeMerge(set: PopupSet<AbstractKeyData>, hadMain: Boolean) {
        if (set.relevant.any { it.compute(EVALUATOR) == null }) see("merge: skips a key")
        if (set.main?.compute(EVALUATOR) != null) see(if (hadMain) "merge: adds main" else "merge: sets main")
    }

    /** Which way the set's order goes for [config], to tell the runs reached each. */
    private fun branch(set: ReferenceMutablePopupSet, config: KeyHintConfiguration): String {
        val symbol = set.symbolHint != null && config.symbolHintMode != KeyHintMode.DISABLED
        val number = set.numberHint != null && config.numberHintMode != KeyHintMode.DISABLED
        val first = when {
            set.main != null -> "main"
            set.relevant.isNotEmpty() -> "relevant"
            else -> "hint"
        }
        return when {
            symbol && number -> "both ${config.symbolHintMode} ${config.numberHintMode}" +
                if (config.symbolHintMode == KeyHintMode.ACCENT_PRIORITY && config.numberHintMode == KeyHintMode.ACCENT_PRIORITY) " $first" else ""
            symbol -> "symbol ${config.symbolHintMode}" + if (config.symbolHintMode == KeyHintMode.ACCENT_PRIORITY) " $first" else ""
            number -> "number ${config.numberHintMode}" + if (config.numberHintMode == KeyHintMode.ACCENT_PRIORITY) " $first" else ""
            else -> "no hints"
        }
    }

    private companion object {
        const val MAX_DIFFERENCES = 20
        val KEY_WIDTHS = listOf(108f, 98.18182f, 72f, 1f, 33.333332f)
        val KEYBOARD_WIDTHS = listOf(1080f, 720f, 1440f, 2400f, 100f, 50f)
        val EDGES = listOf(0f, 0f, 0f, 0.5f, -0.5f, 0.25f)
        val NUDGES = listOf(0f, 0f, 0.5f, -0.5f, 0.001f, -0.001f, 0.99f, 1e-7f)
        val ODD_FLOATS = listOf(
            0f, -0f, 1e-30f, -1e-30f, Float.MIN_VALUE, 1e20f, -1e20f, 3e38f, Float.MAX_VALUE, -Float.MAX_VALUE,
            Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, Float.NaN, 16_777_217f, 0.1f,
        )
        val ODD_INTS = listOf(Int.MAX_VALUE, Int.MIN_VALUE, Int.MAX_VALUE - 1, Int.MIN_VALUE + 1, -1)
        val MODES = KeyHintMode.entries + KeyHintMode.ACCENT_PRIORITY + KeyHintMode.ACCENT_PRIORITY

        /** Never asked anything: the merged items compute without it. */
        val EVALUATOR = Proxy.newProxyInstance(
            ComputingEvaluator::class.java.classLoader, arrayOf(ComputingEvaluator::class.java),
        ) { _, method, _ -> error("asked ${method.name}") } as ComputingEvaluator
    }
}
