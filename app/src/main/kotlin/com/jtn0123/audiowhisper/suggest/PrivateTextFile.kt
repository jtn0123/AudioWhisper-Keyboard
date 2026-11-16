package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.io.IOException

/**
 * A small text file in the app's private files, replaced whole only once the new copy is
 * written, so a crash mid-save leaves the old one. Its contents are never logged.
 */
class PrivateTextFile(private val file: File) {
    /** The saved text, or "" if there is none or it can't be read. Blocking. */
    fun read(): String = try {
        if (file.isFile) file.readText(Charsets.UTF_8) else ""
    } catch (_: IOException) {
        ""
    }

    /** Saves [text]; false if it could not be. Blocking. */
    @Synchronized
    fun write(text: String): Boolean {
        val partial = File(file.path + ".tmp")
        return try {
            partial.writeText(text, Charsets.UTF_8)
            partial.renameTo(file)
        } catch (_: IOException) {
            false
        }
    }

    /** Removes the file. Blocking. */
    @Synchronized
    fun delete() {
        file.delete()
    }
}
