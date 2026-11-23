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

package dev.patrickgold.florisboard.ime.clipboard

import android.content.ClipData
import android.content.Context
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.editorInstance
import androidx.core.net.toUri
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardStoreFile
import dev.patrickgold.florisboard.ime.clipboard.provider.addItem
import dev.patrickgold.florisboard.ime.clipboard.provider.restoreItems
import dev.patrickgold.florisboard.ime.clipboard.provider.toItem
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogError
import com.jtn0123.audiowhisper.rust.ClipboardRules
import com.jtn0123.audiowhisper.rust.ClipboardStore
import kotlinx.coroutines.CompletableJob
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.florisboard.lib.android.AndroidClipboardManager
import org.florisboard.lib.android.AndroidClipboardManager_OnPrimaryClipChangedListener
import org.florisboard.lib.android.setOrClearPrimaryClip
import org.florisboard.lib.android.showShortToast
import org.florisboard.lib.android.systemService
import org.florisboard.lib.kotlin.tryOrNull
import java.io.Closeable
import java.io.IOException

/**
 * [ClipboardManager] manages the clipboard and clipboard history.
 *
 * Also just going to document how all the classes here work.
 *
 * [ClipboardManager] handles storage and retrieval of clipboard items. All manipulation of the
 * clipboard goes through here. The history itself is kept by Rust ([ClipboardStoreFile]), which applies
 * every rule of it (duplicates, pins, the size limit, expiry); each change runs in order on one thread,
 * then the history the store holds is shown.
 */
