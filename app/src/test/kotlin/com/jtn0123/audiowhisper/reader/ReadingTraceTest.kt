package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.ReadingRules
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.yield
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Seeded traces holding a reading whose steps are Rust's ([Reading], reader::reading) to main's
 * ([ReferenceReading]): each session runs both against a fake Android of the same seed, whose
 * screens, scrolls, voices and refusals come from it, and compares every effect in order: the
 * status, the cache, the button, the voice taken and made, each walk, wait, scroll, reveal, block
 * queued and wait for blocks, what is said, the end, and anything thrown. A third of the sessions are cancelled at
 * a random effect (a stop, a new reading, or the service unbound); a session after one is a new
 * Reading on a new fake, as after the service is made again or the process died. Also the window a
 * reading reads, against main's choice over the same windows.
 */
class ReadingTraceTest {
    private class FakeHost(private val random: Random, private val cancelAt: Int) : ReadingHost {
        val effects = mutableListOf<String>()
        var job: Job? = null
        private var walks = 0

        /** A session of a screen that keeps coming up blank: a window not there, or without text. */
        private val blank = random.nextInt(6) == 0

        fun record(effect: String) {
            effects += effect
            if (effects.size == cancelAt) job?.cancel()
        }

        override fun status(reading: Boolean) = record("status $reading")
        override fun cache(on: Boolean) = record("cache $on")

        override suspend fun <T> onMain(block: suspend () -> T): T {
            record("main")
            yield()
            return block().also { yield() }
        }

        override fun showButton() = record("show")
        override fun hideButton() = record("hide")
        override suspend fun takeVoice(): Boolean {
            record("take")
            yield()
            return random.nextInt(4) != 0
        }

        override suspend fun voiceFailedJustNow() = (random.nextInt(6) == 0).also { record("failed? $it") }
        override fun makeVoice(neural: Boolean): Voice = FakeVoice(this, random).also { record("make $neural") }
        override fun stopSpeaking() = record("stop speaking")
        override suspend fun idleVoice() = record("idle").also { yield() }
        override suspend fun tell(message: Int, long: Boolean) = record("tell ${said(message)} $long").also { yield() }
        override suspend fun wait(ms: Long) = record("wait $ms").also { yield() }

        override fun walk(skipOwn: Boolean): Screen? {
            val id = ++walks
            val screen = if (random.nextInt(9) == 0 || blank && random.nextInt(10) < 5) null else screen(id)
            val found = screen?.let { "${it.blocks.size} blocks${if (it.web && it.allShown) " settling" else ""}" }
            record("walk $skipOwn: ${found ?: "none"}")
            return screen
        }

        /** A screen: blocks of a few repeated texts, some in a list, a short page now and then. */
        private fun screen(id: Int): Screen {
            val count = if (random.nextInt(7) == 0 || blank && random.nextInt(10) < 7) 0 else random.nextInt(1, 8)
            val top = random.nextInt(0, 300)
            val bottom = top + random.nextInt(200, 1500)
            val short = random.nextInt(3) == 0
            var y = if (short) top + 10 else random.nextInt(-400, 400)
            val blocks = List(count) { block ->
                val height = if (short) 20 else random.nextInt(20, 300)
                val edges = y to y + height
                y += height + random.nextInt(0, 40)
                TextBlock(TEXTS[random.nextInt(TEXTS.size)], edges.first, edges.second, random.nextInt(3) != 0) {
                    record("reveal $id:$block")
                    yield()
                }
            }
            val scroller = if (random.nextInt(4) == 0) null else Scroller { forward ->
                (random.nextInt(4) != 0).also { record("scroll $forward $id: $it") }
            }
            val app = listOf("a", "a", "a", "b", null)[random.nextInt(5)]
            return Screen(app, blocks, scroller, top, bottom, random.nextInt(0, 60), web = random.nextInt(3) == 0)
        }
    }

    /** A voice that plays what is queued while it is waited on, starting each block in turn. */
    private class FakeVoice(private val host: FakeHost, private val random: Random) : Voice {
        private val queued = ArrayDeque<() -> Unit>()

        override suspend fun unavailable(): Int? {
            yield()
            return (if (random.nextInt(8) == 0) UNAVAILABLE else null).also { host.record("unavailable? $it") }
        }

        override fun begin() = (random.nextInt(10) != 0).also { host.record("begin $it") }

        override fun queue(text: String, onStart: () -> Unit) {
            host.record("queue ${text.length} ${text.hashCode()}")
            queued.addLast(onStart)
            if (queued.size == 1) start()
        }

        override suspend fun awaitBlocks(count: Int) {
            host.record("await $count")
            while (queued.size > count) {
                queued.removeFirst()
                if (queued.isNotEmpty()) start()
            }
            yield()
        }

        private fun start() {
            host.record("start")
            queued.first()()
        }

        override fun shutdown() = host.record("shutdown")
    }

    @Test
    fun readingsMatchMain() {
        val tape = GoldenTape.ofReference("reading-sessions")
        for (seed in SEEDS) {
            val random = Random(seed)
            val seen = HashMap<String, Int>()
            var effects = 0
            var session = 0
            while (effects < EVENTS) {
                val from = from(random)
                val cancelAt = if (random.nextInt(3) == 0) random.nextInt(1, 80) else 0
                val main = session(seed, session, from, cancelAt, reference = true)
                tape.record(main)
                val new = session(seed, session, from, cancelAt, reference = false)
                assertEquals(main, new, "seed $seed session $session")
                count(seen, main, cancelAt)
                effects += main.size
                session++
            }
            println("Reading trace seed $seed: $session sessions, $effects effects, 0 differ, seen $seen")
            for (kind in KINDS) assertTrue((seen[kind] ?: 0) >= MIN_SEEN, "seed $seed saw $kind ${seen[kind]} times")
        }
        tape.finish()
    }

