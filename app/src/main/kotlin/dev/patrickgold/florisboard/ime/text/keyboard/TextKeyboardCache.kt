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

package dev.patrickgold.florisboard.ime.text.keyboard

import com.jtn0123.audiowhisper.rust.LayoutCache
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import kotlinx.coroutines.runBlocking
import kotlin.contracts.InvocationKind
import kotlin.contracts.contract

/** Kotlin holds keyboard objects; Rust owns the per-mode subtype-hash keys and invalidation. */
class TextKeyboardCache {
    private val cache = LayoutCache<TextKeyboard>()

    fun clear() = cache.clear()
    fun clear(mode: KeyboardMode) = cache.clearMode(mode.value)
    fun clear(subtype: Subtype) = cache.clearSubtype(subtype.hashCode())
    fun clear(mode: KeyboardMode, subtype: Subtype) = cache.clear(mode.value, subtype.hashCode())
    fun getAsync(mode: KeyboardMode, subtype: Subtype): TextKeyboard? = cache.get(mode.value, subtype.hashCode())

    fun getOrElseAsync(mode: KeyboardMode, subtype: Subtype, block: suspend () -> TextKeyboard): TextKeyboard {
        contract { callsInPlace(block, InvocationKind.AT_MOST_ONCE) }
        return getAsync(mode, subtype) ?: runBlocking { block() }.also { set(mode, subtype, it) }
    }

    fun set(mode: KeyboardMode, subtype: Subtype, keyboard: TextKeyboard) = cache.set(mode.value, subtype.hashCode(), keyboard)
}