class ClipboardManager(
    context: Context,
) : AndroidClipboardManager_OnPrimaryClipChangedListener, Closeable {
    companion object {
        // 1 minute
        private const val INTERVAL = 60 * 1000L

        /**
         * Taken from ClipboardDescription.java from the AOSP
         *
         * Helper to compare two MIME types, where one may be a pattern.
         * @param concreteType A fully-specified MIME type.
         * @param desiredType A desired MIME type that may be a pattern such as * / *.
         * @return Returns true if the two MIME types match.
         */
        fun compareMimeTypes(concreteType: String, desiredType: String): Boolean {
            return ClipboardRules.mimeMatches(concreteType, desiredType)
        }
    }

    private val prefs by FlorisPreferenceStore
    private val appContext by context.appContext()
    private val editorInstance by context.editorInstance()
    private val systemClipboardManager = context.systemService(AndroidClipboardManager::class)

    private val ioScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    /**
     * The history's changes, made one at a time in the order asked, once the settings are loaded: each applies
     * the size limit, which must not be the default one while the user's is still loading.
     */
    private val changes = Channel<Pair<(ClipboardStore) -> Unit, CompletableJob>>(Channel.UNLIMITED)
    private var cleanUpJob: Job

    private val _historyFlow = MutableStateFlow(ClipboardHistory.EMPTY)
    val historyFlow = _historyFlow.asStateFlow()
    val currentHistory: ClipboardHistory
        get() = historyFlow.value

    private val primaryClipLastFromCallbackGuard = Mutex(locked = false)
    private var primaryClipLastFromCallback: ClipData? = null
    private val _primaryClipFlow = MutableStateFlow<ClipboardItem?>(null)
    val primaryClipFlow = _primaryClipFlow.asStateFlow()
    inline var primaryClip
        get() = primaryClipFlow.value
        private set(v) {
            _primaryClipFlow.value = v
        }

    init {
        systemClipboardManager.addPrimaryClipChangedListener(this)
        ioScope.launch {
            appContext.preferenceStoreLoaded.first { it }
            for ((step, done) in changes) {
                try {
                    change(step)
                } finally {
                    done.complete()
                }
            }
        }
        cleanUpJob = ioScope.launch {
            while (isActive) {
                delay(INTERVAL)
                later { store ->
                    store.expire(now(), prefs.clipboard.historyAutoCleanOldEnabled.get(),
                        prefs.clipboard.historyAutoCleanOldAfter.get(),
                        prefs.clipboard.historyAutoCleanSensitiveEnabled.get(),
                        prefs.clipboard.historyAutoCleanSensitiveAfter.get(), limitEnabled(), limit())
                }
            }
        }
    }

    /** Opens the store (moving main's Room history into it the first time) and shows its history. */
    fun initialize() {
        later { store -> store.enforce(now(), limitEnabled(), limit()) }
    }

    private fun now() = System.currentTimeMillis()
    private fun limitEnabled() = prefs.clipboard.historySizeLimitEnabled.get()
    private fun limit() = prefs.clipboard.historySizeLimit.get()

    /**
     * Makes [change] to the store, then shows the history it holds. A store that couldn't be saved, or is kept
     * read-only, is left as it was: the failure is logged, never a clip.
     */
    private fun change(change: (ClipboardStore) -> Unit) {
        val store = ClipboardStoreFile.get(appContext)
        try {
            change(store)
        } catch (e: IOException) {
            flogError(LogTopic.CLIPBOARD) { "Clipboard history not saved: ${e.javaClass.simpleName}" }
        }
        _historyFlow.value = ClipboardHistory(store.items().mapNotNull { it.toItem() })
    }

    /** [change], after every change asked before it; the job is done once it is. */
    private fun later(change: (ClipboardStore) -> Unit): Job = Job().also { changes.trySend(change to it) }

    /** Deletes the media files no clip refers to any more, as their clips' removal left them. */
    private fun deleteMedia(uris: List<String>) {
        for (uri in uris) tryOrNull { appContext.contentResolver.delete(uri.toUri(), null, null) }
    }

    /**
     * Sets the current primary clip without updating the internal clipboard history.
     */
    fun updatePrimaryClip(item: ClipboardItem?) {
        primaryClip = item
        if (prefs.clipboard.useInternalClipboard.get()) {
            val syncBehavior = prefs.clipboard.syncToSystem.get()
            val clipData = item?.toClipData(appContext)
            if (clipData != null && syncBehavior.shouldSyncSet) {
                systemClipboardManager.setPrimaryClip(clipData)
            } else if (clipData == null && syncBehavior.shouldSyncClear) {
                systemClipboardManager.clearPrimaryClip()
            }
        } else {
            systemClipboardManager.setOrClearPrimaryClip(item?.toClipData(appContext))
        }
    }

    /**
     * Called by system clipboard when the system primary clip has changed.
     */
    override fun onPrimaryClipChanged() {
        val syncBehavior = prefs.clipboard.syncToFloris.get()
        if (!prefs.clipboard.useInternalClipboard.get() || syncBehavior != ClipboardSyncBehavior.NO_EVENTS) {
            val systemPrimaryClip = systemClipboardManager.primaryClip
            ioScope.launch {
                val isDuplicate: Boolean
                primaryClipLastFromCallbackGuard.withLock {
                    val a = primaryClipLastFromCallback?.getItemAt(0)
                    val b = systemPrimaryClip?.getItemAt(0)
                    isDuplicate = when {
                        a === b -> true
                        a == null || b == null -> false
                        else -> a.text == b.text && a.uri == b.uri
                    }
                    primaryClipLastFromCallback = systemPrimaryClip
                }
                if (isDuplicate) return@launch

                val internalPrimaryClip = primaryClip

                if (systemPrimaryClip == null) {
                    if (syncBehavior.shouldSyncClear) {
                        primaryClip = null
                    }
                    return@launch
                }

                if (systemPrimaryClip.getItemAt(0).let { it.text == null && it.uri == null }) {
                    if (syncBehavior.shouldSyncClear) {
                        primaryClip = null
                    }
                    return@launch
                }

                if (!syncBehavior.shouldSyncSet) {
                    return@launch
                }

                val isEqual = internalPrimaryClip?.isEqualTo(systemPrimaryClip) == true
                if (!isEqual) {
                    val item = ClipboardItem.fromClipData(appContext, systemPrimaryClip, cloneUri = true)
                    primaryClip = item
                    insertOrMoveBeginning(item)
                }
            }
        }
    }

    /**
     * Change the current text on clipboard, update history (if enabled).
     */
    private fun addNewClip(item: ClipboardItem) {
        insertOrMoveBeginning(item)
        updatePrimaryClip(item)
    }

    /**
     * Wraps some plaintext in a ClipData and calls [addNewClip]
     */
    fun addNewPlaintext(newText: String) {
        val newData = ClipboardItem.text(newText)
        addNewClip(newData)
    }

    /**
     * Adds a new item to the clipboard history (if enabled): a text the history has already moves to the top
     * instead, pinned if either was.
     */
    private fun insertOrMoveBeginning(newItem: ClipboardItem) {
        if (!prefs.clipboard.historyEnabled.get()) return
        later { store ->
            newItem.id = store.addItem(newItem, now(), limitEnabled(), limit())
        }
    }

    fun clearExactHistory(items: List<ClipboardItem>) {
        later { store -> deleteMedia(store.delete(items.map { it.id }.toLongArray(), false, now(), limitEnabled(), limit())) }
    }

    /**
     * Clears all unpinned items from the clipboard history
     */
    fun clearHistory() {
        later { store -> deleteMedia(store.clear(true, now(), limitEnabled(), limit())) }
    }

    /**
     * Clears the full clipboard history; the job is done once the history and its media files are gone.
     */
    fun clearFullHistory(): Job = later { store -> deleteMedia(store.clear(false, now(), limitEnabled(), limit())) }


    /**
     * Restore the clipboard history from a [List]
     *
     * @param items the [ClipboardItem] list with the new items
     */
    fun restoreHistory(items: List<ClipboardItem>): Job = later { store ->
        store.restoreItems(items, now(), limitEnabled(), limit())
    }

    fun deleteClip(item: ClipboardItem, onlyIfUnpinned: Boolean) {
        later { store -> deleteMedia(store.delete(longArrayOf(item.id), onlyIfUnpinned, now(), limitEnabled(), limit())) }
    }

    fun pinClip(item: ClipboardItem) {
        later { store -> store.pin(item.id, true, now(), limitEnabled(), limit()) }
    }

    fun unpinClip(item: ClipboardItem) {
        later { store -> store.pin(item.id, false, now(), limitEnabled(), limit()) }
    }

    fun pasteItem(item: ClipboardItem) {
        val editorInstance by appContext.editorInstance()
        editorInstance.commitClipboardItem(item).also { result ->
            if (!result) {
                ioScope.launch { appContext.showShortToast("Failed to paste item.") }
            }
        }
    }

    /**
     * Returns true if the editor can accept the clip item, else false.
     */
    fun canBePasted(clipItem: ClipboardItem?): Boolean {
        if (clipItem == null) return false

        return ClipboardRules.eligible(clipItem.mimeTypes.toTypedArray(), editorInstance.activeInfo.contentMimeTypes)
    }

    /**
     * Cleans up.
     *
     * Unregisters the system clipboard listener, cancels clipboard clean ups.
     */
    override fun close() {
        systemClipboardManager.removePrimaryClipChangedListener(this)
        cleanUpJob.cancel()
    }
}
