/*
 * Copyright (C) 2024-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.media.emoji

import com.jtn0123.audiowhisper.rust.EmojiRules
import com.jtn0123.audiowhisper.rust.EmojiTones
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import java.util.stream.IntStream

enum class EmojiSkinTone(val id: Int) {
    DEFAULT(0x0),
    LIGHT_SKIN_TONE(0x1F3FB),
    MEDIUM_LIGHT_SKIN_TONE(0x1F3FC),
    MEDIUM_SKIN_TONE(0x1F3FD),
    MEDIUM_DARK_SKIN_TONE(0x1F3FE),
    DARK_SKIN_TONE(0x1F3FF);
}

enum class EmojiHairStyle(val id: Int) {
    DEFAULT(0x0),
    RED_HAIR(0x1F9B0),
    CURLY_HAIR(0x1F9B1),
    WHITE_HAIR(0x1F9B2),
    BALD(0x1F9B3);
}

/**
 * An emoji. Its [skinTone] and [hairStyle] are the first of each enum's entries whose id is one of
 * its code points, else `DEFAULT`, as Rust finds them (lib/rust/crates/emoji, `EmojiRules.tones`);
 * an emoji parsed from a file arrives with them found.
 */
class Emoji internal constructor(
    val value: String,
    val name: String,
    val keywords: List<String>,
    val skinTone: EmojiSkinTone,
    val hairStyle: EmojiHairStyle,
) : KeyData {
    constructor(value: String, name: String, keywords: List<String>) : this(value, name, keywords, EmojiRules.tones(value))

    private constructor(value: String, name: String, keywords: List<String>, tones: EmojiTones) :
        this(value, name, keywords, EmojiSkinTone.entries[tones.skinTone], EmojiHairStyle.entries[tones.hairStyle])

    override val type = KeyType.CHARACTER
    override val code = KeyCode.UNSPECIFIED
    override val label = value
    override val groupId = 0
    override val popup: PopupSet<AbstractKeyData>? = null

    val codePoints: IntStream
        get() = value.codePoints()

    override fun compute(evaluator: ComputingEvaluator): KeyData {
        return this
    }

    override fun asString(isForDisplay: Boolean): String {
        return value
    }

    override fun toString(): String {
        return "Emoji { value=$value, name=$name, keywords=$keywords }"
    }

    override fun hashCode(): Int {
        return value.hashCode()
    }

    override fun equals(other: Any?): Boolean {
        return other is Emoji && value == other.value
    }

    object ValueOnlySerializer : KSerializer<Emoji> {
        override val descriptor = PrimitiveSerialDescriptor("EmojiValueOnly", PrimitiveKind.STRING)

        override fun serialize(encoder: Encoder, value: Emoji) {
            encoder.encodeString(value.value)
        }

        override fun deserialize(decoder: Decoder): Emoji {
            return Emoji(decoder.decodeString(), "", emptyList())
        }
    }
}
