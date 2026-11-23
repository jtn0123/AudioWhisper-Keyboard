package com.jtn0123.audiowhisper.reader

import androidx.annotation.StringRes
import com.jtn0123.audiowhisper.rust.ReadingEffect
import com.jtn0123.audiowhisper.rust.ReadingFact
import com.jtn0123.audiowhisper.rust.ReadingFrom
import com.jtn0123.audiowhisper.rust.ReadingMessage
import com.jtn0123.audiowhisper.rust.ReadingPlan
import dev.patrickgold.florisboard.R
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** The list that scrolls: a page on, or back; false if it didn't move. */
internal fun interface Scroller {
    fun scroll(forward: Boolean): Boolean
}

/** What a reading does to Android: [ReadAloudService]'s, or a test's. */
internal interface ReadingHost {
    /** The reading status the keyboard shows. */
    fun status(reading: Boolean)

    /** Android's node cache, kept only while a reading is on. */
    fun cache(on: Boolean)

    suspend fun <T> onMain(block: suspend () -> T): T

    /** Main thread. */
    fun showButton()

    /** Main thread. */
    fun hideButton()

    /** Takes the neural voice, loading it if need be; whether there is one. */
    suspend fun takeVoice(): Boolean

    /** Whether the last [takeVoice] found the voice won't load; true once per failure. */
    suspend fun voiceFailedJustNow(): Boolean

    /** The reading's voice, the neural one or the phone's; main thread. */
    fun makeVoice(neural: Boolean): Voice

    /** Ends the reading's voice for good, once; main thread. */
    fun stopSpeaking()

    /** A reading is over: frees the neural voice unless another starts soon. */
    suspend fun idleVoice()

    suspend fun tell(@StringRes message: Int, long: Boolean)

    /** The app's text, skipping this app's own window if [skipOwn]; null if there is no window. */
    fun walk(skipOwn: Boolean): Screen?

    suspend fun wait(ms: Long)
}

/**
 * One reading of the page aloud. Its steps are Rust's ([ReadingPlan], reader::reading): this runs
 * each effect on [host] and answers it, keeping the screens' Android objects and the voice. The
 * end of a reading is here, since it must run once the reading is cancelled: the cache off, then
 * on the main thread the voice shut down, the button hidden and the neural voice let go.
 */
internal class Reading(private val host: ReadingHost) {
    private var walked: Screen? = null
    private var screen: Screen? = null
    private var voice: Voice? = null

    @StringRes
    private var unavailable: Int? = null
    private val inSight = Channel<TextBlock>(Channel.CONFLATED)
    private var follower: Job? = null

    suspend fun run(from: ReadAloud.From) {
        host.status(true)
        host.cache(true)
        try {
            ReadingPlan().use { plan -> coroutineScope { steps(plan, from) } }
        } finally {
            // Before the switch to the main thread: a cancelled coroutine throws as that returns,
            // even out of NonCancellable, so nothing after it would run on a stop.
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

    private suspend fun CoroutineScope.steps(plan: ReadingPlan, from: ReadAloud.From) {
        val selection = (from as? ReadAloud.From.Selection)?.text
        var effect = plan.start(code(from), selection)
        while (effect[0] != ReadingEffect.DONE) {
            effect = when (effect[0]) {
                ReadingEffect.TAKE_VOICE, ReadingEffect.MAKE_VOICE, ReadingEffect.BEGIN,
                ReadingEffect.SCROLL_BACK, ReadingEffect.SCROLL_FORWARD -> plan.answer(ask(effect))
                ReadingEffect.WALK -> look(plan, skipOwn = effect[1] == 1)
                else -> {
                    run(effect, selection)
                    plan.done()
                }
            }
        }
    }

    /** Runs an effect that asks a question, and answers it. */
    private suspend fun ask(effect: IntArray): Int = when (effect[0]) {
        ReadingEffect.TAKE_VOICE -> bit(host.takeVoice()) or 2 * bit(host.voiceFailedJustNow())
        ReadingEffect.MAKE_VOICE -> {
            // Kept inside the block: a cancelled onMain throws as it returns, which would lose it.
            host.onMain { host.makeVoice(neural = effect[1] == 1).also { voice = it } }
            unavailable = voice().unavailable()
            bit(unavailable != null)
        }
        ReadingEffect.BEGIN -> bit(voice().begin())
        else -> bit(checkNotNull(screen().scroller).scroll(forward = effect[0] == ReadingEffect.SCROLL_FORWARD))
    }

    private suspend fun CoroutineScope.run(effect: IntArray, selection: String?) {
        when (effect[0]) {
            ReadingEffect.SHOW_BUTTON -> host.onMain { host.showButton() }
            ReadingEffect.TELL -> host.tell(message(effect[1]), long = effect[2] == 1)
            ReadingEffect.WAIT -> host.wait(effect[1].toLong())
            ReadingEffect.KEEP -> screen = walked
            ReadingEffect.REVEAL -> screen().blocks[effect[1]].reveal()
            ReadingEffect.FOLLOW -> follower = launch { for (block in inSight) block.reveal() }
            ReadingEffect.UNFOLLOW -> follower?.cancel()
            ReadingEffect.AWAIT_BLOCKS -> voice().awaitBlocks(effect[1])
            ReadingEffect.QUEUE -> screen().blocks[effect[1]].let { block ->
                voice().queue(block.text.substring(effect[2])) { inSight.trySend(block) }
            }
            ReadingEffect.SPEAK_SELECTION -> voice().queue(checkNotNull(selection))
            else -> error("No reading effect ${effect[0]}")
        }
    }

    /** Walks the app's window and tells the plan what it found. */
    private fun look(plan: ReadingPlan, skipOwn: Boolean): IntArray {
        val found = host.walk(skipOwn).also { walked = it } ?: return plan.lookedNothing()
        val blocks = found.blocks
        val facts = IntArray(ReadingFact.SIZE)
        facts[ReadingFact.VISIBLE_TOP] = found.visibleTop
        facts[ReadingFact.VISIBLE_BOTTOM] = found.visibleBottom
        facts[ReadingFact.NODES] = found.nodes
        facts[ReadingFact.WEB] = bit(found.web)
        facts[ReadingFact.SCROLLER] = bit(found.scroller != null)
        return plan.looked(found.app, Array(blocks.size) { blocks[it].text }, PageText.edges(blocks),
            BooleanArray(blocks.size) { blocks[it].scrolls }, facts)
    }

    @StringRes
    private fun message(code: Int): Int = when (code) {
        ReadingMessage.VOICE_FAILED -> R.string.read_aloud__voice_failed
        ReadingMessage.NOTHING -> R.string.read_aloud__nothing
        else -> checkNotNull(unavailable)
    }

    private fun code(from: ReadAloud.From) = when (from) {
        ReadAloud.From.Screen -> ReadingFrom.SCREEN
        ReadAloud.From.Top -> ReadingFrom.TOP
        is ReadAloud.From.Selection -> ReadingFrom.SELECTION
    }

    private fun voice() = checkNotNull(voice)

    private fun screen() = checkNotNull(screen)

    private fun bit(value: Boolean) = if (value) 1 else 0
}
