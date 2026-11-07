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
        prune(dir)
        val file = File(dir, "${gif.id.filter(Char::isLetterOrDigit).ifEmpty { "gif" }}.gif")
        if (!file.exists()) {
            try {
                GifImages.save(gif.sendUrl, file, MAX_BYTES)
            } catch (e: IOException) {
                file.delete()
                throw e
            }
        }
        file
    }

    private fun prune(dir: File) {
        val cutoff = System.currentTimeMillis() - KEEP_MS
        dir.listFiles()?.filter { it.lastModified() < cutoff }?.forEach { it.delete() }
    }
}
