package com.jtn0123.audiowhisper.gif

import android.content.Context
import android.util.Log
import androidx.core.content.FileProvider
import androidx.core.net.toUri
import com.jtn0123.audiowhisper.rust.GifMedia
import com.jtn0123.audiowhisper.rust.GifSendAction
import com.jtn0123.audiowhisper.rust.GifSendStep
import com.jtn0123.audiowhisper.rust.GifSessionPlan
import dev.patrickgold.florisboard.ime.editor.EditorInstance
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException
import kotlin.coroutines.CoroutineContext

/** The app a GIF goes into. Main thread. */
internal interface GifTarget {
    fun acceptsGifs(): Boolean

    /** Offers [file] to the app as [gif]: whether it took it. */
    fun commitGif(file: File, gif: Gif): Boolean

    fun commitText(text: String): Boolean
}

/**
 * Sends one GIF from [dir], doing each step Rust's `gif::send` decides (`GifSessionPlan.sendNext`):
 * as an image if the app takes GIFs, downloaded once into a part file and renamed only when whole,
 * else its link. Files are touched on [io]; nothing logged names the GIF.
 */
internal class GifSend(
    private val dir: File,
    private val io: CoroutineContext,
    private val clock: () -> Long,
    private val source: GifDataSource?,
    private val target: GifTarget,
    private val log: (String) -> Unit,
) {
    suspend fun send(gif: Gif): GifSender.Outcome {
        val (name, partName) = GifSessionPlan.sendFiles(gif.id)
        val files = Files(File(dir, name), File(dir, partName))
        var answer = GifSendStep.START to target.acceptsGifs()
        while (true) {
            var next: Pair<GifSendStep, Boolean>? = null
            for (action in GifSessionPlan.sendNext(answer.first, answer.second, gif.pageUrl.isNotEmpty())) {
                OUTCOMES[action]?.let { return it }
                next = act(action, gif, files) ?: next
            }
            answer = checkNotNull(next) { "Sending a GIF stopped" }
        }
    }

    private class Files(val file: File, val part: File) {
        var now = 0L
    }

    /** Does [action]: the step it answers with, if any. */
    private suspend fun act(action: String, gif: Gif, files: Files): Pair<GifSendStep, Boolean>? = when (action) {
        GifSendAction.TYPE_LINK -> GifSendStep.TYPED to target.commitText(gif.pageUrl)
        GifSendAction.PRUNE -> GifSendStep.CHECKED to withContext(io) { prune(files) }
        GifSendAction.TOUCH -> withContext(io) { files.file.setLastModified(files.now) }.let { null }
        GifSendAction.DOWNLOAD -> GifSendStep.SAVED to withContext(io) { download(gif, files.part) }
        GifSendAction.RENAME -> GifSendStep.RENAMED to withContext(io) { files.part.renameTo(files.file) }
        GifSendAction.DELETE_PART -> withContext(io) { files.part.delete() }.let { null }
        GifSendAction.COMMIT -> GifSendStep.COMMITTED to target.commitGif(files.file, gif)
        else -> error("No GIF send action $action")
    }

    /** Makes the folder and deletes its stale files: whether the GIF's file is there. */
    private fun prune(files: Files): Boolean {
        dir.mkdirs()
        files.now = clock()
        val listed = dir.listFiles().orEmpty()
        val modified = LongArray(listed.size) { listed[it].lastModified() }
        for (index in GifSessionPlan.sendStale(Array(listed.size) { listed[it].name }, modified, files.now)) listed[index].delete()
        return files.file.exists()
    }

    private suspend fun download(gif: Gif, part: File): Boolean = try {
        source?.save(gif.sendUrl, part, REQUEST).also { if (it == false) log("Could not download the GIF: refused") } == true
    } catch (e: IOException) {
        log("Could not download the GIF: ${e.javaClass.simpleName}")
        false
    }

    private companion object {
        val REQUEST: IntArray = GifSessionPlan.request(GifMedia.SEND)
        val OUTCOMES = mapOf(
            GifSendAction.SENT to GifSender.Outcome.SENT,
            GifSendAction.LINKED to GifSender.Outcome.LINKED,
            GifSendAction.FAILED to GifSender.Outcome.FAILED,
        )
    }
}

/** Puts a GIF into the app being typed in. Main thread; downloads on IO. */
object GifSender {
    private const val TAG = "AWGif"
    private const val DIR = "gifs" // shared through res/xml/file_paths.xml

    enum class Outcome { SENT, LINKED, FAILED }

    /** Sends [gif] as an image if the app takes GIFs, else types its link. */
    suspend fun send(context: Context, editor: EditorInstance, gif: Gif): Outcome {
        val target = object : GifTarget {
            override fun acceptsGifs() = editor.acceptsGifs()

            override fun commitGif(file: File, gif: Gif): Boolean {
                val uri = FileProvider.getUriForFile(context, "${context.packageName}.provider.file", file)
                return editor.commitGif(uri, gif.title, gif.pageUrl.toUri())
            }

            override fun commitText(text: String) = editor.commitText(text)
        }
        val dir = File(context.cacheDir, DIR)
        return GifSend(dir, Dispatchers.IO, System::currentTimeMillis, GifSearch.source, target) { Log.w(TAG, it) }.send(gif)
    }
}
