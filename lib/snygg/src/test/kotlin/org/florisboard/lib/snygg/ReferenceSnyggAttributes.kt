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

package org.florisboard.lib.snygg

import kotlin.math.min

data class ReferenceSnyggAttributes internal constructor(
    private val attributes: Map<String, List<String>> = emptyMap(),
) : Map<String, List<String>> by attributes, Comparable<ReferenceSnyggAttributes> {
    override fun compareTo(other: ReferenceSnyggAttributes): Int {
        if (attributes.isEmpty() && other.attributes.isEmpty()) {
            return 0
        }
        val sizeDiff = attributes.size.compareTo(other.attributes.size)
        if (sizeDiff != 0) {
            return sizeDiff
        }
        // both have attributes at this point and size matches
        val attrs = attributes.entries.toList().sortedBy { it.key }
        val otherAttrs = other.attributes.entries.toList().sortedBy { it.key }
        for (attrIndex in 0..<min(attrs.size, otherAttrs.size)) {
            val attr = attrs[attrIndex]
            val otherAttr = otherAttrs[attrIndex]
            val keyDiff = attr.key.compareTo(otherAttr.key)
            if (keyDiff != 0) {
                return keyDiff
            }
            for (valueIndex in 0..<min(attr.value.size, otherAttr.value.size)) {
                val value = attr.value[valueIndex]
                val otherValue = otherAttr.value[valueIndex]
                val valueDiff = value.compareTo(otherValue)
                if (valueDiff != 0) {
                    return valueDiff
                }
            }
            val valueSizeDiff = attr.value.size.compareTo(otherAttr.value.size)
            if (valueSizeDiff != 0) {
                return valueSizeDiff
            }
        }
        return 0
    }

    /**
     * Serializes the attributes to a string.
     *
     * @return The serialized representation of the attributes.
     *
     * @since 0.5.0-alpha01
     */
    override fun toString() = buildString {
        for ((key, values) in attributes.entries.sortedBy { it.key }) {
            if (values.isEmpty()) {
                continue
            }
            append(ATTRIBUTE_OPEN)
            append(key)
            append(ATTRIBUTE_ASSIGN)
            val (ints, strings) = values.partition { value ->
                value.toIntOrNull() != null
            }
            val serializedValues = buildList {
                ints.sorted().fold(mutableListOf()) { acc, valueStr ->
                    val value = valueStr.toInt()
                    if (acc.isEmpty()) {
                        acc.add(value..value)
                    } else {
                        val lastRange = acc.last()
                        if (lastRange.last + 1 == value) {
                            acc[acc.size - 1] = lastRange.first..value
                        } else {
                            acc.add(value..value)
                        }
                    }
                    acc
                }.forEach { range ->
                    when (range.first) {
                        range.last -> {
                            add(range.first.toString())
                        }
                        range.last - 1 -> {
                            add(range.first.toString())
                            add((range.first + 1).toString())
                        }
                        else -> {
                            add(range.toString())
                        }
                    }
                }
                addAll(strings.map { string ->
                    buildString {
                        append('`')
                        append(string)
                        append('`')
                    }
                }.sorted())
            }
            append(serializedValues.joinToString(ATTRIBUTE_VALUES_SEPARATOR))
            append(ATTRIBUTE_CLOSE)
        }
    }

    /**
     * Copies the attributes, including the given key-value pairs. Duplicate key-value pairs are ignored.
     *
     * @param pairs The list of key-value pairs to include in the copy.
     * @return A new copy of the attribute mapping.
     *
     * @since 0.5.0-alpha01
     */
    fun including(vararg pairs: Pair<String, Any>): ReferenceSnyggAttributes {
        val copy = attributes.toMutableMap()
        pairs.forEach { (key, anyValue) ->
            val value = anyValue.toString()
            copy[key] = buildSet {
                addAll(copy[key].orEmpty())
                add(value)
            }.toList().sorted()
        }
        return ReferenceSnyggAttributes(copy.toMap())
    }

    /**
     * Copies the attributes, excluding the given key-value pairs. Ignores non-present pairs.
     *
     * @param pairs The list of key-value pairs to exclude in the copy.
     * @return A new copy of the attribute mapping.
     *
     * @since 0.5.0-alpha01
     */
    fun excluding(vararg pairs: Pair<String, Any>): ReferenceSnyggAttributes {
        val copy = attributes.toMutableMap()
        pairs.forEach { (key, anyValue) ->
            val value = anyValue.toString()
            val list = buildSet {
                addAll(copy[key].orEmpty())
                remove(value)
            }.toList().sorted()
            if (list.isNotEmpty()) {
                copy[key] = list
            } else {
                copy.remove(key)
            }
        }
        return ReferenceSnyggAttributes(copy.toMap())
    }

    /**
     * Copies the attributes, toggling the given key-value pairs. Existing key-value pairs will be removed,
     * non-existing will be added.
     *
     * @param pairs The list of key-value pairs to toggle in the copy.
     * @return A new copy of the attribute mapping.
     *
     * @since 0.5.0-alpha01
     */
    fun toggling(vararg pairs: Pair<String, Any>): ReferenceSnyggAttributes {
        val copy = attributes.toMutableMap()
        pairs.forEach { (key, anyValue) ->
            val value = anyValue.toString()
            val list = buildSet {
                addAll(copy[key].orEmpty())
                if (contains(value)) {
                    remove(value)
                } else {
                    add(value)
                }
            }.toList().sorted()
            if (list.isNotEmpty()) {
                copy[key] = list
            } else {
                copy.remove(key)
            }
        }
        return ReferenceSnyggAttributes(copy.toMap())
    }

    companion object {
        private const val ATTRIBUTE_OPEN = "["
        private const val ATTRIBUTE_CLOSE = "]"
        private const val ATTRIBUTE_ASSIGN = "="
        private const val ATTRIBUTE_VALUES_SEPARATOR = ","

        private val INT_PATTERN = """(?:0|-?[1-9][0-9]*)""".toRegex()
        private val INT_RANGE_PATTERN = """$INT_PATTERN[.]{2}$INT_PATTERN""".toRegex()
        private val STRING_PATTERN = """`[^`]+`""".toRegex()
        private val ATTR_VALUE_PATTERN = """(?:$STRING_PATTERN|$INT_RANGE_PATTERN|$INT_PATTERN)""".toRegex()
        internal val ATTRIBUTE_REGEX = """\[(?<attrKey>[a-zA-Z0-9-]+)=(?<attrRawValues>$ATTR_VALUE_PATTERN(?:,$ATTR_VALUE_PATTERN)*)]""".toRegex()

        internal fun from(str: String): ReferenceSnyggAttributes {
            val attributes = mutableMapOf<String, List<String>>()
            for (attrMatch in ATTRIBUTE_REGEX.findAll(str)) {
                val key = attrMatch.groups["attrKey"]!!.value
                val rawValues = attrMatch.groups["attrRawValues"]!!.value
                attributes[key] = buildList {
                    attributes[key]?.let { addAll(it) }
                    for (attrValueMatch in ATTR_VALUE_PATTERN.findAll(rawValues)) {
                        val rawValue = attrValueMatch.value
                        if (STRING_PATTERN.matches(rawValue)) {
                            add(rawValue.substring(1, rawValue.length - 1))
                            continue
                        }
                        if (INT_RANGE_PATTERN.matches(rawValue)) {
                            val (start, end) = rawValue.split("..").map { it.toInt() }
                            addAll((start..end).map { it.toString() })
                            continue
                        }
                        if (INT_PATTERN.matches(rawValue)) {
                            add(rawValue)
                            continue
                        }
                    }
                    sort()
                }.distinct()
            }
            return ReferenceSnyggAttributes(attributes.toMap())
        }

        internal fun of(vararg pairs: Pair<String, List<Any>>) = ReferenceSnyggAttributes(
            mapOf(
                *pairs.map { (key, values) -> key to values.map { it.toString() }.distinct() }.toTypedArray()
            )
        )
    }
}
