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

private const val VarKey = "varKey"

sealed interface ReferenceSnyggVarValue : ReferenceSnyggValue {
    companion object {
        val VariableNameRegex = """--[a-zA-Z0-9-]+""".toRegex()
    }
}

data class ReferenceSnyggDefinedVarValue(val key: String) : ReferenceSnyggVarValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(name = "var") { string(id = VarKey, regex = ReferenceSnyggVarValue.VariableNameRegex) }
        }

        override fun defaultValue() = ReferenceSnyggDefinedVarValue("")

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggDefinedVarValue)
            val map = referenceSnyggIdToValueMapOf(VarKey to v.key)
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val key = map.getString(VarKey)
            return@runCatching ReferenceSnyggDefinedVarValue(key)
        }
    }

    override fun encoder() = Companion
}
