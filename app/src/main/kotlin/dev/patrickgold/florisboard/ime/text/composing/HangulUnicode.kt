/*
 * Copyright (C) 2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.text.composing

import com.jtn0123.audiowhisper.rust.Composing
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Korean typed jamo by jamo: an initial and a vowel join into a syllable, a consonant after it
 * becomes its final, finals and vowels that combine do, and a vowel after a final takes that
 * consonant to start the next syllable. The rules are in Rust (lib/rust/crates/composing/src/hangul.rs).
 */
@Serializable
@SerialName("hangul-unicode")
class HangulUnicode : Composer {
    override val id: String = "hangul-unicode"
    override val label: String = "Hangul Unicode"
    override val toRead: Int = 1

    override fun getActions(precedingText: String, toInsert: String): Pair<Int, String> {
        return Composing.hangul(precedingText, toInsert)
    }
}
