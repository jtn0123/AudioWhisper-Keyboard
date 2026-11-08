package com.jtn0123.audiowhisper.dictation

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
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
private const val WARMING_TURN_MS = 900
private const val WARMING_SWEEP = 100f
private val WarmingStroke = 2.dp

/**
 * The suggestion bar's mic button while dictating: a disc whose halo breathes with the voice,
 * dimmed while the last phrase is being typed. Tapping it stops. It takes the Enter key's colours,
 * which every bundled theme sets to its accent, so it follows day, night and Material You.
 * While the speech model loads a ring turns around it: it is already listening, but the first
 * phrase will take a moment longer.
 */
@Composable
fun LitMic(imageVector: ImageVector, modifier: Modifier = Modifier) {
    val status by Dictation.status.collectAsState()
    val level by Dictation.level.collectAsState()
    val warming by Dictation.warming.collectAsState()
    val glow by animateFloatAsState(level, animationSpec = tween(GLOW_MS), label = "mic glow")
    val discAlpha = if (status == Dictation.Status.LISTENING) 1f else FINISHING_ALPHA
    val (accent, iconTint) = keyAccent()
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
        if (warming) WarmingRing(accent)
        Icon(imageVector = imageVector, contentDescription = "Stop dictating", tint = iconTint)
    }
}

/** A lit key's colour and the colour drawn on it: the Enter key's, which every bundled theme sets to its accent. */
@Composable
fun keyAccent(): Pair<Color, Color> {
    val enterKey = rememberSnyggThemeQuery(
        FlorisImeUi.Key.elementName,
        attributes = mapOf(FlorisImeUi.Attr.Code to KeyCode.ENTER),
    )
    val accent = enterKey.background(default = FallbackAccent).takeIf { it.alpha > 0f } ?: FallbackAccent
    return accent to enterKey.foreground(default = Color.White)
}

/** An arc turning around the mic's edge. The angle is read while drawing, so only the ring redraws. */
@Composable
private fun WarmingRing(color: Color) {
    val turn = rememberInfiniteTransition(label = "warming").animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(tween(WARMING_TURN_MS, easing = LinearEasing)),
        label = "warming ring",
    )
    Canvas(Modifier.fillMaxSize()) {
        val stroke = WarmingStroke.toPx()
        rotate(turn.value) {
            drawArc(
                color = color,
                startAngle = 0f,
                sweepAngle = WARMING_SWEEP,
                useCenter = false,
                topLeft = Offset(stroke / 2, stroke / 2),
                size = Size(size.width - stroke, size.height - stroke),
                style = Stroke(width = stroke, cap = StrokeCap.Round),
            )
        }
    }
}
