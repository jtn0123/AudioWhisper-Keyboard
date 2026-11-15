package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.withContext

/** Keep Jetpref's original storage type, cache, Flow and write handler. Rust admits raw input;
 * the final primitive construction/formatting uses the platform's existing numeric ABI. */
internal fun <T : Any> nativeScalar(data: PreferenceData<T>, construct: (String)->T): PreferenceData<T> =
    SettingsPreference(data, object : PreferenceSerializer<T> {
        override fun serialize(value: T) = data.serializer.serialize(value)
        override fun deserialize(value: String): T? = SettingsRules.call(3,data.key,value).firstOrNull()?.let(construct)
    })

/**
 * Every preference the model declares: Jetpref's, except that a write, once started, finishes
 * even when its caller is cancelled. Jetpref's DataStore answers each write on a channel the
 * waiting caller consumes; a caller cancelled while waiting cancels that channel, the store's one
 * persist loop then throws on its answer and stops, and from then on every write in the process
 * fails and nothing reaches the file. Writes are cancelled routinely: NlpManager sets preferences
 * inside collectLatest, and a screen's scope ends when it closes. The caller still sees its own
 * cancellation once the write is done.
 */
class SettingsPreference<T : Any> @PublishedApi internal constructor(
    private val data: PreferenceData<T>,
    override val serializer: PreferenceSerializer<T> = data.serializer,
) : PreferenceData<T> by data {
    override suspend fun set(value: T): Result<Unit> = withContext(NonCancellable) { data.set(value) }
    override suspend fun reset(): Result<Unit> = withContext(NonCancellable) { data.reset() }
}
