package com.jtn0123.audiowhisper.gif

import android.graphics.ImageDecoder
import android.graphics.drawable.Drawable
import com.jtn0123.audiowhisper.rust.GifMedia
import com.jtn0123.audiowhisper.rust.GifPreviewKeys
import com.jtn0123.audiowhisper.rust.GifSessionPlan
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.IOException
import java.nio.ByteBuffer
import kotlin.coroutines.CoroutineContext

/**
 * The previews held in memory while the panel is open, so scrolling back doesn't fetch them
 * again, and dropped when it closes. Which are kept is decided in Rust ([GifPreviewKeys]); the
 * bytes stay here. Any thread.
 */
internal class GifPreviews(private val io: CoroutineContext, private val source: () -> GifDataSource?) {
    private val keys = GifPreviewKeys()
    private val previews = HashMap<String, ByteArray>()

    /** The preview at [url], fetched unless it is held. */
    suspend fun bytes(url: String): ByteArray = withContext(io) {
        held(url) ?: download(url).also { keep(url, it) }
    }

    fun forget() = synchronized(previews) {
        keys.clear()
        previews.clear()
    }

    private fun held(url: String): ByteArray? = synchronized(previews) { if (keys.get(url)) previews[url] else null }

    private fun keep(url: String, bytes: ByteArray) = synchronized(previews) {
        previews[url] = bytes
        for (dropped in keys.put(url, bytes.size)) previews.remove(dropped)
    }

    private suspend fun download(url: String): ByteArray {
        val source = source() ?: throw IOException("No GIF source")
        val answer = source.get(url, REQUEST)
        return answer.body ?: throw GiphyException(answer.status)
    }

    private companion object {
        val REQUEST: IntArray = GifSessionPlan.request(GifMedia.PREVIEW)
    }
}

/**
 * Downloads GIF media for the panel. Nothing is written to disk except the one GIF being sent,
 * which the receiving app reads from [GifSender]'s cache folder.
 */
object GifImages {
    private val previews = GifPreviews(Dispatchers.IO) { GifSearch.source }

    /** The animated preview at [url], or null if it could not be fetched or decoded. */
    suspend fun preview(url: String): Drawable? = withContext(Dispatchers.IO) {
        try {
            decode(previews.bytes(url))
        } catch (_: IOException) {
            null
        }
    }

    internal suspend fun previewBytes(url: String): ByteArray = previews.bytes(url)

    fun forgetPreviews() = previews.forget()

    /** Decodes [bytes] to a drawable, animated when the GIF is. */
    private fun decode(bytes: ByteArray): Drawable =
        ImageDecoder.decodeDrawable(ImageDecoder.createSource(ByteBuffer.wrap(bytes)))
}
