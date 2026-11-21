package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.*
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import java.io.File
import java.io.IOException

/**
 * The GIFs the user sent, newest first, so they can be sent again without a GIPHY call. Their
 * order is decided in Rust ([GifRules.recentsAdd]).
 * Stored in [file] (files/gif-recents.json, never backed up), never logged, and not added to
 * in incognito. Loaded on a background thread and used on the main one, hence the locking.
 */
internal class GifRecents(private val file: File?, private val capacity: Int = CAPACITY) {
    private val gifs = ArrayList<Gif>()

    @Synchronized
    fun all(): List<Gif> = gifs.toList()

    fun isStoredIn(other: File) = file == other

    /** [gif] was sent: it goes first, once. */
    @Synchronized
    fun add(gif: Gif) {
        val before = gifs.toList()
        val order = GifRules.recentsAdd(before.map { it.id }, gif.id, capacity)
        gifs.clear()
        order.mapTo(gifs) { if (GifRules.isSent(it)) gif else before[it] }
    }

    /** Reads [file], keeping anything sent before it finished. A missing or damaged file is no GIFs. */
    fun load() {
        val saved = try {
            file?.takeIf { it.exists() }?.readText()?.let { json.decodeFromString<List<Gif>>(it) }
        } catch (_: IOException) {
            null
        } catch (_: SerializationException) {
            null
        } ?: return
        synchronized(this) {
            val sentMeanwhile = gifs.toList()
            val order = GifRules.recentsLoad(saved.map { it.id }, sentMeanwhile.map { it.id }, capacity)
            gifs.clear()
            order.mapTo(gifs) { if (GifRules.isSent(it)) sentMeanwhile[GifRules.index(it)] else saved[it] }
        }
    }

    /** Writes [file] whole, through a temporary file so a crash mid-write can't damage it. */
    fun save(): Boolean {
        val target = file ?: return false
        val text = json.encodeToString(all())
        return try {
            val part = File(target.parentFile, "${target.name}.part")
            part.writeText(text)
            part.renameTo(target)
        } catch (_: IOException) {
            false
        }
    }

    companion object {
        val CAPACITY: Int = GifRules.recentsCapacity
        private val json = Json { ignoreUnknownKeys = true }
    }
}
