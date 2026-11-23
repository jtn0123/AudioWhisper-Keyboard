package com.jtn0123.audiowhisper.reader

import dev.patrickgold.florisboard.R
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Main's ReadAloudService reading (b1cf45f18): session, readFrom, readOn, scrolledOn, toTop,
 * speakOnly, look and settled, verbatim apart from the calls to Android, each routed through the
 * same [ReadingHost] the Rust-driven [Reading] uses:
 *
 * - `ReadAloud.reading(x)`, `setCacheEnabled(x)`: `host.status(x)`, `host.cache(x)`;
 * - `withContext(Dispatchers.Main)`: `host.onMain`, and `withContext(NonCancellable +
 *   Dispatchers.Main)`: `withContext(NonCancellable) { host.onMain { ... } }`;
 * - `stopButton.show()`, `.hide()`, `stopSpeaking()`: `host.showButton()` and so on;
 * - `voices.take()` returned the engine and `engine?.let { NeuralVoice } ?: Speaker` made the
 *   voice: `host.takeVoice()` says whether there is one and `host.makeVoice(neural)` makes it,
 *   keeping it as `speaking` as the block did;
 * - `voices.failedJustNow()`, `voices.idle()`: `host.voiceFailedJustNow()`, `host.idleVoice()`;
 * - `tell(message, long)`, `delay(ms)`, `walk(skipOwn)`: `host.tell`, `host.wait`, `host.walk`;
 * - `scroller.performAction(ACTION_SCROLL_FORWARD.id)` (and back): `scroller.scroll(true)`;
 * - the `Log.d` lines are left out: logs are not effects.
 */
internal class ReferenceReading(private val host: ReadingHost) {
    suspend fun session(from: ReadAloud.From) {
        host.status(true)
        host.cache(true)
        try {
            host.onMain { host.showButton() }
            val engine = host.takeVoice()
            if (host.voiceFailedJustNow()) host.tell(R.string.read_aloud__voice_failed, false)
            // Kept inside the block: a cancelled withContext throws as it returns, which would lose it.
            val voice = host.onMain { host.makeVoice(engine) }
            val unavailable = voice.unavailable()
            when {
                unavailable != null -> host.tell(unavailable, true)
                voice.begin() -> readFrom(from, voice)
            }
        } finally {
            // Before the switch to Main: a cancelled coroutine throws as that returns, even out of
            // NonCancellable, so nothing after it would run on a stop.
            host.cache(false)
            host.status(false)
            withContext(NonCancellable) {
                host.onMain {
                    host.stopSpeaking()
                    host.hideButton()
                    host.idleVoice()
                }
            }
        }
    }

    private suspend fun readFrom(from: ReadAloud.From, speaker: Voice) {
        var screen = look(waitForApp = from is ReadAloud.From.Selection) ?: return host.tell(R.string.read_aloud__nothing, false)
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

    private suspend fun readOn(speaker: Voice, first: Screen, start: Start) = coroutineScope {
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

    private suspend fun scrolledOn(screen: Screen, read: List<String>): Resume? {
        val scroller = screen.scroller ?: return null
        if (!scroller.scroll(forward = true)) return null
        repeat(TRIES_AFTER_SCROLL) {
            host.wait(SETTLE_MS)
            val next = look(waitForApp = false)?.takeIf { it.app == screen.app } ?: return null
            val blocks = next.blocks.filter(TextBlock::scrolls)
            val index = PageText.resumeAfter(blocks, read)
            if (index < blocks.size) return Resume(next, blocks, index)
        }
        return null
    }

    private suspend fun toTop(screen: Screen): Screen {
        screen.blocks.firstOrNull(TextBlock::scrolls)?.reveal?.invoke()
        val scroller = screen.scroller ?: return screen
        var scrolls = 0
        while (!screen.web && scrolls < MAX_SCROLLS_BACK && scroller.scroll(forward = false)) {
            scrolls++
            host.wait(SCROLL_BACK_MS)
        }
        host.wait(SETTLE_MS)
        return look(waitForApp = false) ?: screen
    }

    private suspend fun speakOnly(text: String, speaker: Voice) {
        speaker.queue(text)
        speaker.awaitBlocks(0)
    }

    private suspend fun look(waitForApp: Boolean): Screen? {
        repeat(LOOK_TRIES) {
            val screen = host.walk(skipOwn = waitForApp)
            if (screen != null && screen.blocks.isNotEmpty()) {
                return if (screen.web && screen.allShown) settled(screen, waitForApp) else screen
            }
            host.wait(LOOK_RETRY_MS)
        }
        return null
    }

    private suspend fun settled(first: Screen, skipOwn: Boolean): Screen {
        var screen = first
        repeat(SETTLE_TRIES) {
            host.wait(LOOK_RETRY_MS)
            screen = host.walk(skipOwn)?.takeIf { it.nodes > screen.nodes } ?: return screen
        }
        return screen
    }

    private companion object {
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

/**
 * Main's ReadAloudService.appWindow choice (b1cf45f18), verbatim over [Window]'s facts in place of
 * AccessibilityWindowInfo's: the chosen window's index and the bottom it shows above the keyboard.
 */
internal object ReferenceWindow {
    /** What appWindow read of a window. */
    class Window(val type: Int, val isActive: Boolean, val isFocused: Boolean, val layer: Int, val top: Int, val bottom: Int)

    fun appWindow(all: List<Window>): Pair<Int, Int>? {
        val apps = all.filter { it.type == android.view.accessibility.AccessibilityWindowInfo.TYPE_APPLICATION }
        val window = apps.firstOrNull { it.isActive } ?: apps.firstOrNull { it.isFocused } ?: apps.maxByOrNull { it.layer }
        window ?: return null
        var bottom = window.bottom
        all.firstOrNull { it.type == android.view.accessibility.AccessibilityWindowInfo.TYPE_INPUT_METHOD }?.let { ime ->
            if (ime.top > window.top && ime.top < bottom) bottom = ime.top
        }
        return all.indexOf(window) to bottom
    }
}
