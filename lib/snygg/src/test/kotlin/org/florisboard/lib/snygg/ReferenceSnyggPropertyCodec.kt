package org.florisboard.lib.snygg

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.decodeFromJsonElement
import org.florisboard.lib.snygg.value.NativeSnyggValues
import org.florisboard.lib.snygg.value.SnyggValue

/** Original property/value loop, retained independently of the native decoder. */
internal object ReferenceSnyggPropertyCodec {
    fun decode(rule: SnyggRule, config: SnyggJsonConfiguration, json: JsonElement): SnyggPropertySet {
        return when (SnyggSpec.propertySetSpecOf(rule)!!.type) {
            SnyggSpecDecl.PropertySet.Type.SINGLE_SET -> single(rule, config, json)
            SnyggSpecDecl.PropertySet.Type.MULTIPLE_SETS -> SnyggMultiplePropertySets(
                config.json.decodeFromJsonElement<JsonArray>(json).map { single(rule, config, it) })
        }
    }
    private fun single(rule: SnyggRule, config: SnyggJsonConfiguration, json: JsonElement): SnyggSinglePropertySet {
        val jsonObject = config.json.decodeFromJsonElement<JsonObject>(json)
        val properties = mutableMapOf<String, SnyggValue>()
        propLoop@ for ((property, valueElem) in jsonObject) {
            val encoders = SnyggSpec.encodersOf(rule, property)
            if (encoders == null) {
                if (config.ignoreInvalidProperties) continue@propLoop
                throw SnyggInvalidPropertyException(rule, property)
            }
            val valueStr = config.json.decodeFromJsonElement<String>(valueElem)
            for (encoder in encoders) {
                val value = referenceValueEncoders[NativeSnyggValues.id(encoder)].deserialize(valueStr).getOrNull()
                if (value != null) {
                    properties[property] = value.production()
                    continue@propLoop
                }
            }
            if (config.ignoreInvalidValues) continue@propLoop
            throw SnyggInvalidValueException(rule, property, valueStr)
        }
        SnyggSpec.propertySetSpecOf(rule)!!.properties.forEach { (propKey, propSpec) ->
            if (propSpec.required && !properties.contains(propKey)) throw SnyggMissingRequiredPropertyException(rule, propKey)
        }
        return SnyggSinglePropertySet(properties)
    }
}
