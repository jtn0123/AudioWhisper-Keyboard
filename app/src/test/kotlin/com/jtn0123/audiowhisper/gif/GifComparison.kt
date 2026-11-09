package com.jtn0123.audiowhisper.gif

import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyType
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.File
import java.io.IOException
import java.net.SocketTimeoutException
import java.util.Locale
import kotlin.random.Random

/**
 * Holds the GIF panel's rules in Rust (lib/rust/crates/gif, behind [SearchText], [GifSearch],
 * [GifSearchKeys], [GifResultCache] and [GifRecents]) up against the Kotlin they replaced,
 * [ReferenceGif], wherever this runs: random queries and texts (emoji and half emoji, odd spaces,
 * apostrophes, İ and ß), typed words for the chips, every key code near FlorisBoard's with every
 * key type, delete-key swipes, failed calls, and random runs of a result cache and of recents,
 * saved and loaded. Self-contained, so a device check can copy it into the app and run it on a
 * phone, whose Java judges spaces, letters and case differently. [dir] holds recents' files.
 */
class GifComparison(private val dir: File) {
    /** How many comparisons failed, and the first few, described. */
    var failures = 0
        private set
    val examples = mutableListOf<String>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    /** How often each outcome worth reaching was reached, so a run that tests nothing shows. */
    val seen = sortedMapOf<String, Int>()

    private fun check(what: String, input: Any?, kotlin: Any?, rust: Any?) {
        checks++
        if (kotlin == rust) return
        failures++
        if (examples.size < MAX_EXAMPLES) examples += "$what(${show(input)}): Kotlin ${show(kotlin)}, Rust ${show(rust)}"
    }

    private fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    /** The constants the panel shares with the Rust. */
    fun constants() {
        check("MAX_LENGTH", null, ReferenceGif.SearchText.MAX_LENGTH, SearchText.MAX_LENGTH)
        check("categories", null, ReferenceGif.categories, GifSearch.categories)
        check("TRENDING", null, ReferenceGif.TRENDING, GifSearch.TRENDING)
        check("RECENTS", null, ReferenceGif.RECENTS, GifSearch.RECENTS)
        check("TTL_MS", null, ReferenceGif.GifResultCache.TTL_MS, GifResultCache.TTL_MS)
        check("cache CAPACITY", null, ReferenceGif.GifResultCache.CAPACITY, GifResultCache.CAPACITY)
        check("recents CAPACITY", null, ReferenceGif.GifRecents.CAPACITY, GifRecents.CAPACITY)
    }

    /** Random queries edited, and texts whose last word is offered. */
    fun text(random: Random, count: Int) = repeat(count) {
        val query = pieces(random, if (random.nextInt(6) == 0) 60 else 8)
        val typed = pieces(random, 3)
        val reference = ReferenceGif.SearchText
        val sum = query + typed
        check("type", listOf(query, typed), reference.type(query, typed), SearchText.type(query, typed))
        see("cut at the limit", sum.length > SearchText.MAX_LENGTH)
        check("typeWord", listOf(query, typed), reference.typeWord(query, typed), SearchText.typeWord(query, typed))
        val one = SearchText.deleteLast(query)
        check("deleteLast", query, reference.deleteLast(query), one)
        see("an emoji deleted whole", query.length - one.length == 2)
        see("half an emoji deleted", query.lastOrNull()?.isSurrogate() == true && query.length - one.length == 1)
        val times = COUNTS.random(random)
        check("deleteLast", listOf(query, times), reference.deleteLast(query, times), SearchText.deleteLast(query, times))
        val words = SearchText.deleteWords(query, times)
        check("deleteWords", listOf(query, times), reference.deleteWords(query, times), words)
        see("words deleted", words.isNotEmpty() && words.length < query.length)
        val before = if (random.nextInt(20) == 0) null else query
        val word = SearchText.lastWord(before)
        check("lastWord", before, reference.lastWord(before), word)
        see(if (word == null) "no word offered" else "a word offered")
        see("a word with an apostrophe", word?.contains('\'') == true)
    }

