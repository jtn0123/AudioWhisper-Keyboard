package dev.patrickgold.florisboard.lib.ext

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class ExtensionMaintainerMatchTest {
    private fun outcome(body: () -> Any?): Any? = try { body() } catch (e: Exception) { e.javaClass.name to e.message }
    private fun check(value: String) {
        assertEquals(outcome { ReferenceExtensionMaintainer.from(value)?.let { listOf(it.name, it.email, it.url) } },
            outcome { ExtensionMaintainer.from(value)?.let { listOf(it.name, it.email, it.url) } })
        assertEquals(outcome { ReferenceExtensionMaintainer.fromOrTakeRaw(value).toString() },
            outcome { ExtensionMaintainer.fromOrTakeRaw(value).toString() })
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
            assertEquals(ReferenceExtensionMaintainer(name, email, url).toString(), ExtensionMaintainer(name, email, url).toString())
        }
    }
}
