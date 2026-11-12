package com.jtn0123.audiowhisper.gif

import android.graphics.ImageDecoder
import android.graphics.drawable.Drawable
import android.util.LruCache
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL
import java.nio.ByteBuffer

/**
 * Downloads GIF media. Previews are kept in memory while the panel is open, so scrolling back
 * doesn't fetch them again, and dropped when it closes: nothing is written to disk except the
 * one GIF being sent, which the receiving app reads from [GifSender]'s cache folder.
 */
object GifImages {
    private const val PREVIEW_CACHE_BYTES = 24 * 1024 * 1024
    private const val MAX_PREVIEW_BYTES = 4 * 1024 * 1024
    private const val TIMEOUT_MS = 15_000

    private val previews = object : LruCache<String, ByteArray>(PREVIEW_CACHE_BYTES) {
        override fun sizeOf(key: String, value: ByteArray) = value.size
    }

    /** The animated preview at [url], or null if it could not be fetched or decoded. */
    suspend fun preview(url: String): Drawable? = withContext(Dispatchers.IO) {
        try {
            val bytes = previews.get(url) ?: download(url, MAX_PREVIEW_BYTES).also { previews.put(url, it) }
            decode(bytes)
        } catch (e: IOException) {
            null
        }
    }

    fun forgetPreviews() = previews.evictAll()

    /** Saves [url] to [file], refusing anything larger than [maxBytes]. */
    suspend fun save(url: String, file: File, maxBytes: Int) = withContext(Dispatchers.IO) {
        open(url) { input -> file.outputStream().use { copyLimited(input, it, maxBytes) } }
    }

    private fun download(url: String, maxBytes: Int): ByteArray = open(url) { readLimited(it, maxBytes) }

    private fun <T> open(url: String, read: (InputStream) -> T): T {
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.connectTimeout = TIMEOUT_MS
        connection.readTimeout = TIMEOUT_MS
        try {
            val status = connection.responseCode
            if (status != HttpURLConnection.HTTP_OK) throw GiphyException(status)
            return connection.inputStream.use(read)
        } finally {
            connection.disconnect()
        }
    }

    /** Decodes [bytes] to a drawable, animated when the GIF is. */
    private fun decode(bytes: ByteArray): Drawable =
        ImageDecoder.decodeDrawable(ImageDecoder.createSource(ByteBuffer.wrap(bytes)))
}
