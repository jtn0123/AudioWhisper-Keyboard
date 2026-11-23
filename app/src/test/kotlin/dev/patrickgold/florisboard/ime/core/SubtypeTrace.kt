package dev.patrickgold.florisboard.ime.core

import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlin.random.Random
import kotlin.test.assertEquals

/**
 * A seeded trace of main's subtypes (frozen in ReferenceKeyboardSubtypes.kt) and the branch's
 * (`SubtypeManager` over Rust's session) side by side, each over its own settings and dispatcher,
 * fed the same edits, language-key presses, switches by id, currency sets, cancellations and
 * process deaths. After every state both must hold the same list, active subtype, settings and
 * writes, fail alike, and serve and compute the same key graph. Failures name fields, indices and
 * hashes, never a value: the stored list is a setting.
 */
internal class SubtypeTrace(private val seed: Long) {
    private val random = Random(seed)
    private val inputs = SubtypeTraceInputs(random)
    val seen = HashMap<String, Int>()
    private val graphs = SubtypeTraceGraphs(random) { saw(it) }
    private var sets: Map<ExtensionComponentName, CurrencySet>? = inputs.currencySets()
    private var numberRow = false
    private var numberHints = false
    /** How many of the main keyboard's writes [count] has counted. */
    private var counted = 0
    private lateinit var main: TraceSide
    private lateinit var branch: TraceSide

    private fun saw(what: String) { seen[what] = (seen[what] ?: 0) + 1 }

    fun run(states: Int) {
        start(inputs.raw(List(random.nextInt(4)) { inputs.subtype(it.toLong() + 1) }), -1)
        for (state in 0 until states) {
            step()?.let { throw AssertionError("seed $seed state $state: $it") }
        }
    }

    private fun start(raw: String, id: Long) {
        main = TraceSide(raw, id, { sets }, ::MainSubtypes)
        branch = TraceSide(raw, id, { sets }, ::BranchSubtypes)
        graphs.reset()
        counted = 0
    }

    private fun step(): String? {
        event()
        drain()
        if (main.failures.isNotEmpty() || branch.failures.isNotEmpty()) return failed()
        return compare() ?: graphs.compare(mode(), numberRow, numberHints, random.nextInt(3) == 0,
            main.subtypes, branch.subtypes)
    }

    private fun event() {
        when (random.nextInt(100)) {
            in 0..24 -> edit()
            in 25..44 -> both("next") { it.subtypes.next() }
            in 45..54 -> both("previous") { it.subtypes.previous() }
            in 55..64 -> inputs.id(inputs.list(main.store.lists.value)).let { id -> both("switch by id") { it.subtypes.select(id) } }
            in 65..69 -> inputs.id(inputs.list(main.store.lists.value)).let { id -> both("active id stored") { it.store.putId(id) } }
            in 70..74 -> { sets = inputs.currencySets(); saw("currency sets loaded") }
            in 75..77 -> { numberRow = !numberRow; graphs.clearCharacters(); saw("number row") }
            78 -> { numberHints = !numberHints; saw("number hints") }
            79 -> { graphs.clear(); saw("resources changed") }
            in 80..81 -> recreate("process death")
            in 82..84 -> { edit(); edit(); saw("edits conflated") }
            85 -> { edit(); runTasks(random.nextInt(1, 4)); recreate("process death mid-flight") }
            else -> saw("nothing")
        }
    }

    /** An edit the settings screens make, written to both keyboards' settings. */
    private fun edit() {
        val (list, name) = inputs.edit(inputs.list(main.store.lists.value))
        val raw = inputs.raw(list)
        both(name) { it.store.put(raw) }
        if (random.nextInt(3) == 0) {
            val id = inputs.id(list)
            both("edit stored an active id") { it.store.putId(id) }
        }
    }

    /** Runs every task both keyboards queued, or now and then only the first few, as a busy device may. */
    private fun drain() {
        if (random.nextInt(8) == 0) {
            val tasks = random.nextInt(1, 4)
            if (runTasks(tasks) != 0) saw("compared mid-flight")
        } else {
            while (runTasks(Int.MAX_VALUE) != 0) Unit
        }
    }

    /** Runs up to [tasks] tasks on each keyboard; the same number must have run on both. */
    private fun runTasks(tasks: Int): Int {
        val ran = main.dispatcher.run(tasks)
        assertEquals(ran, branch.dispatcher.run(tasks), "seed $seed: the keyboards ran different numbers of tasks")
        return ran
    }

    private fun both(what: String, action: (TraceSide) -> Unit) {
        action(main)
        action(branch)
        saw(what)
    }

