package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.KeptWordStore
import com.jtn0123.audiowhisper.rust.WordStoreRules
import java.io.File
import java.io.IOException

/**
 * Words whose auto-correction the user undid, so they are not corrected again. Kept in [file],
 * one lower-cased word per line, newest last, at most [capacity] of them. A word kept with
 * remember = false (incognito) lasts only as long as this object and is never written. The words
 * live in Rust ([KeptWordStore], lib/rust/crates/suggest/src/kept_words.rs); this reads and writes
 * the file.
 *
 * Thread-safe: words are added on the main thread and read while suggesting.
 */
class KeptWords(private val file: File?, capacity: Int = CAPACITY) {
    private val store = KeptWordStore(capacity)

    /** Two saves at once would write the same temporary file. */
    private val saveLock = Any()

    operator fun contains(word: String): Boolean = word in store

    /** Keeps [word]; true when the change needs [save]. */
    fun add(word: String, remember: Boolean): Boolean = store.add(word, remember)

    /** Forgets every kept word; [save] then writes an empty file. */
    fun clear() = store.clear()

    /** Reads the saved words, ahead of any added before it finished. Blocking. */
    fun load() {
        val saved = try {
            file?.takeIf { it.isFile }?.readText(Charsets.UTF_8).orEmpty()
        } catch (_: IOException) {
            ""
        }
        store.merge(saved)
    }

    /** Writes the remembered words, replacing the file only once the new one is complete. Blocking. */
    fun save(): Boolean = synchronized(saveLock) {
        val file = file ?: return false
        val text = store.savedText()
        val partial = File(file.path + ".tmp")
        try {
            partial.writeText(text, Charsets.UTF_8)
            partial.renameTo(file)
        } catch (_: IOException) {
            false
        }
    }

    companion object {
        /** Enough for years of undos; the oldest go first. */
        val CAPACITY: Int = WordStoreRules.keptCapacity
    }
}
