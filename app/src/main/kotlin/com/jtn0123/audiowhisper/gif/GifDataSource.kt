package com.jtn0123.audiowhisper.gif

/** Network edge, replaceable by generated local media in tests without an API key. */
internal interface GifDataSource {
    suspend fun trending(): List<Gif>
    suspend fun search(query: String): List<Gif>
    suspend fun preview(url: String): ByteArray
}
