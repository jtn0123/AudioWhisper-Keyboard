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

package org.florisboard.lib.snygg

import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.encodeToJsonElement

data class ReferenceSnyggStylesheet internal constructor(
    val schema: String,
    val rules: Map<ReferenceSnyggRule, SnyggPropertySet>,
) {
    fun toJson(
        config: SnyggJsonConfiguration = SnyggJsonConfiguration.DEFAULT,
    ): Result<String> = runCatching {
        val schemaElem = config.json.encodeToJsonElement(schema)
        val ruleElems = rules.map { (rule, propertySet) ->
            rule.toString() to propertySet.toJsonElement(rule.production(), config)
        }.toMap()
        val jsonObject = JsonObject(
            mapOf(
                "\$schema" to schemaElem
            ) + ruleElems
        )
        config.json.encodeToString(jsonObject)
    }

    companion object {
        const val SCHEMA_V2 = "https://schemas.florisboard.org/snygg/v2/stylesheet"

        private val SCHEMA_PATTERN = """^https://schemas.florisboard.org/snygg/v[0-9]+/stylesheet$""".toRegex()

        fun fromJson(
            json: String,
            config: SnyggJsonConfiguration = SnyggJsonConfiguration.DEFAULT,
        ): Result<ReferenceSnyggStylesheet> = runCatching {
            val jsonObject = config.json.decodeFromString<JsonObject>(json)
            var schema: String? = jsonObject["\$schema"]?.let { config.json.decodeFromJsonElement(it) }
            if (schema == null) {
                if (config.ignoreMissingSchema) {
                    // assume schema
                    schema = SCHEMA_V2
                } else {
                    throw ReferenceSnyggMissingSchemaException()
                }
            } else {
                val schemaMatch = SCHEMA_PATTERN.matchEntire(schema)
                if (schemaMatch == null) {
                    if (config.ignoreInvalidSchema) {
                        // assume schema
                        schema = SCHEMA_V2
                    } else {
                        throw ReferenceSnyggInvalidSchemaException(schema)
                    }
                }
                if (schema != SCHEMA_V2 && !config.ignoreUnsupportedSchema) {
                    throw ReferenceSnyggUnsupportedSchemaException(schema)
                }
            }
            val ruleMap = mutableMapOf<ReferenceSnyggRule, SnyggPropertySet>()
            for ((key, jsonElement) in jsonObject.entries) {
                if (key == "\$schema") {
                    continue
                } else {
                    val rule = ReferenceSnyggRule.fromOrNull(key)
                    if (rule == null) {
                        if (config.ignoreInvalidRules) {
                            continue
                        }
                        throw ReferenceSnyggInvalidRuleException(key)
                    }
                    val propertySet = SnyggPropertySet.fromJsonElement(rule.production(), config, jsonElement)
                    ruleMap[rule] = propertySet
                }
            }
            ReferenceSnyggStylesheet(schema, ruleMap)
        }
    }
}


class ReferenceSnyggMissingSchemaException : SerializationException("Stylesheet is missing schema reference")
class ReferenceSnyggInvalidSchemaException(schema: String) : SerializationException("Stylesheet references invalid schema '$schema'")
class ReferenceSnyggUnsupportedSchemaException(schema: String) : SerializationException("Stylesheet references unsupported schema '$schema'")
class ReferenceSnyggInvalidRuleException(rule: String) : SerializationException("Invalid rule '$rule'")
