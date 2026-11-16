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

package dev.patrickgold.florisboard.app.settings.typing

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.LibraryBooks
import androidx.compose.material.icons.filled.Contacts
import androidx.compose.material.icons.filled.Language
import androidx.compose.material.icons.filled.SpaceBar
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.LocalNavController
import dev.patrickgold.florisboard.app.Routes
import dev.patrickgold.florisboard.app.enumDisplayEntriesOf
import dev.patrickgold.florisboard.ime.keyboard.IncognitoMode
import dev.patrickgold.florisboard.ime.nlp.SpellingLanguageMode
import dev.patrickgold.florisboard.lib.compose.FlorisHyperlinkText
import dev.patrickgold.florisboard.lib.compose.FlorisScreen
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.jetpref.datastore.model.observeAsState
import dev.patrickgold.jetpref.datastore.ui.ExperimentalJetPrefDatastoreUi
import dev.patrickgold.jetpref.datastore.ui.ListPreference
import dev.patrickgold.jetpref.datastore.ui.Preference
import dev.patrickgold.jetpref.datastore.ui.PreferenceGroup
import dev.patrickgold.jetpref.datastore.ui.SwitchPreference
import dev.patrickgold.jetpref.material.ui.JetPrefAlertDialog
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showShortToast
import org.florisboard.lib.compose.stringRes

