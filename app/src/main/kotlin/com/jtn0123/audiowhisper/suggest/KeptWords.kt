package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.io.IOException
import java.util.Locale

/**
 * Words whose auto-correction the user undid, so they are not corrected again. Kept in [file],
 * one lower-cased word per line, newest last, at most [capacity] of them. A word kept with
 * remember = false (incognito) lasts only as long as this object and is never written.
 *
 * Thread-safe: words are added on the main thread and read while suggesting.
 */
class KeptWords(private val file: File?, private val capacity: Int = CAPACITY) {
    private val remembered = LinkedHashSet<String>()
    private val sessionOnly = HashSet<String>()

    /** Two saves at once would write the same temporary file. */
    private val saveLock = Any()

    @Synchronized
    operator fun contains(word: String): Boolean = key(word).let { it in remembered || it in sessionOnly }

    /** Keeps [word]; true when the change needs [save]. */
    @Synchronized
    fun add(word: String, remember: Boolean): Boolean {
        val key = key(word)
        if (key.isBlank() || '\n' in key) return false
        if (!remember) {
            sessionOnly.add(key)
            return false
        }
        sessionOnly.remove(key)
        remembered.remove(key)
        remembered.add(key)
        trim()
        return true
    }

    /** Reads the saved words, ahead of any added before it finished. Blocking. */
    fun load() {
        val saved = try {
            file?.takeIf { it.isFile }?.readLines(Charsets.UTF_8).orEmpty()
        } catch (_: IOException) {
            emptyList()
        }
        synchronized(this) {
            val added = remembered.toList()
            remembered.clear()
            saved.map(String::trim).filter(String::isNotEmpty).forEach { remembered.add(key(it)) }
            remembered.removeAll(added.toSet())
            remembered.addAll(added)
            trim()
        }
    }

    /** Writes the remembered words, replacing the file only once the new one is complete. Blocking. */
    fun save(): Boolean = synchronized(saveLock) {
        val file = file ?: return false
        val lines = synchronized(this) { remembered.joinToString("\n", postfix = "\n") }
        val partial = File(file.path + ".tmp")
        try {
            partial.writeText(lines, Charsets.UTF_8)
            partial.renameTo(file)
        } catch (_: IOException) {
            false
        }
    }

    private fun trim() {
        val iterator = remembered.iterator()
        while (remembered.size > capacity && iterator.hasNext()) {
            iterator.next()
            iterator.remove()
        }
    }

    private fun key(word: String) = word.lowercase(Locale.ROOT)

    companion object {
        /** Enough for years of undos; the oldest go first. */
        const val CAPACITY = 1000
    }
}
