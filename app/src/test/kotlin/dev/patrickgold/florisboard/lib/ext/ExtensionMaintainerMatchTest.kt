package dev.patrickgold.florisboard.lib.ext

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/** The reference's answers are recorded on the golden tape extension-maintainer (docs/phase6-kotlin-references.md). */
class ExtensionMaintainerMatchTest {
    private val tape = GoldenTape.ofReference("extension-maintainer")

    private fun outcome(body: () -> Any?): Any? = try { body() } catch (e: Exception) { e.javaClass.name to e.message }
    private fun check(value: String) {
        val parsed = outcome { ReferenceExtensionMaintainer.from(value)?.let { listOf(it.name, it.email, it.url) } }
        assertEquals(parsed, outcome { ExtensionMaintainer.from(value)?.let { listOf(it.name, it.email, it.url) } })
        ExtensionDeviceReplay.record(tape, "from", parsed)
        val raw = outcome { ReferenceExtensionMaintainer.fromOrTakeRaw(value).toString() }
        assertEquals(raw, outcome { ExtensionMaintainer.fromOrTakeRaw(value).toString() })
        ExtensionDeviceReplay.record(tape, "fromOrTakeRaw", raw)
    }

    @Test fun unicodeAndSeededMaintainersMatchTheFrozenRegexAndSubstringLogic() {
        for (unit in Char.MIN_VALUE..Char.MAX_VALUE) {
            check(unit.toString())
            check("A${unit} B <x> (y)")
        }
        val random = Random(20261113)
        val pieces = listOf("Jane Doe", "jane123", "__jane__", "jane.doe", "é", "𐐀", "𰀀", "١", "\uD800", " ", "\t", "\n", "\u00a0", "<", ">", "(", ")", ".", "_", "-", "<a@b>", "(a.b)", "< >", "( )")
        repeat(20_000) { check(List(random.nextInt(10)) { pieces.random(random) }.joinToString("")) }
        for (value in listOf("Jane Doe <jane@example.test> (example.test)", "X (a>b)", "X (a<b)", "X <a(b>", "X <a)b>", "A\r\n", "A\u0085")) check(value)
        repeat(10_000) {
            val name = pieces.random(random)
            val email = if (random.nextBoolean()) null else pieces.random(random)
            val url = if (random.nextBoolean()) null else pieces.random(random)
            val formatted = ReferenceExtensionMaintainer(name, email, url).toString()
            assertEquals(formatted, ExtensionMaintainer(name, email, url).toString())
            tape.answer("toString", formatted)
        }
        tape.finish()
    }
}
