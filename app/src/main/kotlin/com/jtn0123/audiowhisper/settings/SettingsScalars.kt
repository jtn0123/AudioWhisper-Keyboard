package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer

/** Keep Jetpref's original storage type, cache, Flow and write handler. Rust admits raw input;
 * the final primitive construction/formatting uses the platform's existing numeric ABI. */
internal fun <T : Any> nativeScalar(data: PreferenceData<T>, construct: (String)->T): PreferenceData<T> =
    object : PreferenceData<T> by data {
        override val serializer = object : PreferenceSerializer<T> {
            override fun serialize(value: T) = data.serializer.serialize(value)
            override fun deserialize(value: String): T? = SettingsRules.call(3,data.key,value).firstOrNull()?.let(construct)
        }
    }
