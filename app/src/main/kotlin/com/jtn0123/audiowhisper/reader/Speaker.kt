package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.speech.tts.TextToSpeech
import android.speech.tts.UtteranceProgressListener
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.channels.Channel
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger

/**
 * The phone's own text-to-speech (whichever engine the user picked in Android's settings), fed one
 * block of text at a time. A block is queued as its sentences, and [awaitBlocks] lets the reader
 * keep a couple queued, so there is no pause between them, without running ahead of the screen.
 *
 * Holds audio focus while it speaks (music pauses, and a call stops it: [onFocusLost]). Text is
 * never logged.
 */
internal class Speaker(context: Context, private val onFocusLost: () -> Unit) {
    private val audio = context.getSystemService(AudioManager::class.java)
    private val ready = CompletableDeferred<Boolean>()
    private val tts = TextToSpeech(context.applicationContext) { status ->
        ready.complete(status == TextToSpeech.SUCCESS)
    }
    private val nextId = AtomicInteger()

    /** Blocks queued and not yet finished. A callback for any other (one [stop] dropped) is ignored. */
    private val live: MutableSet<Int> = ConcurrentHashMap.newKeySet()

    /** What to do when a block starts being spoken, by block; taken on its first sentence. */
    private val starts = ConcurrentHashMap<Int, () -> Unit>()

    /** One signal per block finished (or dropped by [stop]). */
    private val finished = Channel<Unit>(Channel.UNLIMITED)

    private val attributes = AudioAttributes.Builder()
        .setUsage(AudioAttributes.USAGE_MEDIA)
        .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH)
        .build()
    private val focus = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN_TRANSIENT)
        .setAudioAttributes(attributes)
        .setOnAudioFocusChangeListener { change ->
            if (change == AudioManager.AUDIOFOCUS_LOSS || change == AudioManager.AUDIOFOCUS_LOSS_TRANSIENT) onFocusLost()
        }
        .build()

    init {
        tts.setAudioAttributes(attributes)
        tts.setOnUtteranceProgressListener(Progress())
    }

    /** Whether an engine is there and started; waits for it the first time. */
    suspend fun isReady(): Boolean = ready.await()

    /** Takes audio focus for a reading; false if Android refused it (a call is on). */
    fun begin(): Boolean = audio.requestAudioFocus(focus) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED

    /**
     * Queues [text] to be spoken after what is already queued. [onStart] runs, on the speech
     * engine's thread, when its first sentence starts.
     */
    fun queue(text: String, onStart: () -> Unit = {}) {
        val pieces = PageText.utterances(text)
        if (pieces.isEmpty()) return
        val block = nextId.incrementAndGet()
        live += block
        starts[block] = onStart
        pieces.forEachIndexed { index, piece ->
            val id = if (index == pieces.lastIndex) "$block$SEPARATOR$LAST" else "$block$SEPARATOR$index"
            tts.speak(piece, TextToSpeech.QUEUE_ADD, null, id)
        }
    }

    /** Suspends until no more than [count] blocks are still queued or being spoken. */
    suspend fun awaitBlocks(count: Int) {
        while (live.size > count) finished.receive()
    }

    /** Stops speaking, drops the queue and gives audio focus back. */
    fun stop() {
        live.clear()
        starts.clear()
        tts.stop()
        finished.trySend(Unit)
        audio.abandonAudioFocusRequest(focus)
    }

    fun shutdown() {
        stop()
        tts.shutdown()
    }

    private fun blockEnded(utteranceId: String) {
        // A block's middle sentences ending means nothing; its last one does.
        if (utteranceId.substringAfter(SEPARATOR) != LAST) return
        val block = blockOf(utteranceId) ?: return
        starts.remove(block)
        if (live.remove(block)) finished.trySend(Unit)
    }

    /** The engine's word on each sentence; only a block's first start and last end matter. */
    private inner class Progress : UtteranceProgressListener() {
        override fun onStart(utteranceId: String) {
            blockOf(utteranceId)?.let(starts::remove)?.invoke()
        }

        override fun onDone(utteranceId: String) = blockEnded(utteranceId)

        @Deprecated("Deprecated in Java", ReplaceWith("onError(utteranceId, errorCode)"))
        override fun onError(utteranceId: String) = blockEnded(utteranceId)

        override fun onError(utteranceId: String, errorCode: Int) = blockEnded(utteranceId)

        override fun onStop(utteranceId: String, interrupted: Boolean) = blockEnded(utteranceId)
    }

    private fun blockOf(utteranceId: String): Int? = utteranceId.substringBefore(SEPARATOR).toIntOrNull()

    private companion object {
        const val SEPARATOR = ':'
        const val LAST = "last"
    }
}
