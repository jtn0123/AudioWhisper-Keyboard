package com.jtn0123.audiowhisper.rust

/**
 * GIPHY's API apart from the call itself, in Rust (lib/rust/crates/gif/src/giphy.rs): the URLs
 * to ask it, and its answer read into GIFs exactly as the keyboard's kotlinx.serialization classes
 * read it. The keyboard's `Giphy` wraps these and documents them.
 */
object GiphyRules {
    /**
     * The GIFs in GIPHY's answer [body], each made by [gif] from its id, title, preview URL, aspect
     * ratio, send URL and page URL. Throws [IllegalArgumentException] if [body] isn't the JSON
     * expected.
     */
    fun <T> parse(
        body: String,
        gif: (id: String, title: String, previewUrl: String, aspectRatio: Float, sendUrl: String, pageUrl: String) -> T,
    ): List<T> {
        val (texts, ratios) = GiphyNative.parse(body).let { it[0] as Array<*> to it[1] as FloatArray }
        return List(ratios.size) { i ->
            val at = i * TEXTS
            gif(
                texts[at] as String,
                texts[at + 1] as String,
                texts[at + 2] as String,
                ratios[i],
                texts[at + 3] as String,
                texts[at + 4] as String,
            )
        }
    }

    /** The URL that asks [endpoint] with [params], then [apiKey] and GIPHY's settings. */
    fun url(apiKey: String, endpoint: String, params: Map<String, String>): String =
        GiphyNative.url(apiKey, endpoint, params.keys.toTypedArray(), params.values.toTypedArray())

    /** The URL that asks for trending GIFs. */
    fun trendingUrl(apiKey: String): String = GiphyNative.trendingUrl(apiKey)

    /** The URL that searches for [query], cut to GIPHY's longest search. */
    fun searchUrl(apiKey: String, query: String): String = GiphyNative.searchUrl(apiKey, query)

    /** The strings each GIF has in the array [GiphyNative.parse] gives. */
    private const val TEXTS = 5
}
