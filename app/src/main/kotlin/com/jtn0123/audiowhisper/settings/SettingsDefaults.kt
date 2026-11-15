package com.jtn0123.audiowhisper.settings

import androidx.compose.ui.graphics.Color
import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHistory
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.jetpref.datastore.model.LocalTime
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.datastore.model.PreferenceModel
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer

object SettingsDefaults {
    fun boolean(key: String) = SettingsRules.default(key).toBooleanStrict()
    fun int(key: String) = SettingsRules.default(key).toInt()
    fun long(key: String) = SettingsRules.default(key).toLong()
    fun float(key: String) = SettingsRules.default(key).toFloat()
    fun string(key: String) = SettingsRules.default(key)
    inline fun <reified E : Enum<E>> enum(key: String): E = enumValues<E>()[SettingsRules.enumIndex(key, string(key))]
    fun component(key: String) = ExtensionComponentName.from(string(key))
    fun color(key: String) = Color(SettingsRules.color(string(key)).toULong(16))
    fun time(key: String) = checkNotNull(SettingsTimeSerializer.deserialize(string(key)))
    fun history(key: String): EmojiHistory = SettingsCodecs.history(string(key))
    fun arrangement(key: String): QuickActionArrangement = SettingsCodecs.arrangement(string(key))
}

/** Enum parsing policy is native. Enum construction and the Jetpref type binding stay here. */
class SettingsEnumSerializer<E : Enum<E>>(private val key: String, private val entries: List<E>) : PreferenceSerializer<E> {
    override fun serialize(value: E) = value.toString()
    override fun deserialize(value: String): E? = entries.getOrNull(SettingsRules.enumIndex(key, value))
}

abstract class SettingsPreferenceModel : PreferenceModel() {
    protected fun nativeBoolean(key: String) = nativeScalar(boolean(key,SettingsDefaults.boolean(key)),String::toBooleanStrict)
    protected fun nativeInt(key: String) = nativeScalar(int(key,SettingsDefaults.int(key)),String::toInt)
    protected fun nativeLong(key: String) = nativeScalar(long(key,SettingsDefaults.long(key)),String::toLong)
    protected fun nativeFloat(key: String) = nativeScalar(float(key,SettingsDefaults.float(key))) { Float.fromBits(it.toInt()) }
    protected fun nativeString(key: String) = nativeScalar(string(key,SettingsDefaults.string(key))) { it }

    // Typed PreferenceData, not SettingsPreference: Jetpref's KSP processor declares only
    // properties of that type, and the store neither loads nor persists the rest.
    protected inline fun <reified E : Enum<E>> nativeEnum(key: String): PreferenceData<E> = SettingsPreference(custom(
        key, SettingsDefaults.enum<E>(key), SettingsEnumSerializer(key, enumValues<E>().toList()),
    ))
    /** Jetpref's `custom`, with writes that finish (see [SettingsPreference]). */
    protected fun <V : Any> customPreference(key: String, default: V, serializer: PreferenceSerializer<V>): PreferenceData<V> =
        SettingsPreference(custom(key, default, serializer))
}
