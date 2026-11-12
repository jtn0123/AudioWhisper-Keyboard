package com.jtn0123.audiowhisper.gif

import android.content.Context
import android.util.Log
import androidx.core.content.FileProvider
import androidx.core.net.toUri
import dev.patrickgold.florisboard.ime.editor.EditorInstance
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException

/** Puts a GIF into the app being typed in. Main thread; downloads on IO. */
object GifSender {
    private const val TAG = "AWGif"
    private const val DIR = "gifs" // shared through res/xml/file_paths.xml
    private const val MAX_BYTES = 8 * 1024 * 1024

    /** Sent GIFs are kept this long, so the receiving app can still read one it hasn't sent yet. */
    private const val KEEP_MS = 24 * 60 * 60 * 1000L

    private const val PART = ".part"

    /** Longer than a download can take: two 15-second timeouts. */
    private const val PART_STALE_MS = 60_000L

    enum class Outcome { SENT, LINKED, FAILED }

    /** Sends [gif] as an image if the app takes GIFs, else types its link. */
    suspend fun send(context: Context, editor: EditorInstance, gif: Gif): Outcome {
        if (!editor.acceptsGifs()) return typeLink(editor, gif)
        val file = try {
            download(context, gif)
        } catch (e: IOException) {
            Log.w(TAG, "Could not download the GIF", e)
            return Outcome.FAILED
        }
        val uri = FileProvider.getUriForFile(context, "${context.packageName}.provider.file", file)
        return if (editor.commitGif(uri, gif.title, gif.pageUrl.toUri())) Outcome.SENT else typeLink(editor, gif)
    }

    private fun typeLink(editor: EditorInstance, gif: Gif) =
        if (gif.pageUrl.isNotEmpty() && editor.commitText(gif.pageUrl)) Outcome.LINKED else Outcome.FAILED

    private suspend fun download(context: Context, gif: Gif): File = withContext(Dispatchers.IO) {
        val dir = File(context.cacheDir, DIR).apply { mkdirs() }
        val now = System.currentTimeMillis()
        prune(dir, now)
        val file = File(dir, "${gif.id.filter(Char::isLetterOrDigit).ifEmpty { "gif" }}.gif")
        fetchInto(file, now) { GifImages.save(gif.sendUrl, it, MAX_BYTES) }
        file
    }

    /**
     * Makes [file] hold the whole GIF. A new one is written to a `.part` file by [save] and renamed
     * only once complete, so a download cut off by a crash is never mistaken for a cached GIF. A
     * reused file is touched to [now], so [prune] counts its 24 hours from this send.
     */
    internal suspend fun fetchInto(file: File, now: Long, save: suspend (File) -> Unit) {
        if (file.exists()) {
            file.setLastModified(now)
            return
        }
        val part = File(file.parentFile, file.name + PART)
        try {
            save(part)
            if (!part.renameTo(file)) throw IOException("Could not keep the downloaded GIF")
        } catch (e: IOException) {
            part.delete()
            throw e
        }
    }

    /**
     * Deletes GIFs sent more than a day before [now], and `.part` files left by a download that
     * stopped: one older than [PART_STALE_MS] can't still be in progress.
     */
    internal fun prune(dir: File, now: Long) {
        dir.listFiles()?.filter {
            val age = now - it.lastModified()
            if (it.name.endsWith(PART)) age > PART_STALE_MS else age > KEEP_MS
        }?.forEach { it.delete() }
    }
}