@OptIn(ExperimentalJetPrefDatastoreUi::class)
@Composable
fun TypingScreen() = FlorisScreen {
    title = stringRes(R.string.settings__typing__title)
    previewFieldVisible = true

    val navController = LocalNavController.current
    var confirmForgetLearned by remember { mutableStateOf(false) }

    content {
        PreferenceGroup(title = stringRes(R.string.pref__suggestion__title)) {
            SwitchPreference(
                prefs.suggestion.enabled,
                title = stringRes(R.string.pref__suggestion__enabled__label),
                summary = stringRes(R.string.pref__suggestion__enabled__summary),
            )
            SwitchPreference(
                prefs.suggestion.blockPossiblyOffensive,
                title = stringRes(R.string.pref__suggestion__block_possibly_offensive__label),
                summary = stringRes(R.string.pref__suggestion__block_possibly_offensive__summary),
                enabledIf = { prefs.suggestion.enabled isEqualTo true },
            )
            SwitchPreference(
                prefs.suggestion.api30InlineSuggestionsEnabled,
                title = stringRes(R.string.pref__suggestion__api30_inline_suggestions_enabled__label),
                summary = stringRes(R.string.pref__suggestion__api30_inline_suggestions_enabled__summary),
            )
            ListPreference(
                prefs.suggestion.incognitoMode,
                icon = ImageVector.vectorResource(id = R.drawable.ic_incognito),
                title = stringRes(R.string.pref__suggestion__incognito_mode__label),
                entries = enumDisplayEntriesOf(IncognitoMode::class),
            )
            // AudioWhisper: learning from typing.
            SwitchPreference(
                prefs.suggestion.learnFromTyping,
                title = stringRes(R.string.pref__suggestion__learn_from_typing__label),
                summary = stringRes(R.string.pref__suggestion__learn_from_typing__summary),
                enabledIf = { prefs.suggestion.enabled isEqualTo true },
            )
            // AudioWhisper: where this user's taps land in each key (suggest/tap_aim.rs).
            SwitchPreference(
                prefs.suggestion.learnTapAim,
                title = stringRes(R.string.pref__suggestion__learn_tap_aim__label),
                summary = stringRes(R.string.pref__suggestion__learn_tap_aim__summary),
                enabledIf = { prefs.suggestion.enabled isEqualTo true },
            )
            Preference(
                title = stringRes(R.string.pref__suggestion__forget_learned__label),
                summary = stringRes(R.string.pref__suggestion__forget_learned__summary),
                onClick = { confirmForgetLearned = true },
            )
        }
        if (confirmForgetLearned) {
            ForgetLearnedWordsDialog(onDismiss = { confirmForgetLearned = false })
        }

        PreferenceGroup(title = stringRes(R.string.pref__correction__title)) {
            SwitchPreference(
                prefs.correction.autoCapitalization,
                title = stringRes(R.string.pref__correction__auto_capitalization__label),
                summary = stringRes(R.string.pref__correction__auto_capitalization__summary),
            )
            val isAutoSpacePunctuationEnabled by prefs.correction.autoSpacePunctuation.observeAsState()
            SwitchPreference(
                prefs.correction.autoSpacePunctuation,
                icon = Icons.Default.SpaceBar,
                title = stringRes(R.string.pref__correction__auto_space_punctuation__label),
                summary = stringRes(R.string.pref__correction__auto_space_punctuation__summary),
            )
            if (isAutoSpacePunctuationEnabled) {
                Card(modifier = Modifier.padding(8.dp)) {
                    Column(modifier = Modifier.padding(8.dp)) {
                        Text(text = stringRes(R.string.pref__correction__auto_space_punctuation__feedback))
                        FlorisHyperlinkText(
                            text = stringRes(R.string.pref__correction__auto_space_punctuation__feedback_link),
                            url = stringRes(R.string.florisboard__auto_space_feedback_url),
                        )
                    }
                }
            }
            SwitchPreference(
                prefs.correction.rememberCapsLockState,
                title = stringRes(R.string.pref__correction__remember_caps_lock_state__label),
                summary = stringRes(R.string.pref__correction__remember_caps_lock_state__summary),
            )
            SwitchPreference(
                prefs.correction.doubleSpacePeriod,
                title = stringRes(R.string.pref__correction__double_space_period__label),
                summary = stringRes(R.string.pref__correction__double_space_period__summary),
            )
        }

        PreferenceGroup(title = stringRes(R.string.pref__spelling__title)) {
            val florisSpellCheckerEnabled = remember { mutableStateOf(false) }
            SpellCheckerServiceSelector(florisSpellCheckerEnabled)
            ListPreference(
                prefs.spelling.languageMode,
                icon = Icons.Default.Language,
                title = stringRes(R.string.pref__spelling__language_mode__label),
                entries = enumDisplayEntriesOf(SpellingLanguageMode::class),
                enabledIf = { florisSpellCheckerEnabled.value },
            )
            SwitchPreference(
                prefs.spelling.useContacts,
                icon = Icons.Default.Contacts,
                title = stringRes(R.string.pref__spelling__use_contacts__label),
                summary = stringRes(R.string.pref__spelling__use_contacts__summary),
                enabledIf = { florisSpellCheckerEnabled.value },
                visibleIf = { false }, // For now
            )
            SwitchPreference(
                prefs.spelling.useUdmEntries,
                icon = Icons.AutoMirrored.Filled.LibraryBooks,
                title = stringRes(R.string.pref__spelling__use_udm_entries__label),
                summary = stringRes(R.string.pref__spelling__use_udm_entries__summary),
                enabledIf = { florisSpellCheckerEnabled.value },
                visibleIf = { false }, // For now
            )
        }

        PreferenceGroup(title = stringRes(R.string.settings__dictionary__title)) {
            Preference(
                icon = Icons.AutoMirrored.Filled.LibraryBooks,
                title = stringRes(R.string.settings__dictionary__title),
                onClick = { navController.navigate(Routes.Settings.Dictionary) },
            )
        }
    }
}

/** AudioWhisper: asks before erasing what the keyboard learned from typing, then erases it. */
@Composable
private fun ForgetLearnedWordsDialog(onDismiss: () -> Unit) {
    val context = LocalContext.current
    val nlpManager by context.nlpManager()
    val scope = rememberCoroutineScope()
    JetPrefAlertDialog(
        title = stringRes(R.string.pref__suggestion__forget_learned__label),
        confirmLabel = stringRes(R.string.action__delete),
        dismissLabel = stringRes(R.string.action__cancel),
        onDismiss = onDismiss,
        onConfirm = {
            nlpManager.forgetLearnedWords()
            onDismiss()
            scope.launch { context.showShortToast(R.string.pref__suggestion__forget_learned__done) }
        },
    ) {
        Text(stringRes(R.string.pref__suggestion__forget_learned__confirm))
    }
}
