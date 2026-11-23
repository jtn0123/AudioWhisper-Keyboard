/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.ime.dictionary

import android.content.ContentValues
import android.content.Context
import android.database.Cursor
import android.provider.UserDictionary
import dev.patrickgold.florisboard.lib.FlorisLocale
import java.lang.ref.WeakReference

private const val SORT_BY_FREQ_DESC = "${UserDictionary.Words.FREQUENCY} DESC"

private val PROJECTIONS: Array<String> = arrayOf(
    UserDictionary.Words._ID,
    UserDictionary.Words.WORD,
    UserDictionary.Words.FREQUENCY,
    UserDictionary.Words.LOCALE,
    UserDictionary.Words.SHORTCUT,
)

data class UserDictionaryEntry(
    val id: Long,
    val word: String,
    val freq: Int,
    val locale: String?,
    val shortcut: String?,
)

/**
 * A user dictionary's words: the keyboard's own ([FlorisUserDictionaryDatabase]) or Android's
 * ([SystemUserDictionaryDatabase]).
 */
interface UserDictionaryDao {
    /** Every word: the Floris dictionary's in id order, the system's by frequency, highest first. */
    fun queryAll(): List<UserDictionaryEntry>

    /** The words that are [word] exactly, stored under [locale] exactly (no locale for null). */
    fun queryExact(word: String, locale: FlorisLocale?): List<UserDictionaryEntry>

    fun insert(entry: UserDictionaryEntry)

    fun update(entry: UserDictionaryEntry)

    fun delete(entry: UserDictionaryEntry)
}

/** Android's own user dictionary, read and written through its provider as before. */
class SystemUserDictionaryDatabase(context: Context) {
    private val applicationContext: WeakReference<Context> = WeakReference(context.applicationContext ?: context)

    private val dao = object : UserDictionaryDao {
        override fun queryAll(): List<UserDictionaryEntry> {
            return queryResolver(
                selection = null,
                selectionArgs = null,
                sortOrder = SORT_BY_FREQ_DESC,
            )
        }

        override fun queryExact(word: String, locale: FlorisLocale?): List<UserDictionaryEntry> {
            return if (locale == null) {
                queryResolver(
                    selection = "${UserDictionary.Words.WORD} = ? AND ${UserDictionary.Words.LOCALE} IS NULL",
                    selectionArgs = arrayOf(word),
                    sortOrder = SORT_BY_FREQ_DESC,
                )
            } else {
                queryResolver(
                    selection = "${UserDictionary.Words.WORD} = ? AND ${UserDictionary.Words.LOCALE} = ?",
                    selectionArgs = arrayOf(word, locale.localeTag()),
                    sortOrder = SORT_BY_FREQ_DESC,
                )
            }
        }

        private fun queryResolver(selection: String?, selectionArgs: Array<out String>?, sortOrder: String?): List<UserDictionaryEntry> {
            val resolver = applicationContext.get()?.contentResolver ?: return listOf()
            val cursor = resolver.query(
                UserDictionary.Words.CONTENT_URI,
                PROJECTIONS,
                selection,
                selectionArgs,
                sortOrder
            ) ?: return listOf()
            return parseEntries(cursor).also { cursor.close() }
        }

        private fun parseEntries(cursor: Cursor): List<UserDictionaryEntry> {
            if (cursor.count <= 0) {
                return listOf()
            }
            val idIndex = cursor.getColumnIndex(UserDictionary.Words._ID)
            val wordIndex = cursor.getColumnIndex(UserDictionary.Words.WORD)
            val freqIndex = cursor.getColumnIndex(UserDictionary.Words.FREQUENCY)
            val localeIndex = cursor.getColumnIndex(UserDictionary.Words.LOCALE)
            val shortcutIndex = cursor.getColumnIndex(UserDictionary.Words.SHORTCUT)
            val retList = mutableListOf<UserDictionaryEntry>()
            while (cursor.moveToNext()) {
                retList.add(
                    UserDictionaryEntry(
                        id = cursor.getLong(idIndex),
                        word = cursor.getString(wordIndex),
                        freq = cursor.getInt(freqIndex),
                        locale = cursor.getString(localeIndex),
                        shortcut = cursor.getString(shortcutIndex)
                    )
                )
            }
            return retList
        }

        override fun insert(entry: UserDictionaryEntry) {
            val resolver = applicationContext.get()?.contentResolver ?: return
            val contentValues = ContentValues(5).apply {
                put(UserDictionary.Words.WORD, entry.word)
                put(UserDictionary.Words.FREQUENCY, entry.freq)
                put(UserDictionary.Words.LOCALE, entry.locale)
                put(UserDictionary.Words.APP_ID, 0)
                put(UserDictionary.Words.SHORTCUT, entry.shortcut)
            }
            resolver.insert(UserDictionary.Words.CONTENT_URI, contentValues)
        }

        override fun update(entry: UserDictionaryEntry) {
            val resolver = applicationContext.get()?.contentResolver ?: return
            val contentValues = ContentValues(4).apply {
                put(UserDictionary.Words.WORD, entry.word)
                put(UserDictionary.Words.FREQUENCY, entry.freq)
                put(UserDictionary.Words.LOCALE, entry.locale)
                put(UserDictionary.Words.SHORTCUT, entry.shortcut)
            }
            resolver.update(UserDictionary.Words.CONTENT_URI, contentValues, "${UserDictionary.Words._ID} = ${entry.id}", null)
        }

        override fun delete(entry: UserDictionaryEntry) {
            val resolver = applicationContext.get()?.contentResolver ?: return
            resolver.delete(UserDictionary.Words.CONTENT_URI, "${UserDictionary.Words._ID} = ${entry.id}", null)
        }
    }

    fun userDictionaryDao(): UserDictionaryDao {
        return dao
    }
}
