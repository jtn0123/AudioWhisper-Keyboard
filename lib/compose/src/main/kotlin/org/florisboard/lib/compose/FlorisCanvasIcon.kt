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

package org.florisboard.lib.compose

import android.graphics.Canvas
import android.graphics.drawable.Drawable
import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalResources
import androidx.core.content.res.ResourcesCompat
import androidx.core.graphics.createBitmap

@Composable
fun FlorisCanvasIcon(
    @DrawableRes iconId: Int,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
) {
    val resources = LocalResources.current
    remember(resources, iconId) { ResourcesCompat.getDrawable(resources, iconId, null) }?.let { drawable ->
        FlorisCanvasIcon(
            drawable = drawable,
            modifier = modifier,
            contentDescription = contentDescription,
        )
    }
}

@Composable
fun FlorisCanvasIcon(
    drawable: Drawable,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
) {
    // Drawn once per drawable, not on every recomposition.
    val bitmap = remember(drawable) {
        createBitmap(width = drawable.intrinsicWidth, height = drawable.intrinsicHeight).also { target ->
            val canvas = Canvas(target)
            drawable.setBounds(0, 0, canvas.width, canvas.height)
            drawable.draw(canvas)
        }.asImageBitmap()
    }
    Image(
        modifier = modifier,
        bitmap = bitmap,
        contentDescription = contentDescription,
    )
}
