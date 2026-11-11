package com.jtn0123.audiowhisper.dictation

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import com.jtn0123.audiowhisper.reader.VoiceEngine
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.jetpref.datastore.ui.Preference
import dev.patrickgold.jetpref.datastore.ui.PreferenceGroup
import dev.patrickgold.jetpref.datastore.ui.PreferenceUiScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.florisboard.lib.compose.stringRes

/**
 * Whether the speech model and the neural voice are on the phone and whole, and how to fix it when
 * not: both are copied in from a computer, so without this the only sign is a toast, or a reading
 * in the phone's own voice with no word why. Sizes show at once; the SHA-256 check follows (a
 * second or two, the first time only).
 */
@Composable
fun PreferenceUiScope<FlorisPreferenceModel>.ModelStatus() {
    val context = LocalContext.current
    var speech by remember { mutableStateOf(ModelFiles.quickState(context)) }
    var voice by remember { mutableStateOf(VoiceEngine.quickState(context)) }
    var checked by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) {
        withContext(Dispatchers.IO) {
            speech = ModelFiles.fullState(context)
            voice = VoiceEngine.fullState(context)
        }
        checked = true
    }
    PreferenceGroup(title = stringRes(R.string.dictation__models__title)) {
        Preference(
            title = stringRes(R.string.dictation__models__speech),
            summary = describe(context, speech, checked, speechModel = true),
        )
        Preference(
            title = stringRes(R.string.dictation__models__voice),
            summary = describe(context, voice, checked, speechModel = false),
        )
    }
}

private fun describe(context: Context, state: ModelState, checked: Boolean, speechModel: Boolean): String {
    fun text(id: Int, vararg args: Any) = context.getString(id, *args)
    return when (state) {
        ModelState.Ready -> when {
            !checked -> text(R.string.dictation__models__checking)
            speechModel -> text(R.string.dictation__models__speech_ready)
            else -> text(R.string.dictation__models__voice_ready)
        }
        is ModelState.Missing ->
            if (speechModel) text(R.string.dictation__models__speech_missing) else text(R.string.dictation__models__voice_missing)
        is ModelState.Damaged -> {
            val files = state.files.joinToString(", ")
            if (speechModel) text(R.string.dictation__models__speech_damaged, files)
            else text(R.string.dictation__models__voice_damaged, files)
        }
    }
}
