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

import android.content.ContentUris
import android.graphics.BitmapFactory
import android.media.MediaMetadataRetriever
import android.media.ThumbnailUtils
import android.util.Size
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Image
import androidx.compose.material.icons.filled.Videocam
import androidx.compose.material3.Icon
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFileStorage
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import org.florisboard.lib.compose.florisVerticalScroll
import org.florisboard.lib.snygg.ui.SnyggBox
import org.florisboard.lib.snygg.ui.SnyggText

@Composable
internal fun ClipboardItemView(
    elementName: String,
    item: ClipboardItem,
    contentScrollInsteadOfClip: Boolean,
    popupItem: ClipboardItem?,
    onPopup: (ClipboardItem) -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val clipboardManager by context.clipboardManager()
    val attributes = remember(item) {
        mapOf("type" to item.type.toString().lowercase())
    }
    SnyggBox(
        elementName = elementName,
        attributes = attributes,
        modifier = modifier.fillMaxWidth(),
        clickAndSemanticsModifier = Modifier.combinedClickable(
            interactionSource = remember { MutableInteractionSource() },
            indication = ripple(),
            enabled = popupItem == null,
            onLongClick = {
                onPopup(item)
            },
            onClick = {
                clipboardManager.pasteItem(item)
            },
        ),
    ) {
        if (item.type == ItemType.IMAGE) {
            val id = ContentUris.parseId(item.uri!!)
            val file = ClipboardFileStorage.getFileForId(context, id)
            val bitmap = remember(id) {
                runCatching {
                    check(file.exists()) { "Unable to resolve image at ${file.absolutePath}" }
                    val rawBitmap = BitmapFactory.decodeFile(file.absolutePath)
                    checkNotNull(rawBitmap) { "Unable to decode image at ${file.absolutePath}" }
                    rawBitmap.asImageBitmap()
                }
            }
            if (bitmap.isSuccess) {
                Image(
                    modifier = Modifier.fillMaxWidth(),
                    bitmap = bitmap.getOrThrow(),
                    contentDescription = null,
                    contentScale = ContentScale.FillWidth,
                )
            } else {
                SnyggText(
                    modifier = Modifier.fillMaxWidth(),
                    text = bitmap.exceptionOrNull()?.message ?: "Unknown error",
                )
            }
        } else if (item.type == ItemType.VIDEO) {
            val id = ContentUris.parseId(item.uri!!)
            val file = ClipboardFileStorage.getFileForId(context, id)
            val bitmap = remember(id) {
                runCatching {
                    check(file.exists()) { "Unable to resolve video at ${file.absolutePath}" }
                    val dataRetriever = MediaMetadataRetriever()
                    dataRetriever.setDataSource(file.absolutePath)
                    val width = dataRetriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_WIDTH)
                    val height = dataRetriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_HEIGHT)
                    val size = Size(width!!.toInt(), height!!.toInt())
                    val rawBitmap = ThumbnailUtils.createVideoThumbnail(file, size, null)
                    checkNotNull(rawBitmap) { "Unable to decode video at ${file.absolutePath}" }
                    rawBitmap.asImageBitmap()
                }
            }
            if (bitmap.isSuccess) {
                Image(
                    modifier = Modifier.fillMaxWidth(),
                    bitmap = bitmap.getOrThrow(),
                    contentDescription = null,
                    contentScale = ContentScale.FillWidth,
                )
                Icon(
                    modifier = Modifier
                        .align(Alignment.BottomStart)
                        .padding(start = 4.dp, bottom = 4.dp)
                        .background(Color.White, CircleShape),
                    imageVector = Icons.Default.Videocam,
                    contentDescription = null,
                    tint = Color.Black,
                )
            } else {
                SnyggText(
                    modifier = Modifier.fillMaxWidth(),
                    text = bitmap.exceptionOrNull()?.message ?: "Unknown error",
                )
            }
        } else {
            val text = item.stringRepresentation()
            Column {
                ClipTextItemDescription(
                    elementName = FlorisImeUi.ClipboardItemDescription.elementName,
                    attributes = attributes,
                    text = text,
                )
                SnyggText(
                    modifier = Modifier
                        .fillMaxWidth()
                        .run { if (contentScrollInsteadOfClip) this.florisVerticalScroll() else this },
                    text = item.displayText(),
                )
            }
        }
    }
}
