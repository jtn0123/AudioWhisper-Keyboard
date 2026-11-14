package dev.patrickgold.florisboard.app

import androidx.compose.ui.graphics.Color
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer

internal object ReferenceColorSerializer : PreferenceSerializer<Color> {
    @OptIn(ExperimentalStdlibApi::class)
    override fun deserialize(value: String) = Color(value.hexToULong())
    @OptIn(ExperimentalStdlibApi::class)
    override fun serialize(value: Color) = value.value.toHexString()
}
