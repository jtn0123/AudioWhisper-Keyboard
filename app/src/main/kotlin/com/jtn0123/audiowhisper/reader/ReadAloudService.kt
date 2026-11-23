package com.jtn0123.audiowhisper.reader

import android.accessibilityservice.AccessibilityService
import android.content.Intent
import android.graphics.Rect
import android.os.SystemClock
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityWindowInfo
import android.widget.Toast
import com.jtn0123.audiowhisper.rust.ReadingRules
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Reads the app in front aloud, from its accessibility tree, with the built-in neural voice when
 * it is installed ([VoiceEngine]) and the phone's own text-to-speech otherwise.
 *
 * It costs nothing while idle: it asks for no events and keeps no cache, so apps aren't asked to
 * report anything to it, and it looks at the screen only when a reading starts. While one is on
 * it keeps Android's node cache (a long page is walked in a few hundred calls, not thousands),
 * shows a stop button over every app, and scrolls lists that only hold what's on screen.
 *
 * What it reads is never logged or kept, only counts and timings.
 */
class ReadAloudService : AccessibilityService() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default.limitedParallelism(1))
    private val stopButton by lazy { StopButton(this, ::stopReading) }
    private val voices by lazy { LoadedVoice(NeuralVoiceSource(this), scope) }

    @Volatile
    private var reading: Job? = null

    /** The voice reading now; only touched on the main thread. */
    private var speaking: Voice? = null

    /** The neural voice the reading took, if any. */
    private var engine: VoiceEngine? = null

    override fun onServiceConnected() {
        setCacheEnabled(false)
        ReadAloud.attach(this)
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) = Unit

    override fun onInterrupt() = stopReading()

    /**
     * Turned off, perhaps mid-sentence. The reading's voice stops first, so its worker makes no
     * new sentence; then the neural voice is freed off the main thread, after the sentence in
     * progress (which stops at its next piece of audio), so the main thread never waits on it.
     */
    override fun onUnbind(intent: Intent?): Boolean {
        ReadAloud.detach(this)
        stopSpeaking()
        scope.cancel()
        CoroutineScope(Dispatchers.IO).launch { voices.close() }
        return super.onUnbind(intent)
    }

    /** Starts a reading, ending any that is on first. */
    fun read(from: ReadAloud.From) {
        val previous = reading
        reading = scope.launch {
            previous?.cancelAndJoin()
            Reading(host).run(from)
        }
    }

    fun stopReading() {
        reading?.cancel()
    }

    /** Main thread only: ends the reading's voice for good, once. */
    private fun stopSpeaking() {
        speaking?.shutdown()
        speaking = null
    }

    /** What a reading does here; its steps are [Reading]'s. */
    private val host = object : ReadingHost {
        override fun status(reading: Boolean) = ReadAloud.reading(reading)

        override fun cache(on: Boolean) {
            setCacheEnabled(on)
        }

        override suspend fun <T> onMain(block: suspend () -> T): T = withContext(Dispatchers.Main) { block() }

        override fun showButton() = stopButton.show()

        override fun hideButton() = stopButton.hide()

        override suspend fun takeVoice(): Boolean = voices.take().also { engine = it } != null

        override suspend fun voiceFailedJustNow() = voices.failedJustNow()

        override fun makeVoice(neural: Boolean): Voice {
            val service = this@ReadAloudService
            val voice = if (neural) {
                NeuralVoice(service, checkNotNull(engine), onFocusLost = ::stopReading)
            } else {
                Speaker(service, onFocusLost = ::stopReading)
            }
            return voice.also { speaking = it }
        }

        override fun stopSpeaking() = this@ReadAloudService.stopSpeaking()

        override suspend fun idleVoice() = voices.idle()

        override suspend fun tell(message: Int, long: Boolean) = withContext(Dispatchers.Main) {
            Toast.makeText(this@ReadAloudService, message, if (long) Toast.LENGTH_LONG else Toast.LENGTH_SHORT).show()
        }

        override fun walk(skipOwn: Boolean) = this@ReadAloudService.walk(skipOwn)

        override suspend fun wait(ms: Long) = delay(ms)
    }

    private fun walk(skipOwn: Boolean): Screen? {
        val started = SystemClock.elapsedRealtime()
        val screen = appWindow(skipOwn)?.let { (root, visible) -> ScreenWalk(visible).walk(root) } ?: return null
        val took = SystemClock.elapsedRealtime() - started
        val inList = screen.blocks.count(TextBlock::scrolls)
        Log.d(TAG, "Walked ${screen.nodes} nodes into ${screen.blocks.size} blocks ($inList in the list) in $took ms")
        return screen
    }

    /** The app the user is in and the part of the screen it shows above the keyboard: [ReadingRules]' choice. */
    private fun appWindow(skipOwn: Boolean): Pair<AccessibilityNodeInfo, Rect>? {
        val all = windows
        val (app, keyboard) = ReadingRules.window(
            BooleanArray(all.size) { all[it].type == AccessibilityWindowInfo.TYPE_APPLICATION },
            BooleanArray(all.size) { all[it].type == AccessibilityWindowInfo.TYPE_INPUT_METHOD },
            BooleanArray(all.size) { all[it].isActive }, BooleanArray(all.size) { all[it].isFocused },
            IntArray(all.size) { all[it].layer })
        val window = all.getOrNull(app)
        val root = window?.root ?: return null
        if (skipOwn && root.packageName == packageName) return null
        val visible = Rect().also(window::getBoundsInScreen)
        all.getOrNull(keyboard)?.let { ime ->
            visible.bottom = ReadingRules.visibleBottom(visible.top, visible.bottom, Rect().also(ime::getBoundsInScreen).top)
        }
        return root to visible
    }

    private companion object {
        const val TAG = "ReadAloud"
    }
}
