package dev.patrickgold.florisboard.lib.ext

import kotlin.random.Random

/** Runs the frozen regex against Android's actual ICU rules. Reports counts, never inputs. */
object ExtensionDeviceReplay {
    private fun outcome(body: () -> Any?): Any? = try { body() } catch (e: Exception) { e.javaClass.name to e.message }

    fun run(): Long {
        var checks = 0L
        fun compare(value: String) {
            val expected = outcome { ReferenceExtensionMaintainer.from(value)?.let { listOf(it.name, it.email, it.url) } }
            val actual = outcome { ExtensionMaintainer.from(value)?.let { listOf(it.name, it.email, it.url) } }
            check(expected == actual) { "Extension maintainer parse mismatch" }
            checks++
            check(outcome { ReferenceExtensionMaintainer.fromOrTakeRaw(value).toString() } ==
                outcome { ExtensionMaintainer.fromOrTakeRaw(value).toString() }) { "Extension maintainer format mismatch" }
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
        return checks
    }
}