    private fun session(seed: Long, session: Int, from: ReadAloud.From, cancelAt: Int, reference: Boolean): List<String> =
        runBlocking {
            val host = FakeHost(Random(seed * 1_000_003 + session), cancelAt)
            val job = launch {
                try {
                    if (reference) ReferenceReading(host).session(from) else Reading(host).run(from)
                } catch (e: CancellationException) {
                    throw e
                } catch (e: RuntimeException) {
                    host.record("threw ${e::class.simpleName}")
                }
            }
            host.job = job
            job.join()
            host.effects
        }

    private fun from(random: Random): ReadAloud.From = when (random.nextInt(10)) {
        in 0..3 -> ReadAloud.From.Screen
        in 4..6 -> ReadAloud.From.Top
        else -> ReadAloud.From.Selection(SELECTIONS[random.nextInt(SELECTIONS.size)])
    }

    /** What a session did, by the effects that mark each path. */
    private fun count(seen: HashMap<String, Int>, effects: List<String>, cancelAt: Int) {
        fun saw(kind: String) {
            seen[kind] = (seen[kind] ?: 0) + 1
        }
        if (cancelAt in 1..effects.size) saw("cancelled")
        if (effects.any { it.startsWith("tell") && it.endsWith("true") }) saw("unavailable")
        if (effects.contains("tell nothing false")) saw("nothing")
        if (effects.contains("tell voice failed false")) saw("voice failed")
        if (effects.any { it.startsWith("scroll true") }) saw("scrolled on")
        if (effects.any { it.startsWith("scroll false") }) saw("scrolled back")
        if (effects.any { it.startsWith("reveal") }) saw("revealed")
        if (effects.count { it.startsWith("walk ") } > LOOK_TRIES) saw("many walks")
        if (effects.contains("wait 250") && effects.count { it.startsWith("walk ") } > 1) saw("walked again")
        if (effects.zipWithNext().any { (a, b) -> a.endsWith("settling") && b == "wait 250" }) saw("settled")
        val scrolled = effects.indexOfFirst { it.startsWith("scroll true") && it.endsWith("true") }
        if (scrolled >= 0 && effects.drop(scrolled).any { it.startsWith("queue") }) saw("carried on")
        if (effects.any { it.startsWith("queue") }) saw("queued")
        if (effects.contains("status false")) saw("ended")
    }

    @Test
    fun windowsMatchMain() {
        val tape = GoldenTape.ofReference("reading-windows")
        for (seed in SEEDS) {
            val random = Random(seed)
            var chosen = 0
            var cut = 0
            repeat(WINDOW_LISTS) { n ->
                val all = List(random.nextInt(0, 7)) {
                    val top = random.nextInt(-50, 1500)
                    ReferenceWindow.Window(random.nextInt(1, 6), random.nextInt(4) == 0, random.nextInt(4) == 0,
                        random.nextInt(0, 4), top, top + random.nextInt(0, 2400))
                }
                val main = ReferenceWindow.appWindow(all)
                tape.record("$main")
                val (app, keyboard) = ReadingRules.window(
                    BooleanArray(all.size) { all[it].type == android.view.accessibility.AccessibilityWindowInfo.TYPE_APPLICATION },
                    BooleanArray(all.size) { all[it].type == android.view.accessibility.AccessibilityWindowInfo.TYPE_INPUT_METHOD },
                    BooleanArray(all.size) { all[it].isActive }, BooleanArray(all.size) { all[it].isFocused },
                    IntArray(all.size) { all[it].layer })
                val window = all.getOrNull(app)
                val new = window?.let { w -> app to (all.getOrNull(keyboard)?.let { ReadingRules.visibleBottom(w.top, w.bottom, it.top) } ?: w.bottom) }
                assertEquals(main, new, "seed $seed windows $n")
                if (main != null) chosen++
                if (main != null && main.second != all[main.first].bottom) cut++
            }
            println("Window trace seed $seed: $WINDOW_LISTS window lists, $chosen with an app, $cut cut to the keyboard, 0 differ")
            assertTrue(chosen >= MIN_SEEN && cut >= MIN_SEEN && WINDOW_LISTS - chosen >= MIN_SEEN)
        }
        tape.finish()
    }

    private companion object {
        val SEEDS = longArrayOf(20261122, 7, 4_000_000_007)
        const val EVENTS = 50_000
        const val WINDOW_LISTS = 20_000
        const val MIN_SEEN = 20
        const val LOOK_TRIES = 8
        const val UNAVAILABLE = 0x7f0f0001

        /** A message by its name: a resource's id moves whenever a string is added, and goldens keep. */
        fun said(message: Int) = when (message) {
            UNAVAILABLE -> "unavailable"
            dev.patrickgold.florisboard.R.string.read_aloud__nothing -> "nothing"
            dev.patrickgold.florisboard.R.string.read_aloud__voice_failed -> "voice failed"
            else -> "message $message"
        }
        val TEXTS = listOf("alpha beta", "gamma", "delta epsilon", "zeta", "eta theta iota", "kappa", "lambda mu", "nu")
        val SELECTIONS = listOf("beta", "gamma", "epsilon", "theta iota", "omega", "kappa lambda")
        val KINDS = listOf(
            "cancelled", "unavailable", "nothing", "voice failed", "scrolled on", "scrolled back", "revealed",
            "many walks", "walked again", "settled", "carried on", "queued", "ended",
        )
    }
}
