package com.jtn0123.audiowhisper.gif

import java.io.File

/**
 * Where GIFs come from: GIPHY, or local fixtures in a debug build or a test, so the panel can be
 * driven without an API key. Requests are made as `GifSessionPlan` says, each with a
 * `GifRequest` row; called on an IO thread.
 */
internal interface GifDataSource {
    /** The API key, which only goes into the URLs Rust builds. */
    val key: String

    /** [url]'s status, and its body if that is the status [request] asks for; [TooLargeException] past its limit. */
    suspend fun get(url: String, request: IntArray): GifAnswer

    /** Writes [url]'s body to [file] if it answers the status [request] asks for, and says whether it did. */
    suspend fun save(url: String, file: File, request: IntArray): Boolean

    /** The animated preview at [url], held in memory by [GifImages] while the panel is open. */
    suspend fun preview(url: String): ByteArray = GifImages.previewBytes(url)
}
