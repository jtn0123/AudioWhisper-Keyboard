package com.jtn0123.audiowhisper.gif

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException
import java.io.InputStream
import kotlin.coroutines.CoroutineContext

/**
 * The GIF panel's sequencing as main had it at b1cf45f18, before it moved to Rust
 * (crates/gif's session): GifSearch.kt, Giphy.kt's fetch, GifImages.kt and GifSender.kt,
 * copied with only these mechanical changes, so [GifSearchTraceTest] can hold the new code to it:
 * the object is a class whose scope, IO dispatcher, clock and log are passed in; the network is
 * [FakeNet] in place of `HttpURLConnection`; Android's `LruCache` is [ReferenceLruCache]; a GIF
 * is committed through [GifTarget] in place of `FileProvider` and the editor; and the rules
 * main called in Rust are [ReferenceGif] and [ReferenceGiphy], the Kotlin those replaced, which
 * GifRulesMatchTest holds equal to them. Don't change it.
 */
internal class ReferenceGifSearch(
    private val scope: CoroutineScope,
    private val io: CoroutineContext,
    private val clock: () -> Long,
    private val net: FakeNet,
    apiKey: String?,
) {
    private val TRENDING = ReferenceGif.TRENDING
    private val RECENTS = ReferenceGif.RECENTS

    internal var source: Giphy? = apiKey?.let { Giphy(it) }
    val available: Boolean get() = source != null
    private var loading: Job? = null
    private val cache = ReferenceGif.GifResultCache(clock)
    private var recents = ReferenceGif.GifRecents(file = null)
    private var beforeSwipe = ""

    private val mutableShowingGifs = MutableStateFlow(false)
    private val mutableTyping = MutableStateFlow(false)
    private val mutableQuery = MutableStateFlow("")
    private val mutableShown = MutableStateFlow(TRENDING)
    private val mutableResults = MutableStateFlow<GifSearch.Results>(GifSearch.Results.Loading)
    private val mutableChips = MutableStateFlow(ReferenceGif.chips(typedWord = null, hasRecents = false))
    private var typedWord: String? = null

    val showingGifs: StateFlow<Boolean> = mutableShowingGifs.asStateFlow()
    val typing: StateFlow<Boolean> = mutableTyping.asStateFlow()
    val query: StateFlow<String> = mutableQuery.asStateFlow()
    val shown: StateFlow<String> = mutableShown.asStateFlow()
    val results: StateFlow<GifSearch.Results> = mutableResults.asStateFlow()
    val chips: StateFlow<List<String>> = mutableChips.asStateFlow()

    fun attach(filesDir: File) {
        val file = File(filesDir, RECENTS_FILE)
        if (recents.isStoredIn(file)) return
        recents = ReferenceGif.GifRecents(file).also { loaded ->
            scope.launch {
                withContext(io) { loaded.load() }
                updateChips()
            }
        }
    }

    fun showGifs(show: Boolean) {
        mutableShowingGifs.value = show && available
        if (!show || results.value is GifSearch.Results.Loaded) return
        val first = shown.value == TRENDING && recents.all().isNotEmpty()
        load(if (first) RECENTS else shown.value)
    }

    fun offerTypedWord(textBefore: CharSequence?, private: Boolean) {
        typedWord = if (private) null else ReferenceGif.SearchText.lastWord(textBefore)
        updateChips()
    }

    fun sent(gif: Gif, incognito: Boolean) {
        if (incognito) return
        recents.add(gif)
        updateChips()
        val toSave = recents
        scope.launch(io) { toSave.save() }
    }

    private fun updateChips() {
        mutableChips.value = ReferenceGif.chips(typedWord, hasRecents = recents.all().isNotEmpty())
    }

    fun startTyping() {
        mutableQuery.value = if (shown.value == TRENDING) "" else shown.value
        mutableTyping.value = true
    }

    fun stopTyping() {
        mutableTyping.value = false
    }

    fun type(text: String) {
        mutableQuery.value = ReferenceGif.SearchText.type(query.value, text)
    }

    fun typeWord(word: String) {
        mutableQuery.value = ReferenceGif.SearchText.typeWord(query.value, word)
    }

    fun deleteLast() {
        mutableQuery.value = ReferenceGif.SearchText.deleteLast(query.value, 1)
    }

    fun deleteWord() {
        mutableQuery.value = ReferenceGif.SearchText.deleteWords(query.value, 1)
    }

    fun deleteSwiped(swipe: GifSearchKeys.SwipeDelete, starts: Boolean) {
        if (starts) beforeSwipe = query.value
        mutableQuery.value = if (swipe.words) {
            ReferenceGif.SearchText.deleteWords(beforeSwipe, swipe.count)
        } else {
            ReferenceGif.SearchText.deleteLast(beforeSwipe, swipe.count)
        }
    }

    fun submit() {
        stopTyping()
        load(query.value.trim().ifEmpty { TRENDING })
    }

    fun load(search: String) {
        val giphy = source ?: return
        if (search == RECENTS) return showRecents()
        val alreadyShowing = search == shown.value && (results.value is GifSearch.Results.Loaded || loading?.isActive == true)
        if (alreadyShowing) return
        mutableShown.value = search
        loading?.cancel()
        cache.get(search)?.let { gifs ->
            mutableResults.value = GifSearch.Results.Loaded(gifs)
            return
        }
        mutableResults.value = GifSearch.Results.Loading
        loading = scope.launch {
            mutableResults.value = try {
                val gifs = if (search == TRENDING) giphy.trending() else giphy.search(search)
                cache.put(search, gifs)
                GifSearch.Results.Loaded(gifs)
            } catch (e: IOException) {
                GifSearch.Results.Failed(ReferenceGif.problem(e))
            }
        }
    }

    private fun showRecents() {
        loading?.cancel()
        mutableShown.value = RECENTS
        mutableResults.value = GifSearch.Results.Loaded(recents.all())
    }

    fun reset() {
        stopTyping()
        mutableShowingGifs.value = false
        images.forgetPreviews()
    }

    /** Giphy.kt's calls, through [net]. */
    inner class Giphy(private val apiKey: String) {
        suspend fun trending(): List<Gif> = fetch(ReferenceGiphy.trendingUrl(apiKey))

        suspend fun search(query: String): List<Gif> = fetch(ReferenceGiphy.searchUrl(apiKey, query))

        private suspend fun fetch(url: String): List<Gif> = withContext(io) {
            val connection = net.open(url, MAX_REPLY_BYTES)
            val status = connection.responseCode
            if (status != 200) throw GiphyException(status)
            ReferenceGiphy.parse(connection.inputStream.use(::readReply))
        }
    }

    /** GifImages.kt. */
    val images = Images()

    inner class Images {
        private val previews = object : ReferenceLruCache<String, ByteArray>(PREVIEW_CACHE_BYTES) {
            override fun sizeOf(key: String, value: ByteArray) = value.size
        }

        suspend fun previewBytes(url: String): ByteArray = withContext(io) {
            previews.get(url) ?: download(url, MAX_PREVIEW_BYTES).also { previews.put(url, it) }
        }

        fun forgetPreviews() = previews.evictAll()

        suspend fun save(url: String, file: File, maxBytes: Int) = withContext(io) {
            open(url, maxBytes) { input -> file.outputStream().use { copyLimited(input, it, maxBytes) } }
        }

        private suspend fun download(url: String, maxBytes: Int): ByteArray = open(url, maxBytes) { readLimited(it, maxBytes) }

        private suspend fun <T> open(url: String, max: Int, read: (InputStream) -> T): T {
            val connection = net.open(url, max)
            val status = connection.responseCode
            if (status != 200) throw GiphyException(status)
            return connection.inputStream.use(read)
        }
    }

    /** GifSender.kt, sending into [target] from [dir] at [now]. */
    suspend fun send(dir: File, now: Long, target: GifTarget, gif: Gif): GifSender.Outcome {
        if (!target.acceptsGifs()) return typeLink(target, gif)
        val file = try {
            download(dir, now, gif)
        } catch (_: IOException) {
            return GifSender.Outcome.FAILED
        }
        return if (target.commitGif(file, gif)) GifSender.Outcome.SENT else typeLink(target, gif)
    }

    private fun typeLink(target: GifTarget, gif: Gif) =
        if (gif.pageUrl.isNotEmpty() && target.commitText(gif.pageUrl)) GifSender.Outcome.LINKED else GifSender.Outcome.FAILED

    private suspend fun download(cacheDir: File, now: Long, gif: Gif): File = withContext(io) {
        val dir = cacheDir.apply { mkdirs() }
        prune(dir, now)
        val file = File(dir, "${gif.id.filter(Char::isLetterOrDigit).ifEmpty { "gif" }}.gif")
        fetchInto(file, now) { images.save(gif.sendUrl, it, MAX_BYTES) }
        file
    }

    private suspend fun fetchInto(file: File, now: Long, save: suspend (File) -> Unit) {
        if (file.exists()) {
            file.setLastModified(now)
            return
        }
        val part = File(file.parentFile, file.name + PART)
        try {
            save(part)
            if (!part.renameTo(file)) throw IOException("Could not keep the downloaded GIF")
        } catch (e: IOException) {
            part.delete()
            throw e
        }
    }

    private fun prune(dir: File, now: Long) {
        dir.listFiles()?.filter {
            val age = now - it.lastModified()
            if (it.name.endsWith(PART)) age > PART_STALE_MS else age > KEEP_MS
        }?.forEach { it.delete() }
    }

    private companion object {
        const val RECENTS_FILE = "gif-recents.json"
        const val MAX_REPLY_BYTES = 2 * 1024 * 1024
        const val PREVIEW_CACHE_BYTES = 24 * 1024 * 1024
        const val MAX_PREVIEW_BYTES = 4 * 1024 * 1024
        const val MAX_BYTES = 8 * 1024 * 1024
        const val KEEP_MS = 24 * 60 * 60 * 1000L
        const val PART = ".part"
        const val PART_STALE_MS = 60_000L

        fun readReply(input: InputStream): String = try {
            readLimited(input, MAX_REPLY_BYTES).decodeToString()
        } catch (e: TooLargeException) {
            throw GiphyFormatException(e)
        }
    }
}
