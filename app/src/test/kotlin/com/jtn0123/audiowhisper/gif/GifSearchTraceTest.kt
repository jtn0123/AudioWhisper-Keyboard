package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.GifRules
import com.jtn0123.audiowhisper.rust.GifSessionPlan
import kotlinx.coroutines.launch
import java.io.File
import java.nio.file.Files
import java.security.MessageDigest
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The GIF panel in Rust (crates/gif's session, behind [GifSearchSession], [GifPreviews] and
 * [GifSend]) does what main's Kotlin did ([ReferenceGifSearch]), step for step: seeded runs of
 * 50,000 events each (opening and closing the panel, typing and swiping in the search, chips,
 * searches answered, refused, cut off, garbled or cancelled by searching again, the clock passing
 * the cache's 10 minutes, previews past the 24 MB held, GIFs sent into apps that take them or
 * don't, part files left by a crash, recents read with GIFs sent meanwhile, damaged recents,
 * process death and the service recreated). After every event both sides' flows, requests,
 * recents file, previews and sent-GIF folder must match. Differences are counted by field and
 * described by hashes, never by their text.
 */
class GifSearchTraceTest {
    @Test fun firstSeed() = run(SEEDS[0])

    @Test fun secondSeed() = run(SEEDS[1])

    @Test fun thirdSeed() = run(SEEDS[2])