    /** Random words just typed, and the chips offered after them. */
    fun chips(random: Random, count: Int) = repeat(count) {
        val typed = when (random.nextInt(4)) {
            0 -> null
            1 -> recase(random, (GifSearch.categories + GifSearch.RECENTS).random(random))
            else -> pieces(random, 3)
        }
        val hasRecents = random.nextBoolean()
        val chips = GifSearch.chips(typed, hasRecents)
        check("chips", listOf(typed, hasRecents), ReferenceGif.chips(typed, hasRecents), chips)
        val moved = typed != null && chips.size == GifSearch.categories.size + (if (hasRecents) 1 else 0)
        see(if (typed == null) "no word" else if (moved) "a chip moved first" else "a word put first")
    }

    /** Every key code near FlorisBoard's, with every key type, and delete-key swipes. */
    fun keys(random: Random, swipes: Int) {
        val codes = (-10_000..13_000) + listOf(Int.MIN_VALUE, Int.MAX_VALUE, -1, 0, 65_535, 65_536)
        for (code in codes) {
            for (type in KeyType.entries) {
                val route = GifSearchKeys.route(code, type)
                check("route", listOf(code, type), ReferenceGif.GifSearchKeys.route(code, type).name, route.name)
                see("route ${route.name}")
            }
        }
        repeat(swipes) {
            val action = SwipeAction.entries.random(random)
            val units = if (random.nextInt(10) == 0) EDGES.random(random) else random.nextInt(-40, 40)
            val swipe = GifSearchKeys.swipeDelete(action, units)?.let { it.count to it.words }
            val old = ReferenceGif.GifSearchKeys.swipeDelete(action, units)?.let { it.count to it.words }
            check("swipeDelete", listOf(action, units), old, swipe)
            see(if (swipe == null) "no swipe delete" else if (swipe.first > 0) "a swipe deletes" else "a swipe deletes nothing")
        }
    }

    /** Failed GIPHY calls of every kind, and every status near HTTP's. */
    fun problems() {
        val failures = (-1..700).map { GiphyException(it) } +
            listOf(Int.MIN_VALUE, Int.MAX_VALUE).map { GiphyException(it) } +
            listOf(GiphyFormatException(IllegalStateException()), IOException(), SocketTimeoutException())
        for (e in failures) {
            val problem = GifSearch.problem(e)
            check("problem", e.message, ReferenceGif.problem(e), problem)
            see("problem $problem")
        }
    }

    /** Random runs of a result cache: searches stored and asked for as time passes. */
    fun caches(random: Random, runs: Int, steps: Int) = repeat(runs) {
        var now = TIMES.random(random)
        val ttl = TTLS.random(random)
        val capacity = random.nextInt(5)
        val old = ReferenceGif.GifResultCache({ now }, ttl, capacity)
        val new = GifResultCache({ now }, ttl, capacity)
        val stored = HashMap<String, Long>()
        repeat(steps) { step ->
            now += GAPS.random(random)
            val search = recase(random, SEARCHES.random(random)) + SPACES.random(random)
            val key = search.trim().lowercase(Locale.ROOT)
            if (random.nextInt(3) == 0) {
                val gifs = listOf(gif("$step", "run $it step $step"))
                old.put(search, gifs)
                new.put(search, gifs)
                stored[key] = now
            } else {
                val found = new.get(search)
                check("cache get", listOf(search, now, ttl, capacity), old.get(search), found)
                val at = stored.remove(key)
                if (found != null) stored[key] = at ?: now
                see(
                    when {
                        found != null -> "found"
                        at == null -> "never stored"
                        now - at >= ttl -> "expired"
                        else -> "dropped when full"
                    },
                )
            }
            // A search dropped from the keys must take its GIFs with it, not leave them held.
            check("cache size", listOf(search, now, ttl, capacity), old.size, new.size)
        }
    }

