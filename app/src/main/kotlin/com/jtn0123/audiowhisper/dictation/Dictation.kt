package com.jtn0123.audiowhisper.dictation

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.provider.Settings
import android.util.Log
import androidx.annotation.StringRes
import androidx.core.net.toUri
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Dictation inside the keyboard. A tap on the mic key toggles it and a hold talks until
 * it is let go (see [MicKey]); while it listens,
 * each phrase is typed as the user pauses and every key keeps working, so
 * the keyboard never goes away. Hiding the keyboard stops it, and leaving
 * the field drops whatever it hasn't typed yet: a dictation only ever types
 * into the field it started in.
 *
 * Main thread only, apart from a worker thread, which owns the model: it
 * loads, runs and releases it, and keeps phrases in the order they were
 * spoken. The logic is in [DictationSession]; this is its Android side. The model takes about 750 MB, so it stays loaded for [KEEP_LOADED_MS]
 * after a dictation (the next one starts at once) and is then released; once
 * the keyboard is out of sight, within [KEEP_LOADED_HIDDEN_MS] (see [releaseAt]).
 *
 * Every dictation that ends without typing anything says why: nothing was
 * heard, the mic stopped, the model is broken, or transcription failed.
 *
 * While a phrase is being spoken its words so far are shown in [preview]. Previews share
 * the worker with the phrases, in order, so the text typed never waits for more than one.
 */
class Dictation(private val context: Context, private val editor: Editor) {
    /** Where dictation types. Called on the main thread. */
    interface Editor {
        /** The field is one dictation may type into (see [DictationFields]). */
        fun allowsDictation(): Boolean
        fun textBeforeCursor(): CharSequence?

        /** Deletes [replacing] characters before the cursor, then types [text]. */
        fun type(text: String, replacing: Int)

        /** A capitalised word is usually written that way ("John"), so it keeps its capital mid-sentence. */
        fun isName(word: String): Boolean

        /** [phrase] as the user's settings tidy it before it is typed: filler words, vocabulary. */
        fun tidy(phrase: String): String

        /** Takes back the phrase dictation typed last, if nothing has changed since. */
        fun undoLastPhrase(): Boolean
        fun problem(@StringRes message: Int)

        /** A key-press haptic, for letting go of the mic after a hold: the press had one, the release none. */
        fun tick()
    }

    enum class Status { IDLE, LISTENING, FINISHING }

    private val platform = AndroidPlatform(context)
    private val session = DictationSession(editor, platform, flows)

    init {
        LargeModels.register(LargeModels.Kind.DICTATION, session::freeIfIdle)
    }

    /** The mic key went down. */
    fun pressMic() = session.pressMic()

    /** The mic key came up, or the finger slid off it. */
    fun releaseMic() = session.releaseMic()

    /**
     * The keyboard was hidden (back, the hide key) but its field is still there: stop
     * listening and type what was said, as a tap on the mic would. No message, since the
     * keyboard is gone; a dictation already finishing carries on.
     */
    fun stopListening() = session.stopListening()

    /** The field went away. Typed phrases stay; the rest, including the one being spoken, is dropped. */
    fun abandon() = session.abandon()

    /** The keyboard came into sight or went out of it, which sets how long an idle model is kept. */
    fun keyboardVisible(visible: Boolean) = session.keyboardVisible(visible)

    fun release() {
        LargeModels.unregister(LargeModels.Kind.DICTATION)
        session.release()
    }

    private class AndroidPlatform(private val context: Context) : DictationSession.Platform {
        private val main = Handler(Looper.getMainLooper())
        private val worker: ExecutorService = Executors.newSingleThreadExecutor()

        override fun post(action: Runnable) {
            main.post(action)
        }

        override fun postAtTime(action: Runnable, uptimeMs: Long) {
            main.postAtTime(action, uptimeMs)
        }

        override fun removeCallbacks(action: Runnable) = main.removeCallbacks(action)

