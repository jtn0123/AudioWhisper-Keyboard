package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRequest
import com.jtn0123.audiowhisper.rust.GiphyRules
import kotlinx.serialization.Serializable
import java.io.File
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

/** What a request answered: its HTTP [status], and its [body] when that was the status asked for. */
internal class GifAnswer(val status: Int, val body: ByteArray?)

/**
 * GIPHY's GIF API (https://developers.giphy.com/docs/api/endpoint) and media servers. A beta key
 * allows 100 calls an hour, so the panel calls the API only when the user asks for something:
 * trending once, then a call per search or category. Previews and sent GIFs come from the media
 * servers, which don't count.
 *
 * The URLs, the limits of each request and the reading of the answers are decided in Rust
 * (`GifSessionPlan`, crates/gif); this makes the calls, as each [GifRequest] row says.
 */
internal class Giphy(override val key: String) : GifDataSource {
    override suspend fun get(url: String, request: IntArray): GifAnswer =
        open(url, request) { status, input -> GifAnswer(status, input?.let { readLimited(it, request[GifRequest.MAX_BYTES]) }) }

    override suspend fun save(url: String, file: File, request: IntArray): Boolean = open(url, request) { _, input ->
        input?.let { body -> file.outputStream().use { copyLimited(body, it, request[GifRequest.MAX_BYTES]) } } != null
    }

    /** Opens [url] with [request]'s timeout, reading the body only for its status. */
    private fun <T> open(url: String, request: IntArray, read: (Int, InputStream?) -> T): T {
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.connectTimeout = request[GifRequest.TIMEOUT_MS]
        connection.readTimeout = request[GifRequest.TIMEOUT_MS]
        try {
            val status = connection.responseCode
            if (status != request[GifRequest.OK_STATUS]) return read(status, null)
            return connection.inputStream.use { read(status, it) }
        } finally {
            connection.disconnect()
        }
    }
}
