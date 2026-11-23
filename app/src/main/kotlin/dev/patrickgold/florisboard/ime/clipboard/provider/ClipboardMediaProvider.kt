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

import android.content.ContentProvider
import android.content.ContentResolver
import android.content.ContentUris
import android.content.ContentValues
import android.content.Intent
import android.content.UriMatcher
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.BaseColumns
import android.provider.MediaStore
import android.provider.OpenableColumns
import androidx.exifinterface.media.ExifInterface
import com.jtn0123.audiowhisper.rust.ClipboardStore
import com.jtn0123.audiowhisper.rust.StoredFile
import dev.patrickgold.florisboard.BuildConfig
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogError
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import org.florisboard.lib.kotlin.tryOrNull
import androidx.core.net.toUri
import java.io.IOException

/**
 * Allows apps to access images and videos on the clipboard.
 *
 * The media files' records (name, size, orientation, MIME types) are kept by Rust in the clipboard's store
 * ([ClipboardStoreFile]), which this asks for them; the files themselves are copied, opened and deleted here.
 */
class ClipboardMediaProvider : ContentProvider() {
    private val ioScope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val store: ClipboardStore get() = ClipboardStoreFile.get(context!!)

    companion object {
        const val AUTHORITY = "${BuildConfig.APPLICATION_ID}.provider.clipboard"
        val IMAGE_CLIPS_URI: Uri = "content://$AUTHORITY/clips/images".toUri()
        val VIDEO_CLIPS_URI: Uri = "content://$AUTHORITY/clips/videos".toUri()

        private const val IMAGE_CLIP_ITEM = 0
        private const val IMAGE_CLIPS_TABLE = 1
        private const val VIDEO_CLIP_ITEM = 2
        private const val VIDEO_CLIPS_TABLE = 3

        /** A record's columns, as the Room table had them. */
        private val FILE_COLUMNS = arrayOf(BaseColumns._ID, OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE,
            MediaStore.Images.Media.ORIENTATION, "mimeTypes")

        private val Matcher = UriMatcher(UriMatcher.NO_MATCH).apply {
            addURI(AUTHORITY, "clips/images/#", IMAGE_CLIP_ITEM)
            addURI(AUTHORITY, "clips/images", IMAGE_CLIPS_TABLE)
            addURI(AUTHORITY, "clips/videos/#", VIDEO_CLIP_ITEM)
            addURI(AUTHORITY, "clips/videos", VIDEO_CLIPS_TABLE)
        }
    }

    object Columns {
        const val MediaUri = "media_uri"
        const val MimeTypes = "mime_types"
    }

    /** The record of the file [id], if the store has one. */
    private fun record(id: Long): StoredFile? = store.fileRecords(id).firstOrNull()

    override fun onCreate(): Boolean {
        // Opens the store before anything asks, moving main's Room databases into it the first time.
        ioScope.launch { store }
        return true
    }

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?
    ): Cursor? {
        val id = tryOrNull { ContentUris.parseId(uri) } ?: return null
        //Return null if the projection query is invalid
        if (projection != null && !projection.contains(MediaStore.Images.Media.ORIENTATION)) return null
        return MatrixCursor(FILE_COLUMNS).apply {
            record(id)?.let { addRow(arrayOf<Any>(it.id, it.name, it.size, it.orientation, it.mimeTypes)) }
        }
    }

    override fun getType(uri: Uri): String? {
        return when (Matcher.match(uri)) {
            IMAGE_CLIP_ITEM, VIDEO_CLIP_ITEM -> {
                record(ContentUris.parseId(uri))?.mimeTypes?.split(",")?.getOrNull(0)
            }
            IMAGE_CLIPS_TABLE -> "${ContentResolver.CURSOR_DIR_BASE_TYPE}/vnd.florisboard.image_clip_table"
            VIDEO_CLIPS_TABLE -> "${ContentResolver.CURSOR_DIR_BASE_TYPE}/vnd.florisboard.video_clip_table"
            else -> null
        }
    }

    override fun getStreamTypes(uri: Uri, mimeTypeFilter: String): Array<String>? {
        return when (Matcher.match(uri)) {
            IMAGE_CLIP_ITEM, VIDEO_CLIP_ITEM -> {
                record(ContentUris.parseId(uri))?.mimeTypes?.split(",")?.toTypedArray()
            }
            else -> null
        }
    }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor {
        val id = ContentUris.parseId(uri)
        val file = ClipboardFileStorage.getFileForId(context!!, id)

        // Nothing has permission to write anyway.
        return ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY)
    }

    override fun insert(uri: Uri, values: ContentValues?): Uri {
        when (val m = Matcher.match(uri)) {
            IMAGE_CLIPS_TABLE, VIDEO_CLIPS_TABLE -> {
                return try {
                    values as ContentValues
                    val mediaUri = values.getAsString(Columns.MediaUri).toUri()
                    // Get the orientation of the image
                    val exifInterface = ExifInterface(context!!.contentResolver.openInputStream(mediaUri)!!)
                    var rotation = 0
                    val orientation = exifInterface.getAttributeInt(
                        ExifInterface.TAG_ORIENTATION,
                        ExifInterface.ORIENTATION_NORMAL
                    )
                    when (orientation) {
                        ExifInterface.ORIENTATION_ROTATE_90 -> rotation = 90
                        ExifInterface.ORIENTATION_ROTATE_180 -> rotation = 180
                        ExifInterface.ORIENTATION_ROTATE_270 -> rotation = 270
                    }
                    val id = ClipboardFileStorage.cloneUri(context!!, mediaUri)
                    val size = ClipboardFileStorage.getFileForId(context!!, id).length()
                    val mimeTypes = values.getAsString(Columns.MimeTypes).split(",")
                    val displayName = values.getAsString(OpenableColumns.DISPLAY_NAME)
                    store.addFile(id, displayName, size, rotation, mimeTypes.joinToString(","))
                    if (m == IMAGE_CLIPS_TABLE) {
                        ContentUris.withAppendedId(IMAGE_CLIPS_URI, id)
                    } else {
                        ContentUris.withAppendedId(VIDEO_CLIPS_URI, id)
                    }
                } catch (e: Exception) {
                    flogError { e.message.toString() }
                    uri.buildUpon().appendPath("0").build()
                }
            }
            else -> error("Unable to identify type of $uri")
        }
    }

    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int {
        when (Matcher.match(uri)) {
            IMAGE_CLIP_ITEM, VIDEO_CLIP_ITEM -> {
                val id = ContentUris.parseId(uri)
                ClipboardFileStorage.deleteById(context!!, id)
                try {
                    store.removeFile(id)
                } catch (e: IOException) {
                    flogError(LogTopic.CLIPBOARD) { "Media record not removed: ${e.javaClass.simpleName}" }
                }
                context?.revokeUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
                return 1
            }
            else -> error("Unable to identify type of $uri")
        }
    }

    override fun update(uri: Uri, values: ContentValues?, selection: String?, selectionArgs: Array<out String>?): Int {
        error("This ContentProvider does not support update.")
    }
}
