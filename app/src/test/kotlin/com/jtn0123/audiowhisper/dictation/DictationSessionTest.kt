package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.dictation.Dictation.Status
import dev.patrickgold.florisboard.R
import java.util.concurrent.RejectedExecutionException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Dictation's session logic, with the main thread, the worker, the mic and the model played by
 * [FakePlatform]: nothing runs until the test says so, which is how a phrase is made to arrive
 * late, after the field changed or the keyboard went away.
 */
class DictationSessionTest {
    private val platform = FakePlatform()
    private val editor = FakeEditor()
    private val flows = DictationFlows()
    private val dictation = DictationSession(editor, platform, flows)

    private val status get() = flows.status.value

    /** A tap of the mic: down and up within a hold. */
    private fun tap() {
        dictation.pressMic()
        platform.now += 50
        dictation.releaseMic()
    }

    /** The user says [text] and pauses, so the listener hands the phrase over. */
    private fun say(text: String) = platform.listening.last().phrase(text)

    @Test
    fun `a tap starts, a phrase is typed, a second tap stops`() {
        tap()
        assertEquals(Status.LISTENING, status)
        say("hello there")
        platform.runAll()
        assertEquals("hello there", editor.text.toString())
        tap()
        assertEquals(Status.FINISHING, status)
        platform.runAll()
        assertEquals(Status.IDLE, status)
        assertEquals(emptyList(), editor.problems)
    }

    @Test
    fun `stopping hands over the words since the last pause`() {
        tap()
        platform.listening.last().unfinished = "and one more"
        tap()
        platform.runAll()
        assertEquals("and one more", editor.text.toString())
    }

    @Test
    fun `a dictation that hears nothing says so`() {
        tap()
        tap()
        platform.runAll()
        assertEquals(listOf(R.string.dictation__not_caught), editor.problems)
    }

    @Test
    fun `a failed transcription says so`() {
        tap()
        say(FakeModel.FAIL)
        tap()
        platform.runAll()
        assertEquals(listOf(R.string.dictation__failed), editor.problems)
        assertEquals("", editor.text.toString())
    }

    @Test
    fun `hiding the keyboard types what was said, without a message`() {
        tap()
        platform.listening.last().unfinished = "see you soon"
        dictation.stopListening()
        platform.runAll()
        assertEquals("see you soon", editor.text.toString())
        assertEquals(Status.IDLE, status)

        tap()
        dictation.stopListening()
        platform.runAll()
        assertEquals(emptyList(), editor.problems, "nothing heard, but the keyboard is gone")
    }

    @Test
    fun `hiding the keyboard after a stop lets the stop finish`() {
        tap()
        say("first")
        tap()
        dictation.stopListening()
        platform.runAll()
        assertEquals("first", editor.text.toString())
        assertEquals(Status.IDLE, status)
    }

    @Test
    fun `a field change drops the phrase being transcribed`() {
        tap()
        say("meant for the old field")
        dictation.abandon()
        assertEquals(Status.IDLE, status)
        platform.runAll()
        assertEquals("", editor.text.toString())
        assertTrue(platform.listening.last().stopped)
        assertFalse(platform.listening.last().kept)
    }

    @Test
    fun `a phrase handed over after the field changed is dropped`() {
        tap()
        val old = platform.listening.last()
        dictation.abandon()
        old.phrase("too late")
        platform.runAll()
        assertEquals("", editor.text.toString())
    }

    @Test
    fun `an old dictation's late phrase doesn't reach the next one`() {
        tap()
        val old = platform.listening.last()
        dictation.abandon()
        tap()
        old.phrase("too late")
        say("on time")
        platform.runAll()
        assertEquals("on time", editor.text.toString())
    }

    @Test
    fun `a model that won't load stops the dictation and says so`() {
        platform.model.loads = false
        tap()
        platform.runAll()
        assertEquals(Status.IDLE, status)
        assertEquals(listOf(R.string.dictation__model_failed), editor.problems)
        assertTrue(platform.listening.last().stopped)
    }

