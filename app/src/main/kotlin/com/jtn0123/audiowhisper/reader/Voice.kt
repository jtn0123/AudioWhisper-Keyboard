package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager

/**
 * What reads a page aloud: the phone's own text-to-speech ([Speaker]) or the built-in neural
 * voice ([NeuralVoice]). One is made per reading and shut down at its end.
 */
internal interface Voice {
    /** Whether it can speak; waits for the engine to start the first time. */
    suspend fun isReady(): Boolean

    /** Takes audio focus for a reading; false if Android refused it (a call is on). */
    fun begin(): Boolean

    /**
     * Queues [text] to be spoken after what is already queued. [onStart] runs, on a background
     * thread, when the listener starts hearing it.
     */
    fun queue(text: String, onStart: () -> Unit = {})

    /** Suspends until no more than [count] blocks are still queued or being heard. */
    suspend fun awaitBlocks(count: Int)

    /** Stops speaking, gives audio focus back and frees what the reading held. */
    fun shutdown()
}

/**
 * The audio focus a reading holds: music pauses while it speaks, and losing focus (a call, or
 * another app starting to play) calls [onLost].
 */
internal class AudioFocus(context: Context, onLost: () -> Unit) {
    private val audio = context.getSystemService(AudioManager::class.java)

    val attributes: AudioAttributes = AudioAttributes.Builder()
        .setUsage(AudioAttributes.USAGE_MEDIA)
        .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH)
        .build()

    private val request = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN_TRANSIENT)
        .setAudioAttributes(attributes)
        .setOnAudioFocusChangeListener { change ->
            if (change == AudioManager.AUDIOFOCUS_LOSS || change == AudioManager.AUDIOFOCUS_LOSS_TRANSIENT) onLost()
        }
        .build()

    fun take(): Boolean = audio.requestAudioFocus(request) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED

    fun give() {
        audio.abandonAudioFocusRequest(request)
    }
}
