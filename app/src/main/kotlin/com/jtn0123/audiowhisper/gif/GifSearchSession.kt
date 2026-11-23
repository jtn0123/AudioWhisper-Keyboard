package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifEffect
import com.jtn0123.audiowhisper.rust.GifOp
import com.jtn0123.audiowhisper.rust.GifPlan
import com.jtn0123.audiowhisper.rust.GifReply
import com.jtn0123.audiowhisper.rust.GifRequest
import com.jtn0123.audiowhisper.rust.GifResults
import com.jtn0123.audiowhisper.rust.GifRow
import com.jtn0123.audiowhisper.rust.GifSessionPlan
import com.jtn0123.audiowhisper.rust.WireFlag
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException
import kotlin.coroutines.CoroutineContext

/**
 * What the GIF panel shows, and the search field the keyboard types into while it is open. Every
 * step is decided in Rust (`GifSessionPlan`, crates/gif's session); this runs the effects it
 * answers with: fetches through [source], recents' file on [io], and the flows the panel draws.
 * [scope] is the main thread's: call from it.
 */
open class GifSearchSession internal constructor(
    private val scope: CoroutineScope,
    private val io: CoroutineContext,
    private val clock: () -> Long,
    private val log: (String) -> Unit,
    private val forgetPreviews: () -> Unit,
    source: GifDataSource?,
) {
    private val plan = GifSessionPlan()
    private val fetches = HashMap<Int, Job>()

    internal var source: GifDataSource? = source
        set(value) {
            field = value
            run(GifOp.SOURCE, WireFlag.encode(value != null), value?.key.orEmpty())
        }

    /**
     * GIFs can be fetched: a GIPHY key was built in. Without one there is no GIF tab: nothing on
     * the phone could supply the key, so the tab could only ever say it was missing. See
     * `giphy.apiKey` in app/build.gradle.kts.
     */
    val available: Boolean get() = source != null

    private val mutableShowingGifs = MutableStateFlow(false)
    private val mutableTyping = MutableStateFlow(false)
    private val mutableQuery = MutableStateFlow("")
    private val mutableShown = MutableStateFlow("")
    private val mutableResults = MutableStateFlow<GifSearch.Results>(GifSearch.Results.Loading)
    private val mutableChips = MutableStateFlow(emptyList<String>())

    /** The media panel shows GIFs rather than emoji. */
    val showingGifs: StateFlow<Boolean> = mutableShowingGifs.asStateFlow()

    /** The search field is open over the keyboard and typing goes into it. */
    val typing: StateFlow<Boolean> = mutableTyping.asStateFlow()

    /** What is in the search field. */
    val query: StateFlow<String> = mutableQuery.asStateFlow()

    /** The search (or Trending, or Recents) whose GIFs are shown. */
    val shown: StateFlow<String> = mutableShown.asStateFlow()

    val results: StateFlow<GifSearch.Results> = mutableResults.asStateFlow()

    /** The quick searches to offer: the word just typed, Recents once a GIF was sent, then the categories. */
    val chips: StateFlow<List<String>> = mutableChips.asStateFlow()

    init {
        run(GifOp.SOURCE, WireFlag.encode(source != null), source?.key.orEmpty())
        run(GifOp.SNAPSHOT)
    }

    /** The keyboard started: read the GIFs sent before from [filesDir], once. */
    fun attach(filesDir: File) = run(GifOp.ATTACH, File(filesDir, RECENTS_FILE).path)

    /** The panel shows GIFs, or emoji. The first time, it shows Recents if there are any, else trending. */
    fun showGifs(show: Boolean) = run(GifOp.SHOW, WireFlag.encode(show))

    /**
     * The panel opened after [textBefore]: offer its last word as a search ("congrats"). Not in a
     * [private] field (incognito, or a password), where tapping the chip would send it to GIPHY.
     */
    fun offerTypedWord(textBefore: CharSequence?, private: Boolean) =
        run(GifOp.OFFER, WireFlag.encode(textBefore != null), textBefore?.toString().orEmpty(), WireFlag.encode(private))

    /** [gif] was sent. In incognito it isn't remembered. */
    fun sent(gif: Gif, incognito: Boolean) = run(GifOp.SENT, WireFlag.encode(incognito), *row(gif))

    fun startTyping() = run(GifOp.START_TYPING)

    fun stopTyping() = run(GifOp.STOP_TYPING)

    fun type(text: String) = run(GifOp.TYPE, text)

    /** A glide-typed word, spaced from the one before. */
    fun typeWord(word: String) = run(GifOp.TYPE_WORD, word)

    fun deleteLast() = run(GifOp.DELETE_LAST)

    fun deleteWord() = run(GifOp.DELETE_WORD)

    /**
     * A delete-key swipe has reached [swipe] back from where it began ([starts]: it just
     * began). The query shrinks as it goes and grows back if the swipe returns.
     */
    fun deleteSwiped(swipe: GifSearchKeys.SwipeDelete, starts: Boolean) =
        run(GifOp.SWIPE, WireFlag.encode(swipe.words), swipe.count.toString(), WireFlag.encode(starts))

    /** Searches for what was typed (trending if nothing was) and closes the field. */
    fun submit() = run(GifOp.SUBMIT)

    /** Shows [search]'s GIFs, fetching them only if they aren't showing, loading or kept. */
    fun load(search: String) = run(GifOp.LOAD, search)

    /** The keyboard went away: next time it opens on the keyboard, not a half-typed search. */
    fun reset() = run(GifOp.RESET)

    private fun run(op: GifOp, vararg fields: String) {
        val answer = plan.call(op, clock(), *fields)
        mutableShowingGifs.value = WireFlag.decode(answer[GifPlan.SHOWING])
        mutableTyping.value = WireFlag.decode(answer[GifPlan.TYPING])
        mutableQuery.value = answer[GifPlan.QUERY]
        mutableShown.value = answer[GifPlan.SHOWN]
        var at = GifPlan.SIZE
        while (at < answer.size) at = effect(op, answer, at)
    }

    /** Runs the effect at [at]: where the next one starts. */
    private fun effect(op: GifOp, answer: Array<String>, at: Int): Int = when (answer[at]) {
        GifEffect.RESULTS -> results(op, answer, at + 1)
        GifEffect.CHIPS -> {
            val count = answer[at + 1].toInt()
            mutableChips.value = answer.copyOfRange(at + 2, at + 2 + count).toList()
            at + 2 + count
        }
        GifEffect.FETCH -> {
            fetch(answer[at + 1].toInt(), answer[at + 2], IntArray(GifRequest.SIZE) { answer[at + 3 + it].toInt() })
            at + 3 + GifRequest.SIZE
        }
        GifEffect.CANCEL -> {
            fetches.remove(answer[at + 1].toInt())?.cancel()
            at + 2
        }
        GifEffect.READ -> {
            read(answer[at + 1].toInt(), File(answer[at + 2]))
            at + 3
        }
        GifEffect.SAVE -> {
            save(answer[at + 1].toInt())
            at + 2
        }
        GifEffect.FORGET_PREVIEWS -> {
            forgetPreviews()
            at + 1
        }
        else -> error("No GIF effect ${answer[at]}")
    }

    private fun results(op: GifOp, answer: Array<String>, at: Int): Int {
        val count = answer[at + 2].toInt()
        val rows = at + 3
        mutableResults.value = when (answer[at].toInt()) {
            GifResults.LOADING -> GifSearch.Results.Loading
            GifResults.LOADED -> GifSearch.Results.Loaded(gifs(answer, rows, count))
            else -> GifSearch.Results.Failed(answer[at + 1]).also { if (op == GifOp.FETCHED) log("Could not load GIFs") }
        }
        return rows + count * GifRow.SIZE
    }

    private fun fetch(generation: Int, url: String, request: IntArray) {
        val source = source ?: return
        fetches[generation] = scope.launch {
            val (kind, status, body) = withContext(io) { answer(source, url, request) }
            fetches.remove(generation)
            run(GifOp.FETCHED, generation.toString(), kind, status.toString(), body)
        }
    }

    /** What fetching [url] came to, as a `GifReply`, the status, and the body as text. */
    private suspend fun answer(source: GifDataSource, url: String, request: IntArray): Triple<String, Int, String> = try {
        val answer = source.get(url, request)
        val body = answer.body
        if (body == null) Triple(GifReply.STATUS, answer.status, "") else Triple(GifReply.OK, answer.status, body.decodeToString())
    } catch (_: TooLargeException) {
        Triple(GifReply.TOO_LARGE, 0, "")
    } catch (_: IOException) {
        Triple(GifReply.UNREACHABLE, 0, "")
    }

    private fun read(generation: Int, file: File) {
        scope.launch {
            withContext(io) {
                val saved = GifRecents.read(file)
                plan.read(generation, saved != null, saved.orEmpty().flatMap { row(it).asList() }.toTypedArray())
            }
            run(GifOp.MERGED)
        }
    }

    private fun save(generation: Int) {
        scope.launch(io) {
            val rows = plan.rows(generation)
            if (rows.isEmpty()) return@launch
            GifRecents.write(File(rows[0]), gifs(rows, 1, (rows.size - 1) / GifRow.SIZE))
        }
    }

    internal companion object {
        const val RECENTS_FILE = "gif-recents.json"

        /** [gif] as it crosses: its ratio as bits, so it comes back exactly. */
        fun row(gif: Gif): Array<String> =
            arrayOf(gif.id, gif.title, gif.previewUrl, gif.aspectRatio.toRawBits().toString(), gif.sendUrl, gif.pageUrl)

        fun gifs(rows: Array<String>, at: Int, count: Int): List<Gif> = List(count) {
            val row = at + it * GifRow.SIZE
            Gif(
                rows[row + GifRow.ID], rows[row + GifRow.TITLE], rows[row + GifRow.PREVIEW_URL],
                Float.fromBits(rows[row + GifRow.RATIO].toInt()), rows[row + GifRow.SEND_URL], rows[row + GifRow.PAGE_URL],
            )
        }
    }
}
