package com.jtn0123.audiowhisper.rust

/**
 * The GIFs in GIPHY's answer [body], each made by [gif] from its id, title, preview URL, aspect
 * ratio, send URL and page URL. Throws [IllegalArgumentException] if [body] isn't the JSON
 * expected.
 */
fun <T> GiphyRules.parse(
    body: String,
    gif: (id: String, title: String, previewUrl: String, aspectRatio: Float, sendUrl: String, pageUrl: String) -> T,
): List<T> = GiphyNative.parse(body).asList().chunked(6) { gif(it[0], it[1], it[2], it[3].toFloat(), it[4], it[5]) }
