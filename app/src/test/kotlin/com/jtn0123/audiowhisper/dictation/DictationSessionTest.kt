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

    @Test
    fun `scratch that takes back the last phrase`() {
        tap()
        say("hello there")
        say("scratch that")
        platform.runAll()
        assertEquals(1, editor.undone)
    }
}

private class FakeEditor : Dictation.Editor {
    val text = StringBuilder()
    val problems = mutableListOf<Int>()
    var allows = true
    var undone = 0

    override fun allowsDictation() = allows
    override fun textBeforeCursor(): CharSequence = text
    override fun type(text: String, replacing: Int) {
        this.text.setLength(this.text.length - replacing)
        this.text.append(text)
    }
    override fun isName(word: String) = false
    override fun tidy(phrase: String) = phrase
    override fun undoLastPhrase(): Boolean {
        undone++
        return true
    }
    override fun problem(message: Int) {
        problems += message
    }
    override fun tick() = Unit
}

/** Transcribes a phrase back to the text [FakeListening.phrase] encoded in it. */
private class FakeModel {
    var loads = true
    var loaded = false

    fun load(): DictationSession.SpeechModel {
        check(loads) { "won't load" }
        loaded = true
        return object : DictationSession.SpeechModel {
            override fun transcribe(samples: FloatArray): String {
                val text = samples.map { it.toInt().toChar() }.joinToString("")
                check(text != FAIL) { "transcription failed" }
                return text
            }

            override fun close() {
                loaded = false
            }
        }
    }

    companion object {
        const val FAIL = "<fail>"
    }
}

private class FakeListening(private val callbacks: DictationSession.Callbacks) : DictationSession.Listening {
    var stopped = false
    var kept = false

    /** Said since the last pause; handed over by a [stop] that keeps it. */
    var unfinished: String? = null

    fun phrase(text: String) = callbacks.onPhrase(FloatArray(text.length) { text[it].code.toFloat() })

    override fun stop(keepLastPhrase: Boolean) {
        stopped = true
        kept = keepLastPhrase
        if (keepLastPhrase) unfinished?.let(::phrase)
    }
}

/** Queues the main thread's and the worker's work until [runAll]; time moves only when a test moves [now]. */
private class FakePlatform : DictationSession.Platform {
    var now = 1_000L
    var micAllowed = true
    var micOpens = true
    var modelState: ModelState = ModelState.Ready
    var appSettingsOpened = 0
    var modelSettingsOpened = 0
    val model = FakeModel()
    val listening = mutableListOf<FakeListening>()

    private val main = ArrayDeque<Runnable>()
    private val timed = mutableListOf<Pair<Long, Runnable>>()
    private val worker = ArrayDeque<Runnable>()
    private var shutDown = false

    /** Runs everything due by [now], main and worker, until both are empty. */
    fun runAll() {
        while (true) {
            timed.filter { it.first <= now }.also { due -> timed.removeAll(due) }.forEach { main.addLast(it.second) }
            val next = main.removeFirstOrNull() ?: worker.removeFirstOrNull() ?: return
            next.run()
        }
    }

    override fun post(action: Runnable) = main.addLast(action)
    override fun postAtTime(action: Runnable, uptimeMs: Long) {
        timed += uptimeMs to action
    }
    override fun removeCallbacks(action: Runnable) {
        main.remove(action)
        timed.removeAll { it.second === action }
    }
    override fun uptimeMs() = now
    override fun work(action: Runnable) {
        if (shutDown) throw RejectedExecutionException("worker shut down")
        worker.addLast(action)
    }
    override fun shutDownWorker() {
        shutDown = true
    }
    override fun hasMicPermission() = micAllowed
    override fun openAppSettings() {
        appSettingsOpened++
    }
    override fun modelState() = modelState
    override fun openModelSettings() {
        modelSettingsOpened++
    }
    override fun listen(callbacks: DictationSession.Callbacks): DictationSession.Listening {
        check(micOpens) { "mic busy" }
        return FakeListening(callbacks).also { listening += it }
    }
    override fun loadModel() = model.load()
    override fun log(level: DictationSession.LogLevel, message: String, error: Throwable?) = Unit
}
