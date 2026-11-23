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
import kotlin.test.assertTrue

/**
 * The GIF panel in Rust (crates/gif's session, behind [GifSearchSession], [GifPreviews] and
 * [GifSend]) does what main's Kotlin did, step for step, as recorded in the golden files
 * `gif-search-trace-<seed>` (docs/phase6-kotlin-references.md): seeded runs of 50,000 events each
 * (opening and closing the panel, typing and swiping in the search, chips, searches answered,
 * refused, cut off, garbled or cancelled by searching again, the clock passing the cache's 10
 * minutes, previews past the 24 MB held, GIFs sent into apps that take them or don't, part files
 * left by a crash, recents read with GIFs sent meanwhile, damaged recents, process death and the
 * service recreated). After every event the flows, requests, recents file, previews and sent-GIF
 * folder are recorded; a difference is reported by the range of answers it falls in, never by
 * their text.
 */
class GifSearchTraceTest {
    @Test fun firstSeed() = run(SEEDS[0])

    @Test fun secondSeed() = run(SEEDS[1])

    @Test fun thirdSeed() = run(SEEDS[2])

    private fun run(seed: Long) {
        val root = Files.createTempDirectory("gif-trace").toFile()
        try {
            val trace = GoldenTape.check("gif-search-trace-$seed") { tape ->
                GifTrace(Random(seed), root, tape).apply { repeat(EVENTS) { step() } }
            }
            println("GIF trace seed $seed: $EVENTS events, seen ${trace.seen}")
            for (outcome in OUTCOMES) assertTrue((trace.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${trace.seen[outcome]}: ${trace.seen}")
        } finally {
            root.deleteRecursively()
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

/** One trace of the panel, in its own folder. */
private class GifTrace(private val random: Random, private val root: File, private val tape: GoldenTape) {
    val seen = sortedMapOf<String, Int>()

    private var elapsed = 1_000L
    private var wall = 1_700_000_000_000L
    private var media: FakeReply = FakeReply.Status(404)
    private val pool = (0 until 45).map(::poolGif)
    private val previewBytes = PREVIEW_SIZES.map { ByteArray(it) { i -> (i * 31).toByte() } }
    private lateinit var app: App

    /** When each request's GIFs were fetched, by URL. */
    private val kept = mutableMapOf<String, Long>()

    /** The previews fetched since they were last forgotten. */
    private val heldBefore = mutableSetOf<String>()

    /** The panel as the service makes it, with its files and cache folders, which outlive it. */
    private inner class App {
        val threads = StepThreads()
        val source = FakeSource(KEY) { media }
        val previews: GifPreviews = GifPreviews(threads.io) { search.source }
        val search: GifSearchSession = GifSearchSession(threads.scope, threads.io, { elapsed }, {}, { previews.forget() }, source)
        val files = File(root, "files").apply { mkdirs() }
        val gifs = File(root, "cache/gifs")
    }

    init {
        start()
    }

    private fun start() {
        heldBefore.clear()
        app = App()
        app.search.attach(app.files)
    }

    fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    fun step() {
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
        when (kind) {
            0, 1, 2 -> app.search.showGifs(true)
            3 -> app.search.showGifs(false)
            4, 5 -> {
                val text = if (random.nextInt(8) == 0) null else words(random.nextInt(4))
                val private = random.nextInt(5) == 0
                app.search.offerTypedWord(text, private)
                see("typed word offered", app.search.chips.value.firstOrNull() !in GifRules.categories + GifRules.recents)
            }
            else -> {
                app.search.reset()
                heldBefore.clear()
            }
        }
    }

    private fun typing(kind: Int) {
        when (kind) {
            0 -> app.search.startTyping()
            1 -> app.search.stopTyping()
            2, 3 -> words(random.nextInt(3)).let { text -> app.search.type(text) }
            4 -> PIECES.random(random).let { word -> app.search.typeWord(word) }
            5 -> app.search.deleteLast()
            6 -> app.search.deleteWord()
            7 -> {
                val swipe = GifSearchKeys.SwipeDelete(random.nextInt(-1, 12), random.nextBoolean())
                val starts = random.nextInt(3) == 0
                val before = app.search.query.value
                app.search.deleteSwiped(swipe, starts)
                see("swipe restored", !starts && swipe.count == 0 && app.search.query.value != before)
            }
            else -> app.search.submit()
        }
    }

    private fun load() {
        val search = when (random.nextInt(5)) {
            0 -> GifRules.trending
            1 -> GifRules.recents
            2 -> GifRules.categories.random(random)
            else -> SEARCHES.random(random)
        }
        val asked = app.source.api.size
        app.search.load(search)
        see("recents shown", search == GifRules.recents && app.search.available)
        drain(2)
        val fetched = app.source.api.size > asked
        see("cache hit", !fetched && app.search.results.value is GifSearch.Results.Loaded && search != GifRules.recents)
        see("fetched again", fetched && app.source.api.last().url in kept)
    }

    /** Answers one request the panel made. */
    private fun answer() {
        drain(2)
        val open = app.source.api.indices.filter { !app.source.api[it].reply.isCompleted }
        if (open.isEmpty()) return
        val at = if (random.nextInt(10) < 7) open.last() else open.random(random)
        val before = app.search.results.value
        val reply = when (random.nextInt(40)) {
            0, 1 -> FakeReply.Status(STATUSES.random(random))
            2 -> FakeReply.Unreachable
            3 -> FakeReply.Body(200, "{\"data\":".encodeToByteArray())
            4 -> FakeReply.TooLarge
            else -> FakeReply.Body(200, page().encodeToByteArray())
        }
        record("request", app.source.api[at].url)
        app.source.api[at].reply.complete(reply)
        drain(2)
        val after = app.search.results.value
        see("cancelled answer ignored", after == before && at != app.source.api.lastIndex)
        if (after != before) when (reply) {
            is FakeReply.Status -> see("failed status")
            FakeReply.Unreachable -> see("failed unreachable")
            FakeReply.TooLarge -> see("failed too large")
            is FakeReply.Body -> see(if (after is GifSearch.Results.Failed) "failed unreadable" else "fetched")
        }
        if (after != before && after is GifSearch.Results.Loaded) kept[app.source.api[at].url] = elapsed
    }

    private fun drain(kind: Int) {
        when (kind) {
            0 -> app.threads.main.runAll()
            1 -> app.threads.io.runAll()
            else -> app.threads.drain()
        }
    }

    /** A burst of previews, as when the panel scrolls: enough to pass the 24 MB held. */
    private fun previews() {
        if (app.search.available) repeat(random.nextInt(1, 16)) { preview() }
    }

    private fun preview() {
        val gif = pool[random.nextInt(PREVIEWED)]
        val bytes = previewBytes.random(random)
        media = if (random.nextInt(12) == 0) FakeReply.Status(404) else FakeReply.Body(200, bytes)
        val asked = app.source.mediaAsked.size
        var previewed = ""
        app.threads.scope.launch { previewed = outcome { app.previews.bytes(gif.previewUrl) } }
        drain(2)
        record("preview", previewed)
        record("preview fetched", app.source.mediaAsked.size)
        val fetched = app.source.mediaAsked.size > asked
        see(if (fetched) "preview fetched" else "preview held")
        see("preview failed", previewed.startsWith("!"))
        see("preview evicted", fetched && gif.previewUrl in heldBefore)
        if (!previewed.startsWith("!")) heldBefore += gif.previewUrl
    }

    private suspend fun outcome(fetch: suspend () -> ByteArray): String = try {
        fetch().let { "${it.size} ${hash(it)}" }
    } catch (e: Exception) {
        "!" + e.javaClass.simpleName
    }

    private fun send() {
        if (!app.search.available) return see("no source")
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
        val target = FakeTarget(accepts, takes, text)
        var sendOutcome: GifSender.Outcome? = null
        val reused = File(app.gifs, GifSessionPlanFiles.name(gif)).exists()
        app.threads.scope.launch { sendOutcome = GifSend(app.gifs, app.threads.io, { wall }, app.source, target) {}.send(gif) }
        drain(2)
        normalize(app.gifs)
        record("send outcome", sendOutcome)
        record("send calls", target.calls)
        see("gif reused", reused && accepts && sendOutcome == GifSender.Outcome.SENT)
        see(when (sendOutcome) { GifSender.Outcome.SENT -> "sent"; GifSender.Outcome.LINKED -> "linked"; else -> "send failed" })
        val incognito = random.nextInt(8) == 0
        see("incognito send", incognito && sendOutcome != GifSender.Outcome.FAILED)
        if (sendOutcome != GifSender.Outcome.FAILED) app.search.sent(gif, incognito)
    }

    /** A file written during an event was written at [wall]. */
    private fun normalize(dir: File) {
        dir.listFiles()?.forEach { if (it.lastModified() > wall) it.setLastModified(wall) }
    }

    private fun files(kind: Int) {
        when (kind) {
            0 -> wall += WALL_GAPS.random(random)
            1 -> {
                val gif = pool.random(random)
                val age = random.nextLong(0, 120_000)
                app.gifs.mkdirs()
                File(app.gifs, GifSessionPlanFiles.name(gif) + ".part").apply { writeText("cut"); setLastModified(wall - age) }
                see("part pruned", age > 60_000)
            }
            else -> {
                val damage = random.nextBoolean()
                val file = File(app.files, GifSearchSession.RECENTS_FILE)
                if (damage) file.writeText("[{\"id\": 3") else file.delete()
                see("recents damaged", damage)
                processDeath()
            }
        }
    }

    private fun processDeath() {
        see("process death")
        start()
        if (random.nextBoolean()) {
            // The read starts before the send's save, or (main's race, kept) after it.
            val readFirst = random.nextBoolean()
            if (readFirst) drain(0)
            val gif = pool.random(random)
            app.search.sent(gif, false)
            see("recents read with some sent meanwhile", readFirst && File(app.files, GifSearchSession.RECENTS_FILE).length() > 2)
        }
    }

    private fun serviceRecreated() {
        see("service recreated")
        app.search.reset()
        app.search.attach(app.files)
        heldBefore.clear()
    }

    private fun sourceToggled() {
        val on = !app.search.available
        app.search.source = if (on) app.source else null
        see("no source", !on)
    }

    private fun compare() {
        val search = app.search
        record("showingGifs", search.showingGifs.value)
        record("typing", search.typing.value)
        record("query", search.query.value)
        record("shown", search.shown.value)
        record("results", search.results.value)
        record("chips", search.chips.value)
        record("available", search.available)
        record("crashes", app.threads.crashes.map { it.javaClass.name })
        record("coroutines", app.threads.active())
        record("requests", app.source.api.map { it.url })
        record("recents file", fileState(app.files))
        record("sent folder", folderState(app.gifs))
    }

    private fun fileState(dir: File) = dir.listFiles().orEmpty().sortedBy { it.name }.map { it.name to hash(it.readBytes()) }

    private fun folderState(dir: File) =
        dir.listFiles().orEmpty().sortedBy { it.name }.map { listOf(it.name, it.length(), it.lastModified()) }

    private fun record(what: String, answer: Any?) = tape.record("$what: $answer")

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
