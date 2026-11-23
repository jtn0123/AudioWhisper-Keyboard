package dev.patrickgold.florisboard.lib.ext

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random

/**
 * Runs ExtensionMaintainer under Android's actual ICU rules, recording its answers on the golden
 * tape [TAPE], read from the test APK's assets on a device and written there by the frozen regex it
 * replaced (docs/phase6-kotlin-references.md). Reports counts, never inputs.
 */
object ExtensionDeviceReplay {
    const val TAPE = "extension-maintainer-device"

    private fun outcome(body: () -> Any?): Any? = try { body() } catch (e: Exception) { e.javaClass.name to e.message }

    /**
     * Records [value], a parse's outcome, on [tape] as [what] and its parts: a maintainer's three
     * fields, each null or its text, or the exception's class and message.
     */
    fun record(tape: GoldenTape, what: String, value: Any?) = when (value) {
        is List<*> -> tape.record(listOf(what, "parts") + value.map { if (it == null) "null" else "=$it" })
        is Pair<*, *> -> tape.record(listOf(what, "threw", value.first.toString(), value.second?.let { "=$it" } ?: "null"))
        else -> tape.answer(what, value)
    }

    fun run(goldens: GoldenTape.Store): Long {
        val tape = GoldenTape(TAPE, goldens)
        var checks = 0L
        fun compare(value: String) {
            record(tape, "from", outcome { ExtensionMaintainer.from(value)?.let { listOf(it.name, it.email, it.url) } })
            checks++
            record(tape, "fromOrTakeRaw", outcome { ExtensionMaintainer.fromOrTakeRaw(value).toString() })
            checks++
        }
        for (unit in Char.MIN_VALUE..Char.MAX_VALUE) {
            compare(unit.toString())
            compare("A${unit} B <x> (y)")
        }
        val random = Random(20261113)
        val pieces = listOf("Jane Doe", "é", "𐐀", "١", "\uD800", " ", "\t", "\n", "\u00a0", "<", ">", "(", ")", ".", "_", "-", "<a@b>", "(a.b)", "< >", "( )")
        repeat(20_000) { compare(List(random.nextInt(10)) { pieces.random(random) }.joinToString("")) }
        for (value in listOf("X (a>b)", "X (a<b)", "X <a(b>", "A\u0085")) compare(value)
        tape.finish()
        return checks
    }
}