        override fun uptimeMs(): Long = SystemClock.uptimeMillis()

        override fun work(action: Runnable) = worker.execute(action)

        override fun shutDownWorker() = worker.shutdown()

        override fun hasMicPermission(): Boolean =
            context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED

        override fun openAppSettings() = context.startActivity(
            Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS)
                .setData(Uri.fromParts("package", context.packageName, null))
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
        )

        override fun modelState(): ModelState = ModelFiles.quickState(context)

        override fun openModelSettings() = context.startActivity(
            // BROWSABLE is what FlorisAppActivity takes as a deep link; without it, it would
            // treat the link as an extension file to import.
            Intent(Intent.ACTION_VIEW, DICTATION_SETTINGS.toUri())
                .addCategory(Intent.CATEGORY_BROWSABLE)
                .setPackage(context.packageName)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
        )

        override fun listen(callbacks: DictationSession.Callbacks): DictationSession.Listening =
            PhraseListener(context.assets, callbacks.onLevel, callbacks.onMicError, callbacks.onPhrase, callbacks.onPreview)
                .apply { start(::openAudio) }

        /**
         * The mic, or in a debug build a recording copied in for testing (see [MicAudio.Replay]).
         * The session asked for the permission already; checked again here, where the mic opens,
         * in case it was revoked since, which the session then explains like any mic that won't open.
         */
        private fun openAudio(): MicAudio {
            val replay = MicAudio.Replay.file(context)
            if (replay == null) {
                // Checked inline, where lint can see it guards the mic.
                val granted = context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
                check(granted) { "No microphone permission" }
                return MicAudio.Mic.open()
            }
            Log.i(TAG, "Listening to ${replay.name} instead of the mic")
            return try {
                MicAudio.Replay.open(replay)
            } catch (e: IllegalArgumentException) {
                throw IllegalStateException("Could not read ${replay.name}", e)
            }
        }

        override fun loadModel(): DictationSession.SpeechModel {
            // A cut-short or wrong file makes the native loader fail vaguely, or worse, so the
            // files are checked first (hashed once each; see ModelCheck).
            val state = ModelFiles.fullState(context)
            check(state == ModelState.Ready) { "Model files not as expected: $state in ${ModelFiles.dir(context)}" }
            LargeModels.loading(LargeModels.Kind.DICTATION)
            return Transcriber(ModelFiles.dir(context))
        }

        override fun log(level: DictationSession.LogLevel, message: String, error: Throwable?) {
            when (level) {
                DictationSession.LogLevel.DEBUG -> Log.d(TAG, message, error)
                DictationSession.LogLevel.INFO -> Log.i(TAG, message, error)
                DictationSession.LogLevel.WARN -> Log.w(TAG, message, error)
                DictationSession.LogLevel.ERROR -> Log.e(TAG, message, error)
            }
        }
    }

    companion object {
        private const val TAG = "AWDictation"
        private const val DICTATION_SETTINGS = "ui://florisboard/settings/dictation"
        internal const val KEEP_LOADED_MS = 5 * 60 * 1000L

        /** Long enough to switch to another app and dictate there without loading it again. */
        internal const val KEEP_LOADED_HIDDEN_MS = 60 * 1000L

        // One keyboard per process, so the mic button can watch these
        // without holding on to the service.
        private val flows = DictationFlows()

        /** What the mic button shows. */
        val status: StateFlow<Status> = flows.status.asStateFlow()

        /** Loudness while listening, 0..1, for the mic's glow. */
        val level: StateFlow<Float> = flows.level.asStateFlow()

        /** The words of the phrase being spoken, as heard so far; empty when there is none. Never logged. */
        val preview: StateFlow<String> = flows.preview.asStateFlow()

        /** The model is loading for this dictation (about 2 s on a phone), so its first phrase comes late. */
        val warming: StateFlow<Boolean> = flows.warming.asStateFlow()
    }
}
