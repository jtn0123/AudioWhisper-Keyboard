/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.florisboard.lib.snygg.reference

import org.florisboard.lib.kotlin.getKeyByValue

/**
 * ReferenceSnyggValueEncoder is responsible for the representation of a [ReferenceSnyggValue] specification and for providing methods
 * for encoding and decoding a [ReferenceSnyggValue] for the serialization process.
 *
 * A ReferenceSnyggValueEncoder is typically implemented in the Companion of a ReferenceSnyggValue subclass, as this allows to use the
 * value class name in the ReferenceSnygg stylesheet spec's `supportedValues` field. This is not a requirement though and any
 * subclass / object is allowed to be used.
 */
interface ReferenceSnyggValueEncoder {
    /**
     * Describes the format of the [ReferenceSnyggValue] in the serialized state of the value in the Json file. This spec is
     * used in the serialization process of this encoder for both serializing and deserializing.
     */
    val spec: ReferenceSnyggValueSpec

    /**
     * Describes alternative formats of the [ReferenceSnyggValue] in the serialized state of the value in the Json file. These
     * specs are exclusively used in the deserialization process and ignored when serializing a value.
     */
    val alternativeSpecs: List<ReferenceSnyggValueSpec>
        get() = emptyList()

    /**
     * Returns a default value for this encoder, used for dynamic theme building in a user interface.
     */
    fun defaultValue(): ReferenceSnyggValue

    /**
     * Serialize given ReferenceSnygg value [v] and return a String representation of it. This method is exception-free, which
     * means it must **never** throw an exception. If an error during serialization occurs, it should be returned as a
     * failed result.
     */
    fun serialize(v: ReferenceSnyggValue): Result<String>

    /**
     * Deserialize given String [v] and return a ReferenceSnygg value object of it. This method is exception-free, which
     * means it must **never** throw an exception. If an error during deserialization occurs, it should be returned as
     * a failed result.
     */
    fun deserialize(v: String): Result<ReferenceSnyggValue>
}

abstract class ReferenceSnyggEnumLikeValueEncoder<V> internal constructor(
    val serializationId: String,
    val serializationMapping: Map<String, V>,
    val default: V,
    val construct: (V) -> ReferenceSnyggValue,
    val destruct: (ReferenceSnyggValue) -> V,
) : ReferenceSnyggValueEncoder {
    final override val spec = ReferenceSnyggValueSpec {
        keywords(serializationId, serializationMapping.keys.toList())
    }

    final override fun defaultValue() = construct(default)

    final override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
        val entry = destruct(v)
        val map = referenceSnyggIdToValueMapOf(
            serializationId to serializationMapping.getKeyByValue(entry)
        )
        return@runCatching spec.pack(map)
    }

    final override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
        val map = referenceSnyggIdToValueMapOf()
        spec.parse(v, map)
        val entry = serializationMapping.getOrElse(map.getString(serializationId)) {
            error("Given value \"v\" is not valid")
        }
        return@runCatching construct(entry)
    }
}
