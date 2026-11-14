package com.jtn0123.audiowhisper.dictation

import java.util.concurrent.RejectedExecutionException

internal class FakeEditor : Dictation.Editor {
    val text = StringBuilder()
    val problems = mutableListOf<Int>()
    var allows = true
    var undone = 0
    var maxLength = Int.MAX_VALUE

    override fun allowsDictation() = allows
    override fun textBeforeCursor(): CharSequence = text
    override fun type(text: String, replacing: Int) {
        this.text.setLength(this.text.length - replacing)
        this.text.append(text.take((maxLength - this.text.length).coerceAtLeast(0)))
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
    var ticks = 0
    override fun tick() { ticks++ }
}

/** Transcribes a phrase back to the text [FakeListening.phrase] encoded in it. */
internal class FakeModel {
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

internal class FakeListening(private val callbacks: DictationSession.Callbacks) : DictationSession.Listening {
    var stopped = false
    var kept = false

    /** Said since the last pause; handed over by a [stop] that keeps it. */
    var unfinished: String? = null

    fun preview(text: String) = callbacks.onPreview(FloatArray(text.length) { text[it].code.toFloat() })
    fun error() = callbacks.onMicError(-1)
    fun speech(value: Boolean) = callbacks.onSpeech(value)
    fun modelError() = callbacks.onModelError(IllegalStateException("Native VAD fixture failure"))
    fun level(value: Float) = callbacks.onLevel(value)
    fun phrase(text: String) = callbacks.onPhrase(FloatArray(text.length) { text[it].code.toFloat() })

    override fun stop(keepLastPhrase: Boolean) {
        stopped = true
        kept = keepLastPhrase
        if (keepLastPhrase) unfinished?.let(::phrase)
    }
}

/** Queues the main thread's and the worker's work until [runAll]; time moves only when a test moves [now]. */
internal class FakePlatform : DictationSession.Platform {
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

    fun runOne(mainFirst: Boolean) {
        timed.filter { it.first <= now }.also { due -> timed.removeAll(due) }.forEach { main.addLast(it.second) }
        (if (mainFirst) main.removeFirstOrNull() ?: worker.removeFirstOrNull() else worker.removeFirstOrNull() ?: main.removeFirstOrNull())?.run()
    }
    fun queued() = listOf(main.size, worker.size)
    fun schedule() = timed.map { it.first }
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
