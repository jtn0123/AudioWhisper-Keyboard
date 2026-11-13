package org.florisboard.lib.snygg

import com.jtn0123.audiowhisper.rust.SnyggValues
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.decodeFromJsonElement
import org.florisboard.lib.snygg.value.NativeSnyggValues
import org.florisboard.lib.snygg.value.SnyggValue

/** JSON syntax stays managed. Rust chooses encoders and the first property/value error. */
internal object SnyggPropertyCodec {
    private fun required(rule: SnyggRule): List<String> = SnyggSpec.propertySetSpecOf(rule)!!.properties
        .filterValues { it.required }.keys.toList()

    fun missing(rule: SnyggRule, names: Set<String>): String? = SnyggValues.call(7,
        (listOf(names.size.toString()) + names + required(rule)).toTypedArray()).firstOrNull()

    fun decode(rule: SnyggRule, config: SnyggJsonConfiguration, json: JsonElement): Map<String, SnyggValue> {
        val entries = config.json.decodeFromJsonElement<JsonObject>(json).entries.toList()
        // A decoding failure is recorded, not thrown yet: an earlier unknown property
        // must win, and ignored unknown properties must never expose their JSON error.
        val strings = entries.map { runCatching { config.json.decodeFromJsonElement<String>(it.value) } }
        val required = required(rule)
        val flags = (if (config.ignoreInvalidProperties) 1 else 0) or (if (config.ignoreInvalidValues) 2 else 0)
        val packet = buildList {
            add(flags.toString()); add(required.size.toString()); addAll(required); add(entries.size.toString())
            for ((index, entry) in entries.withIndex()) {
                val encoders = SnyggSpec.encodersOf(rule, entry.key)
                add(entry.key); add(if (encoders == null) "0" else "1"); add((encoders?.size ?: 0).toString())
                encoders?.forEach { add(NativeSnyggValues.id(it).toString()) }
                add(if (strings[index].isSuccess) "1" else "0"); add(strings[index].getOrDefault(""))
            }
        }
        val rows = SnyggValues.call(6, packet.toTypedArray()).toList()
        if (rows.firstOrNull() == "!") {
            if (rows[1] == "missing") throw SnyggMissingRequiredPropertyException(rule, rows[2])
            val index = rows[2].toInt()
            throw when (rows[1]) {
                "property" -> SnyggInvalidPropertyException(rule, entries[index].key)
                "value" -> SnyggInvalidValueException(rule, entries[index].key, strings[index].getOrThrow())
                "json" -> strings[index].exceptionOrNull()!!
                else -> SerializationException("Invalid native property error")
            }
        }
        val iterator = rows.iterator()
        return buildMap {
            repeat(iterator.next().toInt()) {
                val index = iterator.next().toInt()
                val record = List(iterator.next().toInt()) { iterator.next() }
                put(entries[index].key, NativeSnyggValues.value(record))
            }
        }
    }
}