    private fun run(seed: Long) {
        val tape = GoldenTape.ofReference("gif-search-trace-$seed")
        val trace = GifTrace(Random(seed), Files.createTempDirectory("gif-trace").toFile(), tape)
        try {
            repeat(EVENTS) { trace.step(it) }
            println("GIF trace seed $seed: $EVENTS events, ${trace.checks} checks, ${trace.failures} differ, seen ${trace.seen}")
            assertEquals(emptyList(), trace.examples, "${trace.failures} of ${trace.checks} checks differ")
            tape.finish()
            for (outcome in OUTCOMES) assertTrue((trace.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${trace.seen[outcome]}: ${trace.seen}")
        } finally {
            trace.root.deleteRecursively()
        }
    }

    private companion object {
        val SEEDS = longArrayOf(20261122L, 7L, 4_000_000_007L)
        const val EVENTS = 50_000
        const val MIN_SEEN = 20
        val OUTCOMES = listOf(
            "fetched", "cache hit", "cancelled answer ignored", "failed status", "failed unreadable",
            "failed unreachable", "failed too large", "fetched again", "recents shown", "recents read with some sent meanwhile",
            "recents damaged", "preview fetched", "preview held", "preview evicted", "preview failed", "sent", "linked",
            "send failed", "gif reused", "part pruned", "process death", "service recreated", "typed word offered",
            "incognito send", "swipe restored", "no source",
        )
    }
}

/** Both sides of one trace, in their own folders. */
private class GifTrace(private val random: Random, val root: File, private val tape: GoldenTape) {
    var failures = 0
    var checks = 0L
    val examples = mutableListOf<String>()
    val seen = sortedMapOf<String, Int>()

    private var elapsed = 1_000L
    private var wall = 1_700_000_000_000L
    private var media: FakeReply = FakeReply.Status(404)
    private val pool = (0 until 45).map(::poolGif)
    private val previewBytes = PREVIEW_SIZES.map { ByteArray(it) { i -> (i * 31).toByte() } }
    private lateinit var old: Old
    private lateinit var new: New
    private var event = 0

    /** When each request's GIFs were fetched, by URL. */
    private val kept = mutableMapOf<String, Long>()

    /** The previews fetched since they were last forgotten. */
    private val heldBefore = mutableSetOf<String>()
    private var name = ""

    private inner class Old {
        val threads = StepThreads()
        val net = FakeNet { media }
        val search = ReferenceGifSearch(threads.scope, threads.io, { elapsed }, net, KEY)
        val files = File(root, "old/files").apply { mkdirs() }
        val gifs = File(root, "old/cache/gifs")
    }

    private inner class New {
        val threads = StepThreads()
        val source = FakeSource(KEY) { media }
        val previews: GifPreviews = GifPreviews(threads.io) { search.source }
        val search: GifSearchSession = GifSearchSession(threads.scope, threads.io, { elapsed }, {}, { previews.forget() }, source)
        val files = File(root, "new/files").apply { mkdirs() }
        val gifs = File(root, "new/cache/gifs")
    }

    init {
        start()
    }

    private fun start() {
        heldBefore.clear()
        old = Old()
        new = New()
        both({ attach(old.files) }, { attach(new.files) })
    }

    fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    private fun both(oldOp: ReferenceGifSearch.() -> Unit, newOp: GifSearchSession.() -> Unit) {
        old.search.oldOp()
        new.search.newOp()
    }

    fun step(index: Int) {
        event = index
        elapsed += GAPS.random(random)
        val roll = random.nextInt(1000)
        when {
            roll < 120 -> panel(random.nextInt(8))
            roll < 330 -> typing(random.nextInt(9))
            roll < 450 -> load()
            roll < 560 -> answer()
            roll < 700 -> drain(random.nextInt(3))
            roll < 760 -> previews()
            roll < 850 -> send()
            roll < 870 -> files(random.nextInt(3))
            roll < 885 -> processDeath()
            roll < 900 -> serviceRecreated()
            roll < 910 -> sourceToggled()
            else -> drain(2)
        }
        compare()
    }

    private fun panel(kind: Int) {
        name = "panel $kind"
        when (kind) {
            0, 1, 2 -> both({ showGifs(true) }, { showGifs(true) })
            3 -> both({ showGifs(false) }, { showGifs(false) })
            4, 5 -> {
                val text = if (random.nextInt(8) == 0) null else words(random.nextInt(4))
                val private = random.nextInt(5) == 0
                both({ offerTypedWord(text, private) }, { offerTypedWord(text, private) })
                see("typed word offered", new.search.chips.value.firstOrNull() !in GifRules.categories + GifRules.recents)
            }
            else -> {
                both({ reset() }, { reset() })
                heldBefore.clear()
            }
        }
    }

    private fun typing(kind: Int) {
        name = "typing $kind"
        when (kind) {
            0 -> both({ startTyping() }, { startTyping() })
            1 -> both({ stopTyping() }, { stopTyping() })
            2, 3 -> words(random.nextInt(3)).let { text -> both({ type(text) }, { type(text) }) }
            4 -> PIECES.random(random).let { word -> both({ typeWord(word) }, { typeWord(word) }) }
            5 -> both({ deleteLast() }, { deleteLast() })
            6 -> both({ deleteWord() }, { deleteWord() })
            7 -> {
                val swipe = GifSearchKeys.SwipeDelete(random.nextInt(-1, 12), random.nextBoolean())
                val starts = random.nextInt(3) == 0
                val before = new.search.query.value
                both({ deleteSwiped(swipe, starts) }, { deleteSwiped(swipe, starts) })
                see("swipe restored", !starts && swipe.count == 0 && new.search.query.value != before)
            }
            else -> both({ submit() }, { submit() })
        }
    }

    private fun load() {
        val search = when (random.nextInt(5)) {
            0 -> GifRules.trending
            1 -> GifRules.recents
            2 -> GifRules.categories.random(random)
            else -> SEARCHES.random(random)
        }
        name = "load"
        val asked = new.source.api.size
        both({ load(search) }, { load(search) })
        see("recents shown", search == GifRules.recents && new.search.available)
        drain(2)
        val fetched = new.source.api.size > asked
        see("cache hit", !fetched && new.search.results.value is GifSearch.Results.Loaded && search != GifRules.recents)
        see("fetched again", fetched && new.source.api.last().url in kept)
    }

    /** Answers one request both sides made, the same way. */
    private fun answer() {
        name = "answer"
        drain(2)
        val open = new.source.api.indices.filter { !new.source.api[it].reply.isCompleted }
        if (open.isEmpty()) return
        val at = if (random.nextInt(10) < 7) open.last() else open.random(random)
        val before = new.search.results.value
        val reply = when (random.nextInt(40)) {
            0, 1 -> FakeReply.Status(STATUSES.random(random))
            2 -> FakeReply.Unreachable
            3 -> FakeReply.Body(200, "{\"data\":".encodeToByteArray())
            4 -> FakeReply.TooLarge
            else -> FakeReply.Body(200, page().encodeToByteArray())
        }
        check("request", old.net.api.getOrNull(at)?.url, new.source.api[at].url)
        old.net.api.getOrNull(at)?.reply?.complete(reply)
        new.source.api[at].reply.complete(reply)
        drain(2)
        val after = new.search.results.value
        see("cancelled answer ignored", after == before && at != new.source.api.lastIndex)
        if (after != before) when (reply) {
            is FakeReply.Status -> see("failed status")
            FakeReply.Unreachable -> see("failed unreachable")
            FakeReply.TooLarge -> see("failed too large")
            is FakeReply.Body -> see(if (after is GifSearch.Results.Failed) "failed unreadable" else "fetched")
        }
        if (after != before && after is GifSearch.Results.Loaded) kept[new.source.api[at].url] = elapsed
    }

    private fun drain(kind: Int) {
        if (name.isEmpty()) name = "drain $kind"
        when (kind) {
            0 -> { old.threads.main.runAll(); new.threads.main.runAll() }
            1 -> { old.threads.io.runAll(); new.threads.io.runAll() }
            else -> { old.threads.drain(); new.threads.drain() }
        }
    }

    /** A burst of previews, as when the panel scrolls: enough to pass the 24 MB held. */
    private fun previews() {
        name = "preview"
        if (new.search.available) repeat(random.nextInt(1, 16)) { preview() }
    }

    private fun preview() {
        val gif = pool[random.nextInt(PREVIEWED)]
        val bytes = previewBytes.random(random)
        media = if (random.nextInt(12) == 0) FakeReply.Status(404) else FakeReply.Body(200, bytes)
        val asked = new.source.mediaAsked.size
        var oldOutcome = ""
        var newOutcome = ""
        old.threads.scope.launch { oldOutcome = outcome { old.search.images.previewBytes(gif.previewUrl) } }
        new.threads.scope.launch { newOutcome = outcome { new.previews.bytes(gif.previewUrl) } }
        drain(2)
        check("preview", oldOutcome, newOutcome)
        check("preview fetched", old.net.mediaAsked.size, new.source.mediaAsked.size)
        val fetched = new.source.mediaAsked.size > asked
        see(if (fetched) "preview fetched" else "preview held")
        see("preview failed", newOutcome.startsWith("!"))
        see("preview evicted", fetched && gif.previewUrl in heldBefore)
        if (!newOutcome.startsWith("!")) heldBefore += gif.previewUrl
    }

    private suspend fun outcome(fetch: suspend () -> ByteArray): String = try {
        fetch().let { "${it.size} ${hash(it)}" }
    } catch (e: Exception) {
        "!" + e.javaClass.simpleName
    }

    private fun send() {
        name = "send"
        if (!new.search.available) return see("no source")
        val gif = pool.random(random)
        val accepts = random.nextInt(5) != 0
        val takes = random.nextInt(6) != 0
        val text = random.nextInt(4) != 0
        media = when (random.nextInt(30)) {
            0, 1 -> FakeReply.Status(404)
            2 -> FakeReply.Unreachable
            3 -> if (random.nextInt(8) == 0) FakeReply.TooLarge else FakeReply.Unreachable
            else -> FakeReply.Body(200, previewBytes[1 + gif.id.length % 3])
        }
        val oldTarget = FakeTarget(accepts, takes, text)
        val newTarget = FakeTarget(accepts, takes, text)
        var oldOutcome: GifSender.Outcome? = null
        var newOutcome: GifSender.Outcome? = null
        val reused = File(new.gifs, GifSessionPlanFiles.name(gif)).exists()
        old.threads.scope.launch { oldOutcome = old.search.send(old.gifs, wall, oldTarget, gif) }
        new.threads.scope.launch { newOutcome = GifSend(new.gifs, new.threads.io, { wall }, new.source, newTarget) {}.send(gif) }
        drain(2)
        normalize(old.gifs)
        normalize(new.gifs)
        check("send outcome", oldOutcome, newOutcome)
        check("send calls", oldTarget.calls, newTarget.calls)
        see("gif reused", reused && accepts && newOutcome == GifSender.Outcome.SENT)
        see(when (newOutcome) { GifSender.Outcome.SENT -> "sent"; GifSender.Outcome.LINKED -> "linked"; else -> "send failed" })
        val incognito = random.nextInt(8) == 0
        see("incognito send", incognito && newOutcome != GifSender.Outcome.FAILED)
        if (newOutcome != GifSender.Outcome.FAILED) both({ sent(gif, incognito) }, { sent(gif, incognito) })
    }

    /** A file written during an event was written at [wall]. */
    private fun normalize(dir: File) {
        dir.listFiles()?.forEach { if (it.lastModified() > wall) it.setLastModified(wall) }
    }

    private fun files(kind: Int) {
        name = "files $kind"
        when (kind) {
            0 -> wall += WALL_GAPS.random(random)
            1 -> {
                val gif = pool.random(random)
                val age = random.nextLong(0, 120_000)
                for (dir in listOf(old.gifs, new.gifs)) {
                    dir.mkdirs()
                    File(dir, GifSessionPlanFiles.name(gif) + ".part").apply { writeText("cut"); setLastModified(wall - age) }
                }
                see("part pruned", age > 60_000)
            }
            else -> {
                val damage = random.nextBoolean()
                for (dir in listOf(old.files, new.files)) {
                    val file = File(dir, GifSearchSession.RECENTS_FILE)
                    if (damage) file.writeText("[{\"id\": 3") else file.delete()
                }
                see("recents damaged", damage)
                processDeath()
            }
        }
    }

    private fun processDeath() {
        name = "process death"
        see("process death")
        start()
        if (random.nextBoolean()) {
            // The read starts before the send's save, or (main's race, kept) after it.
            val readFirst = random.nextBoolean()
            if (readFirst) drain(0)
            val gif = pool.random(random)
            both({ sent(gif, false) }, { sent(gif, false) })
            see("recents read with some sent meanwhile", readFirst && File(new.files, GifSearchSession.RECENTS_FILE).length() > 2)
        }
    }

    private fun serviceRecreated() {
        name = "service recreated"
        see("service recreated")
        both({ reset(); attach(old.files) }, { reset(); attach(new.files) })
        heldBefore.clear()
    }

    private fun sourceToggled() {
        name = "source"
        val on = !new.search.available
        old.search.source = if (on) old.search.Giphy(KEY) else null
        new.search.source = if (on) new.source else null
        see("no source", !on)
    }

    private fun compare() {
        val o = old.search
        val n = new.search
        check("showingGifs", o.showingGifs.value, n.showingGifs.value)
        check("typing", o.typing.value, n.typing.value)
        check("query", o.query.value, n.query.value)
        check("shown", o.shown.value, n.shown.value)
        check("results", o.results.value, n.results.value)
        check("chips", o.chips.value, n.chips.value)
        check("available", o.available, n.available)
        check("crashes", old.threads.crashes.map { it.javaClass.name }, new.threads.crashes.map { it.javaClass.name })
        check("coroutines", old.threads.active(), new.threads.active())
        check("requests", old.net.api.map { it.url }, new.source.api.map { it.url })
        check("recents file", fileState(old.files), fileState(new.files))
        check("sent folder", folderState(old.gifs), folderState(new.gifs))
        name = ""
    }

    private fun fileState(dir: File) = dir.listFiles().orEmpty().sortedBy { it.name }.map { it.name to hash(it.readBytes()) }

    private fun folderState(dir: File) =
        dir.listFiles().orEmpty().sortedBy { it.name }.map { listOf(it.name, it.length(), it.lastModified()) }

    private fun check(what: String, old: Any?, new: Any?) {
        tape.record("$what: $old")
        checks++
        if (old == new) return
        failures++
        if (examples.size < 10) examples += "event $event ($name): $what differs, ${hash(old)} vs ${hash(new)}"
    }

    private fun hash(value: Any?): String = hash(value.toString().encodeToByteArray())

    private fun hash(bytes: ByteArray): String =
        MessageDigest.getInstance("SHA-256").digest(bytes).take(4).joinToString("") { "%02x".format(it) }

    private fun words(count: Int) = (0 until count).joinToString("") { PIECES.random(random) }

    /** A page of GIPHY results, some repeated, some without what a GIF needs. */
    private fun page(): String = (0 until random.nextInt(31)).joinToString(",", "{\"data\":[", "]}") {
        val gif = pool.random(random)
        val title = gif.title.replace("\\", "\\\\").replace("\"", "\\\"")
        if (random.nextInt(25) == 0) "{\"id\":\"${gif.id}\"}"
        else "{\"id\":\"${gif.id}\",\"title\":\"$title\",\"url\":\"${gif.pageUrl}\",\"images\":{\"fixed_width\":" +
            "{\"url\":\"${gif.previewUrl}\",\"width\":\"${gif.id.length + 100}\",\"height\":\"100\"},\"downsized\":{\"url\":\"${gif.sendUrl}\"}}}"
    }

    private fun poolGif(index: Int): Gif {
        val id = IDS[index % IDS.size] + if (index >= IDS.size) "$index" else ""
        val page = if (index % 7 == 3) "" else "https://giphy.com/gifs/g$index"
        val ratio = (id.length + 100) / 100f
        return Gif(id, "Title $index", "https://media.giphy.com/p$index/200w.gif", ratio, "https://media.giphy.com/s$index/d.gif", page)
    }

    private companion object {
        const val KEY = "trace-key"
        const val PREVIEWED = 20
        val IDS = listOf("a", "b", "c", "abc", "A", "x/y", "../", "", "é1", "中文", "z_z", "g1", "Q")
        val PIECES = listOf("cat", "Congrats", " ", "  ", "dance", "OK", "x", "don't", "é", "😀", "\uD800", "-", "!", "y".repeat(30), "\t")
        val SEARCHES = listOf("cats", "Thank you", "thank you ", "haha", "İstanbul", "STRASSE", "dogs", "a", "")
        val STATUSES = listOf(401, 403, 429, 500, 404, 201)
        val GAPS = listOf(0L, 0L, 1L, 10L, 500L, 1_000L, 30_000L, 120_000L, 599_999L, 600_000L)
        val WALL_GAPS = listOf(1_000L, 59_000L, 61_000L, 3_600_000L, 86_400_000L, 86_400_001L)
        val PREVIEW_SIZES = listOf(0, 1, 9_000, 300_000, 2_000_000, 3_500_000, 4 * 1024 * 1024, 4 * 1024 * 1024 + 1)
    }
}

/** The file a GIF is sent from, as Rust names it. */
private object GifSessionPlanFiles {
    fun name(gif: Gif): String = com.jtn0123.audiowhisper.rust.GifSessionPlan.sendFiles(gif.id)[0]
}
