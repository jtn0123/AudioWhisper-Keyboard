package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.media.AudioFormat
import android.media.AudioTrack
import android.os.SystemClock
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicInteger

/**
 * Reads with the built-in neural voice ([VoiceEngine]). Each block is generated a sentence at a
 * time on a thread of its own and streamed to the speaker as it is made, a few seconds ahead of
 * what is heard; [PlayQueue] follows the output so blocks start and end when they are heard.
 *
 * One reading's worth: [shutdown] stops it for good. Text is never logged, only timings.
 */
internal class NeuralVoice(context: Context, private val engine: VoiceEngine, onFocusLost: () -> Unit) : Voice {
    private val focus = AudioFocus(context, onFocusLost)
    private val track = AudioTrack.Builder()
        .setAudioAttributes(focus.attributes)
        .setAudioFormat(
            AudioFormat.Builder()
                .setEncoding(AudioFormat.ENCODING_PCM_FLOAT)
                .setSampleRate(engine.sampleRate)
                .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                .build(),
        )
        .setTransferMode(AudioTrack.MODE_STREAM)
        .setBufferSizeInBytes(engine.sampleRate * Float.SIZE_BYTES * BUFFER_SECONDS)
        .build()
        .apply { setStartThresholdInFrames(engine.sampleRate / START_FRACTION) }

    private val worker = Executors.newSingleThreadExecutor { Thread(it, "neural-voice") }
    private val scope = CoroutineScope(Dispatchers.Default)
    private val plays = PlayQueue()
    private val finished = Channel<Unit>(Channel.UNLIMITED)
    private val nextId = AtomicInteger()

    @Volatile
    private var stopped = false

    /** Frames written to [track]; only the worker thread changes it. */
    @Volatile
    private var written = 0L

    /** Time the worker has spent waiting for room in [track]'s buffer, in ms. */
    private var waited = 0L

    init {
        scope.launch { follow() }
    }

    override suspend fun isReady(): Boolean = true

    override fun begin(): Boolean = focus.take()

    override fun queue(text: String, onStart: () -> Unit) {
        val pieces = PageText.utterances(text)
        if (pieces.isEmpty() || stopped) return
        val block = nextId.incrementAndGet()
        plays.add(block, onStart)
        worker.execute { speak(block, pieces) }
    }

    override suspend fun awaitBlocks(count: Int) {
        while (plays.size > count) finished.receive()
    }

    override fun shutdown() {
        stopped = true
        scope.cancel()
        repeat(plays.clear()) { finished.trySend(Unit) }
        // Pausing makes a write in progress return, so the worker gets to the release.
        track.pause()
        track.flush()
        worker.execute { track.release() }
        worker.shutdown()
        focus.give()
    }

    private fun speak(block: Int, pieces: List<String>) {
        if (stopped) return
        plays.begins(block, written)
        val began = SystemClock.elapsedRealtime()
        val from = written
        val waitedBefore = waited
        for (piece in pieces) {
            if (stopped) return
            runCatching { engine.speak(piece, ::write) }
                .onFailure { Log.w(TAG, "A sentence failed: ${it.javaClass.simpleName}") }
        }
        plays.ends(block, written)
        // Generation only: writing waits while the buffer is full, which says nothing about speed.
        val generating = SystemClock.elapsedRealtime() - began - (waited - waitedBefore)
        val seconds = (written - from).toDouble() / engine.sampleRate
        Log.d(TAG, "Generated %.1f s of speech in %.1f s".format(seconds, generating / 1000.0))
    }

    /** Sends [samples] to the speaker, waiting while its buffer is full; false once stopped. */
    private fun write(samples: FloatArray): Boolean {
        if (stopped) return false
        if (track.playState != AudioTrack.PLAYSTATE_PLAYING) track.play()
        var offset = 0
        val started = SystemClock.elapsedRealtime()
        try {
            while (offset < samples.size) {
                val count = track.write(samples, offset, samples.size - offset, AudioTrack.WRITE_BLOCKING)
                if (count <= 0 || stopped) return false
                offset += count
                written += count
            }
            return true
        } finally {
            waited += SystemClock.elapsedRealtime() - started
        }
    }

    /** Follows the output, telling [PlayQueue] how far it has played. */
    private suspend fun follow() {
        while (scope.isActive) {
            // The head is an unsigned 32-bit count of frames played since the track started.
            val heard = plays.heard(track.playbackHeadPosition.toLong() and 0xFFFF_FFFFL)
            heard.started.forEach { it() }
            repeat(heard.finished) { finished.trySend(Unit) }
            delay(FOLLOW_MS)
        }
    }

    private companion object {
        const val TAG = "NeuralVoice"
        const val BUFFER_SECONDS = 4
        const val START_FRACTION = 5
        const val FOLLOW_MS = 40L
    }
}
