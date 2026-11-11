package com.jtn0123.audiowhisper.dictation

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.lib.compose.FlorisScreen
import dev.patrickgold.jetpref.datastore.ui.PreferenceGroup
import dev.patrickgold.jetpref.datastore.ui.SwitchPreference
import kotlinx.coroutines.launch
import org.florisboard.lib.compose.stringRes

/**
 * Whether the models are on the phone ([ModelStatus]), and how dictated phrases are tidied before
 * they are typed, by the Mac app's rules: filler words and the user's vocabulary (see
 * `DictationRules.format`).
 */
@Composable
fun DictationScreen() = FlorisScreen {
    title = stringRes(R.string.settings__dictation__title)
    previewFieldVisible = true

    content {
        ModelStatus()
        SwitchPreference(
            prefs.dictation.removeFillers,
            title = stringRes(R.string.pref__dictation__remove_fillers__label),
            summary = stringRes(R.string.pref__dictation__remove_fillers__summary),
        )
        PreferenceGroup(title = stringRes(R.string.pref__dictation__vocabulary__label)) {
            val saved = prefs.dictation.vocabulary
            var text by remember { mutableStateOf(saved.get()) }
            val scope = rememberCoroutineScope()
            Text(
                text = stringRes(R.string.pref__dictation__vocabulary__summary),
                style = MaterialTheme.typography.bodyMedium,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            )
            OutlinedTextField(
                value = text,
                onValueChange = {
                    text = it
                    scope.launch { saved.set(it) }
                },
                placeholder = { Text(stringRes(R.string.pref__dictation__vocabulary__placeholder)) },
                minLines = 4,
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(min = 120.dp)
                    .padding(horizontal = 16.dp, vertical = 8.dp),
            )
        }
    }
}
