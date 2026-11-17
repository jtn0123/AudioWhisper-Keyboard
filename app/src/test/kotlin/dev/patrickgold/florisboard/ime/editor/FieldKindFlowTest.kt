package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.EditorAttributes
import com.jtn0123.audiowhisper.rust.NlpSession
import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * What each kind of field does with the keyboard's own spaces, corrections and learning, from the
 * raw EditorInfo Kotlin hands over: the key variation Rust reads from the input type (as
 * EditorInstance.handleStartInputView asks [EditorAttributes.keyboard]), the native editor's
 * spacing as EditorInstance drives it, and the NLP policy for the fields NlpEnvironment sends.
 */
class FieldKindFlowTest {
    private class Editor(inputType: Int, imeOptions: Int) : AutoCloseable {
        val ic = FakeInputConnection()
        private val driver = NativeEditorDriver(ic)
        private val auto = EditorInstance.AutoSpaceState()
        private val phantom = EditorInstance.PhantomSpaceState()
        private val variation = EditorAttributes.keyboard(inputType, imeOptions, true, 0, false)[1]

        init {
            driver.input = inputType
            driver.run("start", "0", "0", "1")
        }

        private fun edit(operation: String, vararg fields: String) {
            val rule = PunctuationRule.Fallback
            configure()
            driver.session.call("context", auto.bits.toString(), phantom.bits.toString(), variation.toString(), "1",
                "en", rule.symbolsPrecedingAutoSpace, rule.symbolsFollowingAutoSpace, rule.symbolsPrecedingPhantomSpace,
                rule.symbolsFollowingPhantomSpace, "1")
            driver.run(operation, *fields)
            val spaces = driver.session.call("spaces")
            auto.bits = spaces[0].toInt()
            phantom.reset(spaces[1].toInt())
            val update = intArrayOf(ic.start, ic.end, ic.composingStart, ic.composingEnd)
            followUpdate(driver.session.isLate(update[0], update[1], update[2], update[3]), auto, phantom)
            configure()
            driver.run("update", *update.map(Int::toString).toTypedArray(), "1")
        }

        private fun configure() {
            driver.phantom = phantom.isActive
            driver.determine = phantom.isInactive || phantom.showComposingRegion
        }

        fun type(text: String) = text.forEach { edit("typeChar", it.toString()) }

        fun swipe(word: String) = edit("gesture", word)

        fun pick(word: String) = edit("completion", word)

        override fun close() = driver.close()
    }

    /**
     * (inputType, imeOptions, what the field is), and whether the keyboard spaces words in it. An
     * address field that asks for auto-correction (Contacts' e-mail field is 0x8021) is spaced, as
     * Gboard spaces a word picked there.
     */
    private val fields = listOf(
        Triple(0x4001, 0, "text") to true,
        Triple(0x10_0001, 0x1000_0003, "a search box") to true,
        Triple(0x200b1, 0x4000_0006, "a filter") to true,
        Triple(0x11, 0x2, "a URL") to false,
        Triple(0x21, 0x6, "an e-mail address") to false,
        Triple(0x8021, 0x6, "an e-mail address asking for corrections") to true,
        Triple(0x8011, 0x2, "a URL asking for corrections") to true,
        Triple(0xd1, 0x6, "a web e-mail address") to false,
        Triple(0x81, 0x6, "a password") to false,
    )

    @Test
    fun `a word swiped after another is spaced only in ordinary text`() {
        for ((field, spaced) in fields) Editor(field.first, field.second).use {
            it.swipe("hello")
            it.swipe("world")
            assertEquals(if (spaced) "hello world" else "helloworld", it.ic.text, field.third)
        }
    }

    @Test
    fun `a letter typed after a picked word is spaced only in ordinary text`() {
        for ((field, spaced) in fields) Editor(field.first, field.second).use {
            it.type("hel")
            it.pick("hello")
            it.type("w")
            assertEquals(if (spaced) "hello w" else "hellow", it.ic.text, field.third)
        }
    }

    /** NlpEnvironment.fields for [inputType] and [imeOptions], with suggestions and learning on. */
    private fun context(inputType: Int, imeOptions: Int, incognito: Boolean = false) = arrayOf(
        inputType.toString(), "0", "org.florisboard.nlp.providers.latin", "en", "1", "1", "1",
        if (incognito) "1" else "0", "1", "1", "3", "3", "0", "3", "hel", "en-US", ":", "1", "1", "1", "0",
        imeOptions.toString(),
    )

    @Test
    fun `only prose is learned from and remembered as kept, and addresses are corrected only when they ask`() = NlpSession().use { nlp ->
        // (suggests words, corrects them, learns them, remembers one kept as typed)
        val prose = listOf(true, true, true, true)
        val notProse = listOf(true, false, false, false)
        val asks = listOf(true, true, false, false)
        val secret = listOf(false, false, false, false)
        val expected = listOf(prose, prose, notProse, notProse, notProse, asks, asks, notProse, secret)
        for ((at, field) in fields.map { it.first }.withIndex()) {
            nlp.call("context", *context(field.first, field.second))
            val policy = nlp.call("policy", *context(field.first, field.second)).map { it == "1" }
            val kept = nlp.call("keep", "zorb")[0] == "1"
            assertEquals(expected[at], listOf(policy[1], policy[3], policy[4], kept), field.third)
        }
        nlp.call("context", *context(0x4001, 0, incognito = true))
        assertEquals("0", nlp.call("keep", "zorb")[0], "incognito")
    }
}
