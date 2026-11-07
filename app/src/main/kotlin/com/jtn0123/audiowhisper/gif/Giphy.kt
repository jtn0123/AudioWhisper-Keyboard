package com.jtn0123.audiowhisper.gif

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

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
 */
class Giphy(private val apiKey: String) {
    suspend fun trending(): List<Gif> = fetch(url("trending", emptyMap()))

    suspend fun search(query: String): List<Gif> = fetch(url("search", mapOf("q" to query.take(MAX_QUERY_LENGTH))))

    internal fun url(endpoint: String, params: Map<String, String>): String {
        val query = (params + mapOf("api_key" to apiKey, "limit" to "$LIMIT", "rating" to RATING, "lang" to "en"))
            .entries.joinToString("&") { (name, value) -> "$name=${URLEncoder.encode(value, "UTF-8")}" }
        return "$BASE_URL/$endpoint?$query"
    }

    private suspend fun fetch(url: String): List<Gif> = withContext(Dispatchers.IO) {
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.connectTimeout = TIMEOUT_MS
        connection.readTimeout = TIMEOUT_MS
        try {
            val status = connection.responseCode
            if (status != HttpURLConnection.HTTP_OK) throw GiphyException(status)
            parse(connection.inputStream.bufferedReader().use { it.readText() })
        } finally {
            connection.disconnect()
        }
    }

    companion object {
        private const val BASE_URL = "https://api.giphy.com/v1/gifs"
        private const val LIMIT = 30
        private const val RATING = "pg-13"
        private const val MAX_QUERY_LENGTH = 50
        private const val TIMEOUT_MS = 10_000

        private val json = Json { ignoreUnknownKeys = true }

        /**
         * The GIFs in a search or trending response, skipping any without an id or the renditions we
         * need. A body that isn't the expected JSON throws [GiphyFormatException].
         */
        fun parse(body: String): List<Gif> {
            val response = try {
                json.decodeFromString<Response>(body)
            } catch (e: IllegalArgumentException) {
                // kotlinx.serialization's SerializationException is an IllegalArgumentException.
                throw GiphyFormatException(e)
            }
            return response.data.mapNotNull { it.toGif() }.distinctBy { it.id }
        }

        private fun Item.toGif(): Gif? {
            if (id.isBlank()) return null
            val preview = images.fixedWidth ?: return null
            val previewUrl = preview.webp ?: preview.url ?: return null
            val width = preview.width?.toFloatOrNull() ?: return null
            val height = preview.height?.toFloatOrNull()?.takeIf { it > 0f } ?: return null
            val sendUrl = images.downsized?.url ?: preview.url ?: return null
            return Gif(id, title, previewUrl, width / height, sendUrl, url)
        }
    }

    @Serializable
    private class Response(val data: List<Item> = emptyList())

    @Serializable
    private class Item(val id: String = "", val title: String = "", val url: String = "", val images: Images = Images())

    @Serializable
    private class Images(
        @SerialName("fixed_width") val fixedWidth: Rendition? = null,
        val downsized: Rendition? = null,
    )

    /** GIPHY sends sizes as strings. */
    @Serializable
    private class Rendition(
        val url: String? = null,
        val webp: String? = null,
        val width: String? = null,
        val height: String? = null,
    )
}
