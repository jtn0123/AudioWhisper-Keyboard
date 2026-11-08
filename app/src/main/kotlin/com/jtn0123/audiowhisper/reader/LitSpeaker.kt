package com.jtn0123.audiowhisper.reader

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.unit.dp
import com.jtn0123.audiowhisper.dictation.keyAccent
import dev.patrickgold.florisboard.R
import org.florisboard.lib.compose.stringRes

private const val DISC = 0.85f

/** The speaker button while it reads: a disc in the theme's accent with a stop square. Tapping it stops. */
@Composable
fun LitSpeaker(modifier: Modifier = Modifier) {
    val (accent, iconTint) = keyAccent()
    Box(
        modifier = modifier
            .fillMaxSize()
            .padding(2.dp)
            .drawBehind { drawCircle(accent, radius = size.minDimension / 2 * DISC) },
        contentAlignment = Alignment.Center,
    ) {
        Icon(imageVector = Icons.Default.Stop, contentDescription = stringRes(R.string.read_aloud__stop), tint = iconTint)
    }
}
