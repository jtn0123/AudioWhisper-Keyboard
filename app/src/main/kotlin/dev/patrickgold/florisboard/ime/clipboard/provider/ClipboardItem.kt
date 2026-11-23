/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.clipboard.provider

import android.content.ClipData
import android.content.ClipDescription.EXTRA_IS_REMOTE_DEVICE
import android.content.ClipDescription.EXTRA_IS_SENSITIVE
import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.core.database.getStringOrNull
import dev.patrickgold.florisboard.R
import kotlinx.serialization.EncodeDefault
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.Serializable
import org.florisboard.lib.android.UriSerializer
import org.florisboard.lib.android.query
import org.florisboard.lib.android.stringRes
import org.florisboard.lib.kotlin.tryOrNull

enum class ItemType(val value: Int) {
    TEXT(1),
    IMAGE(2),
    VIDEO(3);
}

/**
 * Represents an item on the clipboard.
 *
 * If type == ItemType.IMAGE there must be a uri set
 * if type == ItemType.TEXT there must be a text set
 */
@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class ClipboardItem(
    var id: Long = 0,
    val type: ItemType,
    val text: String?,
    @Serializable(with = UriSerializer::class)
    val uri: Uri?,
    val creationTimestampMs: Long,
    val isPinned: Boolean,
    val mimeTypes: List<String>,
    @EncodeDefault
    val isSensitive: Boolean = false,
    @EncodeDefault
    val isRemoteDevice: Boolean = false,
) {
    companion object {
        /**
         * So that every item doesn't have to allocate its own array.
         */
        private val TEXT_PLAIN = listOf("text/plain")
        private val MEDIA_PROJECTION = arrayOf(OpenableColumns.DISPLAY_NAME)

        const val FLORIS_CLIP_LABEL = "florisboard/clipboard_item"

        fun text(text: String): ClipboardItem {
            return ClipboardItem(
                type = ItemType.TEXT,
                text = text,
                uri = null,
                creationTimestampMs = System.currentTimeMillis(),
                isPinned = false,
                mimeTypes = TEXT_PLAIN,
            )
        }

        /**
         * Returns a new ClipboardItem based on a ClipData.
         *
         * @param data The ClipData to clone.
         * @param cloneUri Whether to store the image using [ClipboardMediaProvider].
         */
        fun fromClipData(context: Context, data: ClipData, cloneUri: Boolean) : ClipboardItem {
            val dataItem = data.getItemAt(0)
            val type = when {
                dataItem?.uri != null && data.description.hasMimeType("image/*") -> ItemType.IMAGE
                dataItem?.uri != null && data.description.hasMimeType("video/*") -> ItemType.VIDEO
                else -> ItemType.TEXT
            }

            val isSensitive = data.description?.extras?.getBoolean(EXTRA_IS_SENSITIVE) ?: false
            val isRemoteDevice = data.description?.extras?.getBoolean(EXTRA_IS_REMOTE_DEVICE) ?: false

            val uri = if (type == ItemType.IMAGE || type == ItemType.VIDEO) {
                if (dataItem.uri.authority == ClipboardMediaProvider.AUTHORITY || !cloneUri) {
                    dataItem.uri
                } else {
                    var displayName = when (type) {
                        ItemType.IMAGE -> "Image"
                        ItemType.VIDEO -> "Video"
                    }
                    tryOrNull {
                        context.contentResolver.query(dataItem.uri, MEDIA_PROJECTION)?.use { cursor ->
                            val displayNameColumn = cursor.getColumnIndexOrThrow(OpenableColumns.DISPLAY_NAME)
                            if (cursor.moveToNext()) {
                                cursor.getStringOrNull(displayNameColumn)?.let { displayName = it }
                            }
                        }
                    }
                    val values = ContentValues(3).apply {
                        put(OpenableColumns.DISPLAY_NAME, displayName)
                        put(ClipboardMediaProvider.Columns.MediaUri, dataItem.uri.toString())
                        put(ClipboardMediaProvider.Columns.MimeTypes, data.description.filterMimeTypes("*/*").joinToString(","))
                    }
                    context.contentResolver.insert(when (type) {
                        ItemType.IMAGE -> ClipboardMediaProvider.IMAGE_CLIPS_URI
                        ItemType.VIDEO -> ClipboardMediaProvider.VIDEO_CLIPS_URI
                    }, values)
                }
            } else { null }

            val text = dataItem.text?.toString()
            val mimeTypes = when (type) {
                ItemType.TEXT -> TEXT_PLAIN
                ItemType.IMAGE, ItemType.VIDEO -> {
                    List(data.description.mimeTypeCount) { data.description.getMimeType(it) }
                }
            }

            return ClipboardItem(0, type, text, uri, System.currentTimeMillis(), false, mimeTypes, isSensitive, isRemoteDevice)
        }
    }

    @Composable
    fun displayText(): String {
        val context = LocalContext.current
        return displayText(context)
    }

    fun displayText(context: Context): String {
        return if (isSensitive) {
            context.stringRes(R.string.clipboard__sensitive_clip_content)
        } else {
            stringRepresentation()
        }
    }

    infix fun isEqualTo(other: ClipData?): Boolean {
        if (other == null) return false
        return when (type) {
            ItemType.TEXT -> text == other.getItemAt(0).text
            ItemType.IMAGE, ItemType.VIDEO -> uri == other.getItemAt(0).uri
        }
    }

    /**
     * Creates a new ClipData which has the same contents as this.
     */
    fun toClipData(context: Context): ClipData {
        return when (type) {
            ItemType.TEXT -> {
                ClipData.newPlainText(FLORIS_CLIP_LABEL, text)
            }
            ItemType.IMAGE, ItemType.VIDEO -> {
                ClipData.newUri(context.contentResolver, FLORIS_CLIP_LABEL, uri)
            }
        }
    }

    fun stringRepresentation(): String {
        return when {
            text != null -> text
            uri != null -> "(Image) $uri"
            else -> "#ERROR"
        }
    }
}
