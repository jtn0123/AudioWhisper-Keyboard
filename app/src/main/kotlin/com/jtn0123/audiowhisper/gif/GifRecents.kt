package com.jtn0123.audiowhisper.gif

import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import java.io.File
import java.io.IOException

/**
 * The GIFs the user sent, newest first, so they can be sent again without a GIPHY call.
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
        gifs.removeAll { it.id == gif.id }
        gifs.add(0, gif)
        while (gifs.size > capacity) gifs.removeAt(gifs.lastIndex)
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
            gifs.clear()
            gifs.addAll(saved.take(capacity))
            sentMeanwhile.asReversed().forEach(::add)
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
        const val CAPACITY = 30
        private val json = Json { ignoreUnknownKeys = true }
    }
}
