package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.ReadRoute
import com.jtn0123.audiowhisper.rust.ReadingRules
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.yield
import kotlin.coroutines.resume
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Seeded traces of the neural voice's hold, now Rust's ([LoadedVoice] over reader::lifetime),
 * checked against main's Kotlin hold's (golden/voice-hold-traces.txt): random readings taking the
 * voice, asking whether it failed and going idle, the clock moving on, memory wanted, and the
 * service made again (its scope cancelled, the hold closed, a new one made), each launched so they
 * contend for the hold's lock as they would. Every call to Android is an effect, in order: the
 * files checked, a load and whether it gave a voice, each voice closed, each wait, and what each
 * reading was told. Also read-aloud's routing of a reading asked for, against main's
 * (golden/read-aloud-routes.txt).
 */
class LoadedVoiceTraceTest {
    private class FakeEngine(private val effects: MutableList<String>, val id: Int) : AutoCloseable {
        override fun close() {
            effects += "close $id"
        }
    }

    /** Android, with a clock the trace moves on: waits resume once it passes their end. */
    private class FakeSource(private val random: Random) : VoiceSource<FakeEngine> {
        val effects = mutableListOf<String>()
        var evict: (() -> Unit)? = null
        private var now = 0L
        private var engines = 0
        private val timers = mutableListOf<Pair<Long, CancellableContinuation<Unit>>>()

        override fun missing() = (random.nextInt(5) == 0).also { effects += "missing? $it" }
        override fun loading() {
            effects += "loading"
        }

        override suspend fun load(): FakeEngine? {
            effects += "load"
            yield()
            return (if (random.nextInt(4) == 0) null else FakeEngine(effects, ++engines)).also { effects += "loaded ${it?.id}" }
        }

        override fun register(evict: () -> Unit) {
            effects += "register"
            this.evict = evict
        }

        override fun unregister() {
            effects += "unregister"
            evict = null
        }

        override suspend fun wait(ms: Long) {
            effects += "wait $ms"
            suspendCancellableCoroutine { continuation ->
                val timer = now + ms to continuation
                timers += timer
                continuation.invokeOnCancellation { timers.remove(timer) }
            }
        }

        fun advance(by: Long) {
            now += by
            val due = timers.filter { it.first <= now }
            timers.removeAll(due)
            due.forEach { it.second.resume(Unit) }
        }
    }

    /** One seed's events, run on the hold: what Android saw. */
    private fun trace(seed: Long): List<String> = runBlocking {
        val events = Random(seed)
        val source = FakeSource(Random(seed * 31 + 1))
        var scope = CoroutineScope(coroutineContext + Job(coroutineContext[Job]))
        fun make(scope: CoroutineScope) = LoadedVoice(source, scope)
        var voice = make(scope)
        repeat(EVENTS) {
            when (events.nextInt(20)) {
                in 0..5 -> scope.launch { source.effects += "took ${voice.take()?.id}" }.also { source.effects += "take" }
                in 6..7 -> scope.launch { source.effects += "failed ${voice.failedJustNow()}" }
                in 8..12 -> scope.launch { voice.idle() }.also { source.effects += "idle" }
                in 13..16 -> source.advance(events.nextLong(0, 240_000)).also { source.effects += "advance" }
                in 17..18 -> source.evict?.invoke().also { source.effects += "evict" }
                else -> {
                    source.effects += "recreate"
                    scope.cancel()
                    voice.close()
                    scope = CoroutineScope(coroutineContext + Job(coroutineContext[Job]))
                    voice = make(scope)
                }
            }
            repeat(events.nextInt(0, 4)) { yield() }
        }
        scope.cancel()
        voice.close()
        source.effects
    }

    @Test
    fun holdsMatchMain() = GoldenTape.check("voice-hold-traces") { tape ->
        for (seed in SEEDS) {
            val effects = trace(seed)
            tape.record(effects)
            val seen = seen(effects)
            println("Voice hold trace seed $seed: $EVENTS events, ${effects.size} effects, seen $seen")
            for ((kind, count) in seen) assertTrue(count >= MIN_SEEN, "seed $seed saw $kind $count times")
        }
    }

    /** What the trace did, by the effects that mark each path. */
    private fun seen(effects: List<String>): Map<String, Int> {
        fun after(a: String, gap: Int = 1, b: (String) -> Boolean) =
            effects.indices.count { it + gap < effects.size && effects[it] == a && b(effects[it + gap]) }
        return mapOf(
            "loaded" to effects.count { it.startsWith("loaded ") && it != "loaded null" },
            "load failed" to effects.count { it == "loaded null" },
            "failure said" to effects.count { it == "failed true" },
            "missing" to effects.count { it == "missing? true" },
            "used loaded" to effects.count { it.startsWith("took ") && it != "took null" } - effects.count { it == "load" },
            "freed after idle" to after("advance") { it.startsWith("close") },
            "evicted" to after("evict") { it.startsWith("close") },
            "closed on recreate" to after("recreate", gap = 2) { it.startsWith("close") },
            "recreated" to effects.count { it == "recreate" },
        )
    }

    @Test
    fun routesMatchMain() = GoldenTape.check("read-aloud-routes") { tape ->
        for (up in listOf(false, true)) for (on in listOf(false, true)) {
            val route = when (val code = ReadingRules.route(up, on)) {
                ReadRoute.READ -> "read"
                ReadRoute.ASK -> "ask"
                ReadRoute.WAIT -> "wait"
                else -> "route $code"
            }
            tape.record("service up $up, turned on $on: $route")
        }
        for (seed in SEEDS) {
            val random = Random(seed)
            var still = 0
            repeat(ASKS) {
                val at = when (random.nextInt(10)) {
                    0 -> random.nextLong()
                    else -> random.nextLong(0, 1L shl 40)
                }
                val now = when (random.nextInt(10)) {
                    0 -> random.nextLong()
                    1 -> at + 4_999 + random.nextLong(0, 3)
                    else -> at + random.nextLong(-1_000, 12_000)
                }
                val asked = ReadingRules.stillAsked(at, now)
                tape.record("$asked")
                if (asked) still++
            }
            println("Service wait trace seed $seed: 4 routes, $ASKS asks, $still still wanted")
            assertTrue(still >= MIN_SEEN && ASKS - still >= MIN_SEEN)
        }
    }

    private companion object {
        val SEEDS = longArrayOf(20261122, 7, 4_000_000_007)
        const val EVENTS = 50_000
        const val ASKS = 20_000
        const val MIN_SEEN = 20
    }
}
