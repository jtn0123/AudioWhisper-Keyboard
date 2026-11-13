package dev.patrickgold.florisboard

import android.inputmethodservice.ExtractEditText
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import com.jtn0123.audiowhisper.render.RustExtractView
import com.jtn0123.audiowhisper.render.rendererTheme
import dev.patrickgold.florisboard.ime.theme.FlorisImeTheme

@Composable
internal fun NativeExtractContent(service: FlorisImeService, editor: ExtractEditText) {
    FlorisImeTheme {
        val theme = rendererTheme()
        val info by service.editorInstance.activeInfoFlow.collectAsState()
        val label = (info.extractedActionLabel ?: service.getTextForImeAction(info.imeOptions.action.toInt()) ?: "ACTION")
        AndroidView(modifier = Modifier.fillMaxSize(), factory = { RustExtractView(it, editor) }, update = { view ->
            view.update(theme, label) {
                if (info.extractedActionId != 0) service.currentInputConnection?.performEditorAction(info.extractedActionId)
                else service.editorInstance.performEnterAction(info.imeOptions.action)
            }
        })
    }
}
