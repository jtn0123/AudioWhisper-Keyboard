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

package dev.patrickgold.florisboard.lib.ext

import kotlinx.serialization.KSerializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import com.jtn0123.audiowhisper.rust.*

@Serializable(with = ExtensionMaintainerSerializer::class)
data class ExtensionMaintainer(
    val name: String,
    val email: String? = null,
    val url: String? = null,
) {
    companion object {
        fun from(str: String): ExtensionMaintainer? {
            val ranges = ExtensionPolicy.maintainerRanges(str)
            if (ranges.isEmpty()) return null
            fun part(start: Int, end: Int): String? = if (ranges[start] < 0) null else str.substring(ranges[start], ranges[end])
            return with(ExtensionMaintainerRanges) { ExtensionMaintainer(requireNotNull(part(NAME, NAME_END)), part(EMAIL, EMAIL_END), part(URL, URL_END)) }
        }

        fun fromOrTakeRaw(str: String): ExtensionMaintainer {
            return from(str) ?: ExtensionMaintainer(str)
        }
    }

    override fun toString(): String = ExtensionPolicy.formatMaintainer(name, email, url)
}

internal class ExtensionMaintainerSerializer : KSerializer<ExtensionMaintainer> {
    override val descriptor = PrimitiveSerialDescriptor("ExtensionMaintainer", PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: ExtensionMaintainer) {
        encoder.encodeString(value.toString())
    }

    override fun deserialize(decoder: Decoder): ExtensionMaintainer {
        return ExtensionMaintainer.fromOrTakeRaw(decoder.decodeString())
    }
}
