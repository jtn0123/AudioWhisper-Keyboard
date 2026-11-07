package com.jtn0123.audiowhisper.dictation

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import org.florisboard.lib.snygg.ui.rememberSnyggThemeQuery

/** For a theme that doesn't style the Enter key. */
private val FallbackAccent = Color(0xFF3E91FF)
private const val GLOW_MS = 120
private const val HALO_ALPHA = 0.35f
private const val DISC = 0.78f
private const val DISC_GROWTH = 0.14f
private const val FINISHING_ALPHA = 0.5f

/**
 * The suggestion bar's mic button while dictating: a disc whose halo breathes with the voice,
 * dimmed while the last phrase is being typed. Tapping it stops. It takes the Enter key's colours,
 * which every bundled theme sets to its accent, so it follows day, night and Material You.
 */
@Composable
fun LitMic(imageVector: ImageVector, modifier: Modifier = Modifier) {
    val status by Dictation.status.collectAsState()
    val level by Dictation.level.collectAsState()
    val glow by animateFloatAsState(level, animationSpec = tween(GLOW_MS), label = "mic glow")
    val discAlpha = if (status == Dictation.Status.LISTENING) 1f else FINISHING_ALPHA
    val enterKey = rememberSnyggThemeQuery(
        FlorisImeUi.Key.elementName,
        attributes = mapOf(FlorisImeUi.Attr.Code to KeyCode.ENTER),
    )
    val accent = enterKey.background(default = FallbackAccent).takeIf { it.alpha > 0f } ?: FallbackAccent
    val iconTint = enterKey.foreground(default = Color.White)
    Box(
        modifier = modifier
            .fillMaxSize()
            .padding(2.dp)
            .drawBehind {
                val radius = size.minDimension / 2
                drawCircle(accent.copy(alpha = HALO_ALPHA * glow), radius = radius)
                drawCircle(accent.copy(alpha = discAlpha), radius = radius * (DISC + DISC_GROWTH * glow))
            },
        contentAlignment = Alignment.Center,
    ) {
        Icon(imageVector = imageVector, contentDescription = "Stop dictating", tint = iconTint)
    }
}
