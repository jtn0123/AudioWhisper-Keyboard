package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GiphyRules
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import java.io.IOException
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL

/** One GIF: a small animated preview for the panel, and the GIF that gets sent. Saved in [GifRecents]. */
@Serializable
data class Gif(
    val id: String,
    val title: String,
    val previewUrl: String,
    /** Width over height of the preview. */
    val aspectRatio: Float,
    val sendUrl: String,
    /** The GIF's page on giphy.com, typed instead when an app can't take GIFs. */
    val pageUrl: String,
)

/** GIPHY refused or failed a request; [status] is the HTTP status. */
class GiphyException(val status: Int) : IOException("GIPHY answered HTTP $status")

/** GIPHY answered, but not with the JSON we expect. An [IOException] so callers handle it like any failed call. */
class GiphyFormatException(cause: Throwable) : IOException("GIPHY sent an unreadable response", cause)

/**
 * GIPHY's GIF API (https://developers.giphy.com/docs/api/endpoint). A beta key allows 100 calls an
 * hour, so the panel calls it only when the user asks for something: trending once, then a call
 * per search or category. Previews and sent GIFs come from GIPHY's media servers, which don't count.
 *
 * The URLs and the reading of GIPHY's answers are in Rust ([GiphyRules]); this makes the call.
 */
internal class Giphy(private val apiKey: String) : GifDataSource {
    override suspend fun trending(): List<Gif> = fetch(trendingUrl())

    /** Searches for [query], cut to GIPHY's 50-character limit. */
    override suspend fun search(query: String): List<Gif> = fetch(searchUrl(query))

    override suspend fun preview(url: String): ByteArray = GifImages.previewBytes(url)

    internal fun trendingUrl(): String = GiphyRules.trendingUrl(apiKey)

    internal fun searchUrl(query: String): String = GiphyRules.searchUrl(apiKey, query)

    /** The URL that asks [endpoint] with [params], then the key, a limit of 30, rating pg-13 and English. */
    internal fun url(endpoint: String, params: Map<String, String>): String = GiphyRules.url(apiKey, endpoint, params)

    private suspend fun fetch(url: String): List<Gif> = withContext(Dispatchers.IO) {
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.connectTimeout = TIMEOUT_MS
        connection.readTimeout = TIMEOUT_MS
        try {
            val status = connection.responseCode
            if (status != HttpURLConnection.HTTP_OK) throw GiphyException(status)
            parse(connection.inputStream.use(::readReply))
        } finally {
            connection.disconnect()
        }
    }

    companion object {
        private const val TIMEOUT_MS = 10_000

        /** A page of 30 results is about 100 KB; anything past this isn't a reply we can use. */
        internal const val MAX_REPLY_BYTES = 2 * 1024 * 1024

        /** The reply as text, refusing one longer than [MAX_REPLY_BYTES] rather than holding it all. */
        internal fun readReply(input: InputStream): String = try {
            readLimited(input, MAX_REPLY_BYTES).decodeToString()
        } catch (e: TooLargeException) {
            throw GiphyFormatException(e)
        }

        /**
         * The GIFs in a search or trending response, skipping any without an id or the renditions we
         * need, and repeats of an id. A body that isn't the expected JSON throws [GiphyFormatException].
         * Read as kotlinx.serialization (`ignoreUnknownKeys`) and `toFloatOrNull` read it.
         */
        fun parse(body: String): List<Gif> = try {
            GiphyRules.parse(body, ::Gif)
        } catch (e: IllegalArgumentException) {
            throw GiphyFormatException(e)
        }
    }
}