    @Test
    fun `the load is tried again on the next dictation`() {
        platform.model.loads = false
        tap()
        platform.runAll()
        platform.model.loads = true
        tap()
        say("works now")
        platform.runAll()
        assertEquals("works now", editor.text.toString())
    }

    @Test
    fun `a missing model opens its settings instead of listening`() {
        platform.modelState = ModelState.Missing(listOf(ModelFiles.ENCODER))
        tap()
        assertEquals(Status.IDLE, status)
        assertEquals(listOf(R.string.dictation__not_installed), editor.problems)
        assertEquals(1, platform.modelSettingsOpened)
        assertEquals(emptyList(), platform.listening)
    }

    @Test
    fun `without the mic permission it opens the app's settings`() {
        platform.micAllowed = false
        tap()
        assertEquals(Status.IDLE, status)
        assertEquals(listOf(R.string.dictation__allow_mic), editor.problems)
        assertEquals(1, platform.appSettingsOpened)
    }

    @Test
    fun `a field that doesn't allow dictation says so`() {
        editor.allows = false
        tap()
        assertEquals(Status.IDLE, status)
        assertEquals(listOf(R.string.dictation__field_off), editor.problems)
    }

    @Test
    fun `a mic that won't open says so`() {
        platform.micOpens = false
        tap()
        assertEquals(Status.IDLE, status)
        assertEquals(listOf(R.string.dictation__mic_busy), editor.problems)
    }

    @Test
    fun `the model is released once idle long enough, and kept while it is used`() {
        tap()
        say("hello")
        tap()
        platform.runAll()
        assertTrue(platform.model.loaded)

        platform.now += Dictation.KEEP_LOADED_MS - 1
        platform.runAll()
        assertTrue(platform.model.loaded)

        platform.now += 1
        platform.runAll()
        assertFalse(platform.model.loaded)
    }

    @Test
    fun `a hidden keyboard releases the model sooner`() {
        tap()
        tap()
        platform.runAll()
        dictation.keyboardVisible(false)
        platform.now += Dictation.KEEP_LOADED_HIDDEN_MS
        platform.runAll()
        assertFalse(platform.model.loaded)
    }

    @Test
    fun `the voice loading frees the model only when no dictation uses it`() {
        tap()
        platform.runAll()
        dictation.freeIfIdle()
        platform.runAll()
        assertTrue(platform.model.loaded, "kept while listening")
        tap()
        platform.runAll()
        dictation.freeIfIdle()
        platform.runAll()
        assertFalse(platform.model.loaded)
    }

    @Test
    fun `an idle release queued before shutdown does not submit work afterwards`() {
        dictation.freeIfIdle()
        dictation.release()
        platform.runAll()
        dictation.freeIfIdle()
        dictation.release()
        platform.runAll()
        assertFalse(platform.model.loaded)
    }

    @Test
    fun `queued model and phrase results do nothing after release`() {
        tap()
        say("too late")
        dictation.release()
        platform.runAll()
        dictation.pressMic()
        dictation.releaseMic()
        assertEquals(Status.IDLE, status)
        assertEquals("", editor.text.toString())
        assertEquals(emptyList(), editor.problems)
        assertFalse(flows.warming.value)
        assertFalse(platform.model.loaded)
    }

    @Test fun nativeVadFailureIsClassifiedAndLateErrorsAreDropped() {
        tap()
        val old = platform.listening.single()
        old.modelError(); platform.runAll()
        assertEquals(Status.IDLE, status)
        assertEquals(listOf(R.string.dictation__model_failed), editor.problems)
        tap()
        old.modelError(); platform.runAll()
        assertEquals(Status.LISTENING, status)
        assertEquals(1, editor.problems.size)
    }

    @Test
    fun `scratch that takes back the last phrase`() {
        tap()
        say("hello there")
        say("scratch that")
        platform.runAll()
        assertEquals(1, editor.undone)
    }
}
