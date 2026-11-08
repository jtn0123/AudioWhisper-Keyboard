package com.jtn0123.audiowhisper.reader

import android.accessibilityservice.AccessibilityService
import android.content.Intent
import android.graphics.Rect
import android.os.SystemClock
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityNodeInfo.AccessibilityAction
import android.view.accessibility.AccessibilityWindowInfo
import android.widget.Toast
import androidx.annotation.StringRes
import dev.patrickgold.florisboard.R
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Reads the app in front aloud, from its accessibility tree, with the phone's text-to-speech.
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

    @Volatile
    private var reading: Job? = null

    override fun onServiceConnected() {
        setCacheEnabled(false)
        ReadAloud.attach(this)
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) = Unit

    override fun onInterrupt() = stopReading()

    override fun onUnbind(intent: Intent?): Boolean {
        ReadAloud.detach(this)
        scope.cancel()
        return super.onUnbind(intent)
    }

    /** Starts a reading, ending any that is on first. */
    fun read(from: ReadAloud.From) {
        val previous = reading
        reading = scope.launch {
            previous?.cancelAndJoin()
            session(from)
        }
    }

    fun stopReading() {
        reading?.cancel()
    }

    private suspend fun session(from: ReadAloud.From) {
        ReadAloud.reading(true)
        setCacheEnabled(true)
        val speaker = withContext(Dispatchers.Main) {
            stopButton.show()
            Speaker(this@ReadAloudService, onFocusLost = ::stopReading)
        }
        try {
            when {
                !speaker.isReady() -> tell(R.string.read_aloud__no_engine)
                speaker.begin() -> readFrom(from, speaker)
            }
        } finally {
            // Before the switch to Main: a cancelled coroutine throws as that returns, even out of
            // NonCancellable, so nothing after it would run on a stop.
            setCacheEnabled(false)
            ReadAloud.reading(false)
            withContext(NonCancellable + Dispatchers.Main) {
                speaker.shutdown()
                stopButton.hide()
            }
        }
    }

    private suspend fun readFrom(from: ReadAloud.From, speaker: Speaker) {
        var screen = look(waitForApp = from is ReadAloud.From.Selection) ?: return tell(R.string.read_aloud__nothing)
        val start = when (from) {
            ReadAloud.From.Screen -> PageText.firstOnScreen(screen.blocks, screen.visibleTop, screen.visibleBottom) ?: Start(0)
            ReadAloud.From.Top -> {
                screen = toTop(screen)
                Start(0)
            }
            is ReadAloud.From.Selection -> PageText.find(screen.blocks, from.text, screen.visibleTop, screen.visibleBottom)
                ?: return speakOnly(from.text, speaker)
        }
        readOn(speaker, screen, start)
    }

    /**
     * Reads from [start] to the end of the page: a couple of blocks queued ahead, so there is no
     * pause between them, and the list scrolled on once they run out. Each block is brought into
     * sight as it starts; if two start while the page is still moving, only the later one is.
     */
    private suspend fun readOn(speaker: Speaker, first: Screen, start: Start) = coroutineScope {
        val inSight = Channel<TextBlock>(Channel.CONFLATED)
        val follower = launch { for (block in inSight) block.reveal() }
        val read = ArrayDeque<String>()
        var screen = first
        var blocks = first.blocks
        var index = start.block
        var offset = start.offset
        while (true) {
            while (index < blocks.size) {
                val block = blocks[index++]
                speaker.awaitBlocks(LOOKAHEAD - 1)
                speaker.queue(block.text.substring(offset)) { inSight.trySend(block) }
                offset = 0
                if (block.scrolls) read.addLast(block.text)
                if (read.size > PageText.RECENT) read.removeFirst()
            }
            speaker.awaitBlocks(1)
            val next = scrolledOn(screen, read) ?: break
            screen = next.screen
            blocks = next.blocks
            index = next.index
        }
        speaker.awaitBlocks(0)
        follower.cancel()
    }

    private class Resume(val screen: Screen, val blocks: List<TextBlock>, val index: Int)

    /** Scrolls the list on and finds where to carry on; null at the end of the page. */
    private suspend fun scrolledOn(screen: Screen, read: List<String>): Resume? {
        val scroller = screen.scroller ?: return null.also { Log.d(TAG, "Nothing scrolls") }
        if (!scroller.performAction(AccessibilityAction.ACTION_SCROLL_FORWARD.id)) return null.also { Log.d(TAG, "At the end") }
        repeat(TRIES_AFTER_SCROLL) {
            delay(SETTLE_MS)
            val next = look(waitForApp = false)?.takeIf { it.app == screen.app } ?: return null
            val blocks = next.blocks.filter(TextBlock::scrolls)
            val index = PageText.resumeAfter(blocks, read)
            Log.d(TAG, "Scrolled on: carrying on at $index of ${blocks.size}")
            if (index < blocks.size) return Resume(next, blocks, index)
        }
        return null
    }

    /** Back to the top of the page: one jump on a web page, which has all its text; a list scrolls back. */
    private suspend fun toTop(screen: Screen): Screen {
        screen.blocks.firstOrNull(TextBlock::scrolls)?.reveal?.invoke()
        val scroller = screen.scroller ?: return screen
        var scrolls = 0
        while (!screen.web && scrolls < MAX_SCROLLS_BACK && scroller.performAction(AccessibilityAction.ACTION_SCROLL_BACKWARD.id)) {
            scrolls++
            delay(SCROLL_BACK_MS)
        }
        delay(SETTLE_MS)
        return look(waitForApp = false) ?: screen
    }

    private suspend fun speakOnly(text: String, speaker: Speaker) {
        speaker.queue(text)
        speaker.awaitBlocks(0)
    }

    /**
     * The app's text. Tried a few times: a web page builds its tree the first time it is asked,
     * and after "Read from here" the app takes a moment to be back in front.
     */
    private suspend fun look(waitForApp: Boolean): Screen? {
        repeat(LOOK_TRIES) {
            val screen = walk(skipOwn = waitForApp)
            if (screen != null && screen.blocks.isNotEmpty()) {
                return if (screen.web && screen.allShown) settled(screen, waitForApp) else screen
            }
            delay(LOOK_RETRY_MS)
        }
        return null
    }

    /**
     * A web page whose text is all in sight may be a short page, or one whose tree is still filling
     * in: Chrome hands over what is on screen first. It is walked again until it stops growing.
     */
    private suspend fun settled(first: Screen, skipOwn: Boolean): Screen {
        var screen = first
        repeat(SETTLE_TRIES) {
            delay(LOOK_RETRY_MS)
            screen = walk(skipOwn)?.takeIf { it.nodes > screen.nodes } ?: return screen
        }
        return screen
    }

    private fun walk(skipOwn: Boolean): Screen? {
        val started = SystemClock.elapsedRealtime()
        val screen = appWindow(skipOwn)?.let { (root, visible) -> ScreenWalk(visible).walk(root) } ?: return null
        val took = SystemClock.elapsedRealtime() - started
        val inList = screen.blocks.count(TextBlock::scrolls)
        Log.d(TAG, "Walked ${screen.nodes} nodes into ${screen.blocks.size} blocks ($inList in the list) in $took ms")
        return screen
    }

    /** The app the user is in and the part of the screen it shows above the keyboard. */
    private fun appWindow(skipOwn: Boolean): Pair<AccessibilityNodeInfo, Rect>? {
        val all = windows
        val apps = all.filter { it.type == AccessibilityWindowInfo.TYPE_APPLICATION }
        val window = apps.firstOrNull { it.isActive } ?: apps.firstOrNull { it.isFocused } ?: apps.maxByOrNull { it.layer }
        val root = window?.root ?: return null
        if (skipOwn && root.packageName == packageName) return null
        val visible = Rect().also(window::getBoundsInScreen)
        all.firstOrNull { it.type == AccessibilityWindowInfo.TYPE_INPUT_METHOD }?.let { ime ->
            val keyboard = Rect().also(ime::getBoundsInScreen)
            if (keyboard.top > visible.top && keyboard.top < visible.bottom) visible.bottom = keyboard.top
        }
        return root to visible
    }

    private suspend fun tell(@StringRes message: Int) = withContext(Dispatchers.Main) {
        Toast.makeText(this@ReadAloudService, message, Toast.LENGTH_SHORT).show()
    }

    private companion object {
        const val TAG = "ReadAloud"
        const val LOOKAHEAD = 2
        const val LOOK_TRIES = 8
        const val LOOK_RETRY_MS = 250L
        const val SETTLE_TRIES = 4
        const val SETTLE_MS = 350L
        const val TRIES_AFTER_SCROLL = 3
        const val MAX_SCROLLS_BACK = 60
        const val SCROLL_BACK_MS = 120L
    }
}
