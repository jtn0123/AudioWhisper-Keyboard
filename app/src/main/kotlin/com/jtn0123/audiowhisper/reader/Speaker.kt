package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.os.Bundle
import android.speech.tts.TextToSpeech
import android.speech.tts.UtteranceProgressListener
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.channels.Channel
import java.util.concurrent.ConcurrentHashMap
import com.jtn0123.audiowhisper.rust.PlaybackQueue
import dev.patrickgold.florisboard.R
import java.util.Locale
import java.util.concurrent.atomic.AtomicInteger

/**
 * The phone's own text-to-speech (whichever engine the user picked in Android's settings), fed one
 * block of text at a time. A block is queued as its sentences, and [awaitBlocks] lets the reader
 * keep a couple queued, so there is no pause between them, without running ahead of the screen.
 *
 * It speaks only with a voice that works offline ([OfflineVoice]), and asks the engine not to
 * synthesise on a server as well: with no such voice it reads nothing.
 *
 * Holds audio focus while it speaks ([AudioFocus]). Text is never logged.
 */
internal class Speaker(context: Context, onFocusLost: () -> Unit) : Voice {
    private val focus = AudioFocus(context, onFocusLost)
    private val started = CompletableDeferred<Boolean>()
    private val tts = TextToSpeech(context.applicationContext) { status ->
        started.complete(status == TextToSpeech.SUCCESS)
    }
    private val nextId = AtomicInteger()

    /** Set once an offline voice is the engine's: nothing is spoken before. */
    @Volatile private var offline = false

    private val params = Bundle().apply { putString(OfflineVoice.NETWORK_SYNTHESIS, false.toString()) }

    /** Blocks queued and not yet finished. A callback for any other (one [stop] dropped) is ignored. */
    private val queue = PlaybackQueue()

    /** What to do when a block starts being spoken, by block; taken on its first sentence. */
    private val starts = ConcurrentHashMap<Int, () -> Unit>()

    /** One signal per block finished (or dropped by [stop]). */
    private val finished = Channel<Unit>(Channel.UNLIMITED)

    init {
        tts.setAudioAttributes(focus.attributes)
        tts.setOnUtteranceProgressListener(Progress())
    }

    override suspend fun unavailable(): Int? = when {
        !started.await() -> R.string.read_aloud__no_engine
        !useOfflineVoice() -> R.string.read_aloud__no_offline_voice
        else -> null
    }

    /** Gives the engine the voice [OfflineVoice] picks; false if there is none or it refused it. */
    private fun useOfflineVoice(): Boolean {
        val voice = OfflineVoice.choose(tts.voices.orEmpty(), tts.defaultVoice, Locale.getDefault()) ?: return false
        offline = tts.setVoice(voice) == TextToSpeech.SUCCESS
        return offline
    }

    override fun begin(): Boolean = focus.take()

    /** [onStart] runs on the speech engine's thread, when the block's first sentence starts. */
    override fun queue(text: String, onStart: () -> Unit) {
        val pieces = PageText.utterances(text)
        if (pieces.isEmpty() || !offline) return
        val block = nextId.incrementAndGet()
        queue.system(0, block)
        starts[block] = onStart
        pieces.forEachIndexed { index, piece ->
            val id = if (index == pieces.lastIndex) "$block$SEPARATOR$LAST" else "$block$SEPARATOR$index"
            tts.speak(piece, TextToSpeech.QUEUE_ADD, params, id)
        }
    }

    override suspend fun awaitBlocks(count: Int) {
        while (queue.system(4)[2] > count) finished.receive()
    }

    override fun shutdown() {
        queue.system(3)
        starts.clear()
        tts.stop()
        finished.trySend(Unit)
        focus.give()
        tts.shutdown()
    }

    private fun blockEnded(utteranceId: String) {
        val ended = queue.system(2, text = utteranceId)
        if (ended[1] != 0L) { starts.remove(ended[0].toInt()); finished.trySend(Unit) }
    }

    /** The engine's word on each sentence; only a block's first start and last end matter. */
    private inner class Progress : UtteranceProgressListener() {
        override fun onStart(utteranceId: String) {
            val started = queue.system(1, text = utteranceId)
            if (started[1] != 0L) starts.remove(started[0].toInt())?.invoke()
        }

        override fun onDone(utteranceId: String) = blockEnded(utteranceId)

        @Deprecated("Deprecated in Java", ReplaceWith("onError(utteranceId, errorCode)"))
        override fun onError(utteranceId: String) = blockEnded(utteranceId)

        override fun onError(utteranceId: String, errorCode: Int) = blockEnded(utteranceId)

        override fun onStop(utteranceId: String, interrupted: Boolean) = blockEnded(utteranceId)
    }

    private companion object {
        const val SEPARATOR = ':'
        const val LAST = "last"
    }
}
