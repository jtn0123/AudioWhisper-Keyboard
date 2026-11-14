package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.jetpref.datastore.model.LocalTime
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer

object SettingsTimeSerializer : PreferenceSerializer<LocalTime> {
    override fun deserialize(value: String): LocalTime? {
        val fields=SettingsRules.call(26,value).map(String::toInt)
        return if(fields.isEmpty()) null else LocalTime(fields[0],fields[1],fields[2],fields[3])
    }
    override fun serialize(value: LocalTime) = SettingsRules.call(27,value.hour.toString(),value.minute.toString(),
        value.second.toString(),value.millisecond.toString())[0]
}
