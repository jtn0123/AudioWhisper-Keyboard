package com.jtn0123.audiowhisper.gif

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.net.URLEncoder

/**
 * The Kotlin that read GIPHY's answers and built its URLs before they moved to Rust, kept
 * unchanged as [GiphyComparison]'s reference: kotlinx.serialization, `toFloatOrNull` and
 * `URLEncoder`.
 */
object ReferenceGiphy {
    const val MAX_QUERY_LENGTH = 50
    private const val BASE_URL = "https://api.giphy.com/v1/gifs"
    private const val LIMIT = 30
    private const val RATING = "pg-13"

    private val json = Json { ignoreUnknownKeys = true }

    fun trendingUrl(apiKey: String): String = url(apiKey, "trending", emptyMap())

    fun searchUrl(apiKey: String, query: String): String = url(apiKey, "search", mapOf("q" to query.take(MAX_QUERY_LENGTH)))

    fun url(apiKey: String, endpoint: String, params: Map<String, String>): String {
        val query = (params + mapOf("api_key" to apiKey, "limit" to "$LIMIT", "rating" to RATING, "lang" to "en"))
            .entries.joinToString("&") { (name, value) -> "$name=${URLEncoder.encode(value, "UTF-8")}" }
        return "$BASE_URL/$endpoint?$query"
    }

    fun parse(body: String): List<Gif> {
        val response = try {
            json.decodeFromString<Response>(body)
        } catch (e: IllegalArgumentException) {
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

    @Serializable
    private class Response(val data: List<Item> = emptyList())

    @Serializable
    private class Item(val id: String = "", val title: String = "", val url: String = "", val images: Images = Images())

    @Serializable
    private class Images(
        @SerialName("fixed_width") val fixedWidth: Rendition? = null,
        val downsized: Rendition? = null,
    )

    @Serializable
    private class Rendition(
        val url: String? = null,
        val webp: String? = null,
        val width: String? = null,
        val height: String? = null,
    )
}
