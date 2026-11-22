package com.jtn0123.audiowhisper.settingsui

import dev.patrickgold.florisboard.ime.input.InputKeyEventReceiver
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlin.test.*

/**
 * The key recorder's decisions on the receiver slot InputEventDispatcher sends through: one key-up is taken, a key
 * down, repeat or cancel is not, and the keyboard's receiver comes back, unless another was put in since. A real
 * InputEventDispatcher cannot be made on the JVM (its companion asks android.view.ViewConfiguration), so [Dispatcher]
 * is its `keyEventReceiver` and the order its sendDownUp calls the receiver in.
 */
class SettingsKeyRecorderTest {
    private class Dispatcher {
        var keyEventReceiver: InputKeyEventReceiver? = null
        fun sendDownUp(data: KeyData) {
            keyEventReceiver?.onInputKeyDown(data)
            keyEventReceiver?.onInputKeyUp(data)
        }
    }
    /** KeyboardManager, the keyboard's own receiver: what reached it. */
    private class Keyboard : InputKeyEventReceiver {
        val seen = mutableListOf<String>()
        override fun onInputKeyDown(data: KeyData) { seen += "down ${data.code}" }
        override fun onInputKeyUp(data: KeyData) { seen += "up ${data.code}" }
        override fun onInputKeyRepeat(data: KeyData) { seen += "repeat ${data.code}" }
        override fun onInputKeyCancel(data: KeyData) { seen += "cancel ${data.code}" }
    }
    private val a = checkNotNull(TextKeyData.getCodeInfoAsTextKeyData(97))
    private val keyboard = Keyboard()
    private val dispatcher = Dispatcher().also { it.keyEventReceiver = keyboard }
    private val recorded = mutableListOf<Int>()
    private val recording = KeyRecording(dispatcher::keyEventReceiver) { recorded += it }

    @Test fun oneKeyUpIsTakenAndTheKeyboardGetsItsReceiverBack() {
        recording.start()
        assertSame(recording, dispatcher.keyEventReceiver)
        assertTrue(recording.listening)
        dispatcher.sendDownUp(TextKeyData.DELETE)
        assertEquals(listOf(KeyCode.DELETE), recorded)
        assertSame(keyboard, dispatcher.keyEventReceiver)
        assertFalse(recording.listening)
        // The next key is the keyboard's, and nothing reached it while recording.
        dispatcher.sendDownUp(a)
        assertEquals(listOf(KeyCode.DELETE), recorded)
        assertEquals(listOf("down 97", "up 97"), keyboard.seen)
    }

    @Test fun aKeyDownRepeatOrCancelIsNotTaken() {
        recording.start()
        val receiver = checkNotNull(dispatcher.keyEventReceiver)
        receiver.onInputKeyDown(a)
        receiver.onInputKeyRepeat(a)
        receiver.onInputKeyCancel(a)
        assertEquals(emptyList(), recorded)
        assertTrue(recording.listening, "still waiting for a key-up")
        assertEquals(emptyList(), keyboard.seen)
        receiver.onInputKeyUp(a)
        assertEquals(listOf(97), recorded)
    }

    @Test fun stoppingGivesTheReceiverBackOnlyWhileItIsHeld() {
        assertFalse(recording.stop(), "not listening")
        recording.start()
        // Starting again keeps the keyboard's receiver to give back, not this one.
        recording.start()
        assertTrue(recording.stop())
        assertSame(keyboard, dispatcher.keyEventReceiver)
        assertFalse(recording.stop())
        // A receiver put in after this one is left alone, and a key-up then reaches it, not the recording.
        recording.start()
        val other = Keyboard()
        dispatcher.keyEventReceiver = other
        assertFalse(recording.listening)
        assertFalse(recording.stop())
        assertSame(other, dispatcher.keyEventReceiver)
        dispatcher.sendDownUp(a)
        assertEquals(emptyList(), recorded)
        assertEquals(listOf("down 97", "up 97"), other.seen)
    }

    @Test fun withNoReceiverTheSlotIsEmptyAgainAfterTheKey() {
        dispatcher.keyEventReceiver = null
        recording.start()
        dispatcher.sendDownUp(a)
        assertEquals(listOf(97), recorded)
        assertNull(dispatcher.keyEventReceiver)
    }
}
