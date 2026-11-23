package dev.patrickgold.florisboard.ime.dictionary

import android.content.Context
import com.jtn0123.audiowhisper.rust.StoredWord
import com.jtn0123.audiowhisper.rust.UserWordStore
import com.jtn0123.audiowhisper.rust.WordFlag
import com.jtn0123.audiowhisper.stores.LegacyRoom
import dev.patrickgold.florisboard.lib.FlorisLocale
import java.io.File
import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock

/**
 * The keyboard's own user dictionary, kept by Rust (crates/stores) in files/user-dictionary.store with the
 * semantics its Room table had: words in id order, an id never handed out twice, a taken id refused. One per
 * process; opening it moves main's Room database into it, once. A transaction holds the dictionary's lock, so the
 * suggestions never read an import half done.
 */
class FlorisUserDictionaryDatabase private constructor(private val store: UserWordStore) {
    companion object {
        const val STORE_FILE_NAME = "user-dictionary.store"

        @Volatile private var instance: FlorisUserDictionaryDatabase? = null

        fun get(context: Context): FlorisUserDictionaryDatabase = instance ?: synchronized(this) {
            instance ?: open(context.applicationContext ?: context).also { instance = it }
        }

        private fun open(context: Context): FlorisUserDictionaryDatabase {
            val store = UserWordStore(File(context.filesDir, STORE_FILE_NAME).path)
            LegacyRoom.importDictionary(context, store)
            return FlorisUserDictionaryDatabase(store)
        }

        /** The dictionary over the store at [path], for tests. */
        internal fun at(path: String) = FlorisUserDictionaryDatabase(UserWordStore(path))
    }

    private val lock = ReentrantLock()

    private val dao = object : UserDictionaryDao {
        override fun queryAll() = lock.withLock { store.all().map(::entry) }

        override fun queryExact(word: String, locale: FlorisLocale?) =
            lock.withLock { store.exact(word, locale?.localeTag()).map(::entry) }

        override fun insert(entry: UserDictionaryEntry) {
            lock.withLock { store.insert(entry.id, entry.word, entry.freq, entry.locale, entry.shortcut) }
        }

        override fun update(entry: UserDictionaryEntry) =
            lock.withLock { store.update(entry.id, entry.word, entry.freq, entry.locale, entry.shortcut) }

        override fun delete(entry: UserDictionaryEntry) = lock.withLock { store.delete(entry.id) }
    }

    fun userDictionaryDao(): UserDictionaryDao = dao

    /** Runs [block] as one change: saved once if it returns, dropped if it throws, as Room's transaction was. */
    fun runInTransaction(block: () -> Unit) = lock.withLock {
        store.begin()
        try {
            block()
        } catch (e: Throwable) {
            store.rollback()
            throw e
        }
        store.commit()
    }

    private fun entry(word: StoredWord) = UserDictionaryEntry(word.id, word.word, word.frequency,
        word.locale.takeIf { word.flags and WordFlag.HAS_LOCALE.code != 0 },
        word.shortcut.takeIf { word.flags and WordFlag.HAS_SHORTCUT.code != 0 })
}
