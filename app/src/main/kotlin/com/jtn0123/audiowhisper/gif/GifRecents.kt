package com.jtn0123.audiowhisper.gif

import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import java.io.File
import java.io.IOException

/**
 * The file of the GIFs the user sent, newest first (files/gif-recents.json, never backed up),
 * so they can be sent again without a GIPHY call. Their order, and when the file is read and
 * written, are decided in Rust (`GifSessionPlan`); this reads and writes the JSON. Never logged,
 * and not added to in incognito. IO thread.
 */
internal object GifRecents {
    private val json = Json { ignoreUnknownKeys = true }

    /** What [file] holds; a missing or damaged file is null. */
    fun read(file: File): List<Gif>? = try {
        file.takeIf { it.exists() }?.readText()?.let { json.decodeFromString<List<Gif>>(it) }
    } catch (_: IOException) {
        null
    } catch (_: SerializationException) {
        null
    }

    /** Writes [gifs] to [file] whole, through a temporary file so a crash mid-write can't damage it. */
    fun write(file: File, gifs: List<Gif>): Boolean {
        val text = json.encodeToString(gifs)
        return try {
            val part = File(file.parentFile, "${file.name}.part")
            part.writeText(text)
            part.renameTo(file)
        } catch (_: IOException) {
            false
        }
    }
}
