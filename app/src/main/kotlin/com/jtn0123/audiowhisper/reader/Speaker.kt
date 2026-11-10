package com.jtn0123.audiowhisper.reader

import android.content.Context
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
 * Holds audio focus while it speaks ([AudioFocus]). Text is never logged.
 */
internal class Speaker(context: Context, onFocusLost: () -> Unit) : Voice {
    private val focus = AudioFocus(context, onFocusLost)
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

    init {
        tts.setAudioAttributes(focus.attributes)
        tts.setOnUtteranceProgressListener(Progress())
    }

    override suspend fun isReady(): Boolean = ready.await()

    override fun begin(): Boolean = focus.take()

    /** [onStart] runs on the speech engine's thread, when the block's first sentence starts. */
    override fun queue(text: String, onStart: () -> Unit) {
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

    override suspend fun awaitBlocks(count: Int) {
        while (live.size > count) finished.receive()
    }

    override fun shutdown() {
        live.clear()
        starts.clear()
        tts.stop()
        finished.trySend(Unit)
        focus.give()
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
