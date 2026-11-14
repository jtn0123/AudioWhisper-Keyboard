package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer

/** Theme preference strings; extension/asset serializers keep their existing DTO API. */
object SettingsComponentSerializer : PreferenceSerializer<ExtensionComponentName> {
    override fun serialize(value: ExtensionComponentName) = SettingsRules.call(31,value.extensionId,value.componentId)[0]
    override fun deserialize(value: String): ExtensionComponentName? {
        val fields=SettingsRules.call(32,value)
        return if(fields.isEmpty()) null else ExtensionComponentName(fields[0],fields[1])
    }
}
