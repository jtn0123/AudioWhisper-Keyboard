package com.jtn0123.audiowhisper.render

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import com.jtn0123.audiowhisper.rust.KeyboardRenderer

/** Offscreen pixels only. The settings app keeps its accessible Material controls. */
@Composable
internal fun RustKeyPreview(code: Int, type: String, label: String, icon: ImageVector?) {
    val density = LocalDensity.current
    val background = MaterialTheme.colorScheme.onSurface.copy(alpha = .12f).toArgb()
    val foreground = LocalContentColor.current.toArgb()
    val path = icon?.let(::iconPath) ?: ""
    val bitmap = remember(code, type, label, path, background, foreground, density) {
        KeyboardRenderer(systemRendererFonts()).use { renderer ->
            val size = if (icon == null) 16 else 24
            val json = """{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key":{"background":"#${Integer.rotateLeft(background, 8).toUInt().toString(16).padStart(8, '0')}","foreground":"#${Integer.rotateLeft(foreground, 8).toUInt().toString(16).padStart(8, '0')}","font-size":"${size}sp","shape":"rounded-corner(12dp, 12dp, 12dp, 12dp)"}}"""
            renderer.theme(json, emptyArray(), intArrayOf(), floatArrayOf(density.density, density.fontScale, 1f))
            val pixels = renderer.keyPreview(code, arrayOf(type, label, path))
            Bitmap.createBitmap(pixels.copyOfRange(2, pixels.size), pixels[0], pixels[1], Bitmap.Config.ARGB_8888).asImageBitmap()
        }
    }
    Image(bitmap, contentDescription = null,
        modifier = Modifier.size((bitmap.width / density.density).dp, (bitmap.height / density.density).dp))
}