    /** Random runs of recents: GIFs sent, some again, then saved ones loaded with more sent meanwhile. */
    fun recents(random: Random, runs: Int, steps: Int) = repeat(runs) { run ->
        val capacity = if (random.nextInt(4) == 0) GifRecents.CAPACITY else random.nextInt(6)
        val old = ReferenceGif.GifRecents(file = null, capacity)
        val new = GifRecents(file = null, capacity)
        repeat(steps) { step ->
            val gif = gif(IDS.random(random), "run $run step $step")
            val again = new.all().any { it.id == gif.id }
            old.add(gif)
            new.add(gif)
            check("add", listOf(gif.id, capacity), old.all(), new.all())
            see("sent again", again)
            see("the oldest dropped", !again && new.all().size == capacity && capacity > 0)
        }
        val file = File(dir, "gif-recents-$run.json")
        val saved = List(random.nextInt(8)) { gif(IDS.random(random), "saved $it") }
        file.writeText(json.encodeToString(saved))
        val oldLoaded = ReferenceGif.GifRecents(file, capacity)
        val newLoaded = GifRecents(file, capacity)
        repeat(random.nextInt(4)) {
            val gif = gif(IDS.random(random), "meanwhile $it")
            oldLoaded.add(gif)
            newLoaded.add(gif)
        }
        val meanwhile = newLoaded.all().isNotEmpty()
        oldLoaded.load()
        newLoaded.load()
        check("load", listOf(saved.map { it.id }, capacity), oldLoaded.all(), newLoaded.all())
        see("loaded with some sent meanwhile", meanwhile && saved.isNotEmpty())
        see("a saved duplicate kept", newLoaded.all().map { it.id }.let { it.size != it.toSet().size })
        file.delete()
    }

    private fun gif(id: String, title: String) = Gif(id, title, "p/$id", 1.5f, "s/$id", "u/$id")

    /** Up to [parts] random pieces of text. */
    private fun pieces(random: Random, parts: Int): String = buildString {
        repeat(random.nextInt(parts + 1)) { append(PARTS.random(random)) }
    }

    private fun recase(random: Random, text: String): String = when (random.nextInt(4)) {
        0 -> text.uppercase()
        1 -> text.lowercase()
        else -> text
    }

    private fun show(value: Any?): String = when (value) {
        is String -> "\"" + value.map { if (it.code in 0x20..0x7E) "$it" else "\\u%04X".format(it.code) }.joinToString("") + "\""
        is List<*> -> value.joinToString(", ", "[", "]") { show(it) }
        else -> value.toString()
    }

    private companion object {
        const val MAX_EXAMPLES = 10
        private val json = Json { ignoreUnknownKeys = true }

        val PARTS = listOf(
            "cat", "Congrats", "happy", "dance", "a", "I", "OK", "x", "7", "42", "don't", "'", "''", "’", "İ",
            "ß", "ǅ", "Σ", "café", "é", "😀", "\uD800", "\uDC00", "-", "!", ".", ",", "?", ";", ":", ")", "…",
            " ", " ", " ", "  ", "\t", "\n", " ", "\u0085", " ", "　", "\u001C", "​", "ـ", "ـا", "ा", "中文",
            "\u00A0", "\u2007", "\u202F", "\u2028",
            "x".repeat(29), "y".repeat(30), "z".repeat(31),
        )
        val COUNTS = listOf(-2, -1, 0, 1, 1, 2, 3, 5, 9, 60, Int.MAX_VALUE / 1_000_000, Int.MIN_VALUE)
        val EDGES = listOf(Int.MIN_VALUE, Int.MAX_VALUE, Int.MIN_VALUE + 1, -1_000_000, 1_000_000)
        val SEARCHES = listOf("cats", "Thank you", "Trending", "haha", "İstanbul", "STRASSE", "ΣΟΦΊΑ", "")
        val SPACES = listOf("", "", "", " ", "\t", " ", "\n ")
        val IDS = listOf("a", "b", "c", "d", "e", "f", "g", "abc", "A")
        val TIMES = listOf(0L, 1_000L, Long.MAX_VALUE - 5_000, Long.MIN_VALUE)
        val TTLS = listOf(0L, 1L, 1_000L, 600_000L, Long.MAX_VALUE, -1L)
        val GAPS = listOf(0L, 0L, 1L, 10L, 500L, 999L, 1_000L, 1_001L, 600_000L, -1L)
    }
}
