package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.jetpref.datastore.model.LocalTime
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer

object SettingsTimeSerializer : PreferenceSerializer<LocalTime> {
    override fun deserialize(value: String): LocalTime? {
        val fields=SettingsRules.call(SettingsOp.TIME_DECODE,value).map(String::toInt)
        return if(fields.isEmpty()) null else with(SettingsTime) { LocalTime(fields[HOUR],fields[MINUTE],fields[SECOND],fields[MILLISECOND]) }
    }
    override fun serialize(value: LocalTime) = SettingsRules.call(SettingsOp.TIME_ENCODE, *with(SettingsTime) { row(HOUR to "${value.hour}",
        MINUTE to "${value.minute}", SECOND to "${value.second}", MILLISECOND to "${value.millisecond}") })[0]
}
