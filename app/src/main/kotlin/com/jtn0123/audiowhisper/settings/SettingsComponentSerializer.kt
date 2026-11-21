package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer

/** Theme preference strings; extension/asset serializers keep their existing DTO API. */
object SettingsComponentSerializer : PreferenceSerializer<ExtensionComponentName> {
    override fun serialize(value: ExtensionComponentName) = SettingsRules.call(SettingsOp.COMPONENT_ENCODE,value.extensionId,value.componentId)[0]
    override fun deserialize(value: String): ExtensionComponentName? {
        val fields=SettingsRules.call(SettingsOp.COMPONENT_DECODE,value)
        return if(fields.isEmpty()) null else ExtensionComponentName(fields[SettingsComponent.EXTENSION],fields[SettingsComponent.COMPONENT])
    }
}