    /** The process ended and started again from the stored settings. */
    private fun recreate(why: String) {
        main.end(); branch.end()
        start(main.store.lists.value, main.store.activeId())
        saw(why)
    }

    /** Both failed alike, or how they differed; then the settings are rewritten whole and the process restarts. */
    private fun failed(): String? {
        if (main.failures != branch.failures) return "failures: main ${main.failures}, the branch ${branch.failures}"
        saw("failed alike")
        main.end(); branch.end()
        start(inputs.raw(listOf(inputs.subtype(1))).takeIf { it.startsWith("[") } ?: "[]", -1)
        return null
    }

    private fun compare(): String? {
        val a = main.store
        val b = branch.store
        if (a.lists.value != b.lists.value) return "stored lists: ${a.lists.value.hashCode()} vs ${b.lists.value.hashCode()}"
        if (a.activeId() != b.activeId()) return "the stored active ids differ"
        if (a.writes != b.writes) {
            val at = a.writes.indices.firstOrNull { a.writes[it] != b.writes.getOrNull(it) } ?: a.writes.size
            return "writes differ at $at of ${a.writes.size} and ${b.writes.size}: ${kind(a.writes, at)} vs ${kind(b.writes, at)}"
        }
        val mainList = main.subtypes.list
        val branchList = branch.subtypes.list
        if (mainList.size != branchList.size) return "list sizes ${mainList.size} vs ${branchList.size}"
        mainList.indices.firstNotNullOfOrNull { at -> differs(mainList[at], branchList[at])?.let { "subtype $at: $it" } }
            ?.let { return it }
        differs(main.subtypes.active, branch.subtypes.active)?.let { return "active subtype: $it" }
        val manager = (branch.subtypes as BranchSubtypes).manager
        if (manager.active.subtype != manager.activeSubtype) return "the branch's active entry and flow disagree"
        count(mainList)
        return currency(mainList)
    }

    /** The currency set each keyboard finds for the active subtype and one in the list: the same object. */
    private fun currency(list: List<Subtype>): String? {
        for (subtype in listOfNotNull(main.subtypes.active, list.randomOrNull(random))) {
            val a = main.subtypes.currencySet(subtype)
            val b = branch.subtypes.currencySet(subtype)
            if (a !== b) return "currency sets differ: main ${a.id.hashCode()}, the branch ${b.id.hashCode()}"
            saw(if (a === CurrencySet.Fallback) "fallback currency set" else "loaded currency set")
        }
        return null
    }

    private fun count(list: List<Subtype>) {
        saw("list of ${minOf(list.size, 5)}")
        if (list.map { it.id }.toSet().size < list.size) saw("duplicate ids")
        for (write in main.store.writes.drop(counted)) saw("stored ${write.substringBefore(' ')}")
        counted = main.store.writes.size
        val tags = list.flatMap { it.locales() }.map { it.languageTag() }
        if (tags.any { it in RTL }) saw("right-to-left subtype")
    }

    private fun mode() = if (random.nextInt(3) != 0) KeyboardMode.CHARACTERS else MODES.random(random)

    private fun kind(writes: List<String>, at: Int) = writes.getOrNull(at)?.substringBefore(' ') ?: "none"

    private companion object {
        val RTL = setOf("ar", "ar-EG", "fa-IR", "he-IL", "iw")
        val MODES = listOf(KeyboardMode.SYMBOLS, KeyboardMode.SYMBOLS2, KeyboardMode.NUMERIC,
            KeyboardMode.NUMERIC_ADVANCED, KeyboardMode.PHONE, KeyboardMode.PHONE2)
    }
}

/** The first field in which [main] and [branch] differ, by name, or null when they are equal. */
internal fun differs(main: Subtype, branch: Subtype): String? {
    val fields = listOf<Pair<String, (Subtype) -> Any?>>(
        "id" to { it.id }, "primary locale" to { it.primaryLocale }, "primary tag" to { it.primaryLocale.languageTag() },
        "secondary locales" to { it.secondaryLocales }, "secondary tags" to { s -> s.secondaryLocales.map { it.languageTag() } },
        "providers" to { it.nlpProviders }, "composer" to { it.composer }, "currency set" to { it.currencySet },
        "punctuation rule" to { it.punctuationRule }, "popup mapping" to { it.popupMapping },
        "layouts" to { it.layoutMap }, "whole" to { it }, "hash" to { it.hashCode() },
    )
    return fields.firstOrNull { (_, field) -> field(main) != field(branch) }?.first
}
