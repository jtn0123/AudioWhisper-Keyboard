package com.jtn0123.audiowhisper.dictation

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import org.florisboard.lib.snygg.ui.SnyggRow
import org.florisboard.lib.snygg.ui.SnyggText

/**
 * In place of the suggestions while a phrase is spoken: its words so far, on one line, cut
 * from the start ("…some milk because") so the latest words always show.
 */
@Composable
fun DictationPreview(text: String, modifier: Modifier = Modifier) {
    SnyggRow(
        elementName = FlorisImeUi.SmartbarCandidatesRow.elementName,
        modifier = modifier.fillMaxSize(),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SnyggText(
            elementName = "${FlorisImeUi.SmartbarCandidateWord.elementName}-text",
            text = text,
            maxLines = 1,
            overflow = TextOverflow.StartEllipsis,
        )
    }
}
