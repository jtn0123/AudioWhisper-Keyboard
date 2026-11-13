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

package dev.patrickgold.florisboard.ime.keyboard

import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlinx.serialization.Serializable
import com.jtn0123.audiowhisper.rust.KeyModels

@Serializable
class CurrencySet(
    val id: String,
    val label: String,
    private val slots: List<TextKeyData>
) {
    companion object {
        val Fallback = CurrencySet(
            id = "fallback",
            label = "Fallback",
            slots = KeyModels.currencyDefaults().map { code ->
                TextKeyData(code = code, label = KeyModels.text(code, "", false, false))
            }
        )

        fun isCurrencySlot(keyCode: Int): Boolean {
            return KeyModels.isCurrency(keyCode)
        }
    }

    fun getSlot(keyCode: Int): TextKeyData? {
        val slot = KeyModels.currencySlot(keyCode)
        return slots.getOrNull(slot)
    }

    override fun toString(): String {
        return "${CurrencySet::class.simpleName} { id=$id, label\"$label\", slots=$slots }"
    }
}
