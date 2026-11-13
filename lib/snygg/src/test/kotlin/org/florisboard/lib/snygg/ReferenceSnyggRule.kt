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

/**
 * Base interface for all Snygg stylesheet rules. A rule in a stylesheet is a core component, acting as the key for a
 * property set map. There are two main rule categories, annotation and element rules.
 *
 * - **Annotation rules**: Represent meta-rules, describing globally valid properties. This includes global style
 *   variables, global font faces, etc.
 * - **Element rules**: These rules target a specific element with a specific attribute/selector set. Used to describe
 *   the style of a specific element.
 *
 * @since 0.5.0-alpha01
 * @see [ReferenceSnyggAnnotationRule.Defines]
 * @see [ReferenceSnyggAnnotationRule.Font]
 * @see [ReferenceSnyggElementRule]
 */
sealed interface ReferenceSnyggRule : Comparable<ReferenceSnyggRule> {
    /**
     * Returns the associated declaration of this rule.
     *
     * @since 0.5.0-alpha01
     */
    fun decl(): SnyggSpecDecl.RuleDecl

    /**
     * Compares this Snygg rule with [other]. The ordering is defined as follows:
     * - @defines
     * - @font (multiple fonts => sort by fontName)
     * - elements (first by element name, then by attributes/selectors)
     *
     * @since 0.5.0-alpha01
     */
    override fun compareTo(other: ReferenceSnyggRule): Int

    /**
     * Serializes the Snygg rule to a string. This method never fails.
     *
     * @return The serialized representation of the Snygg rule instance.
     * @since 0.5.0-alpha01
     */
    override fun toString(): String

    companion object {
        /**
         * Attempts to parse the given string into a `ReferenceSnyggRule` instance or `null` if no match is found.
         *
         * @param str The string to parse into a `ReferenceSnyggRule`.
         * @return A `ReferenceSnyggRule` instance if the string matches any supported rule type, or `null` if no match is found.
         * @since 0.5.0-alpha01
         */
        fun fromOrNull(str: String): ReferenceSnyggRule? {
            return ReferenceSnyggAnnotationRule.Defines.fromOrNull(str)
                ?: ReferenceSnyggAnnotationRule.Font.fromOrNull(str)
                ?: ReferenceSnyggElementRule.fromOrNull(str)
        }
    }
}

/**
 * Annotation rule base interface. See the specific implementations for details.
 *
 * @since 0.5.0-alpha01
 *
 * @see [ReferenceSnyggAnnotationRule.Defines]
 * @see [ReferenceSnyggAnnotationRule.Font]
 */
sealed interface ReferenceSnyggAnnotationRule : ReferenceSnyggRule {
    data object Defines : ReferenceSnyggAnnotationRule, SnyggSpecDecl.RuleDecl {
        override val name: String = "defines"

        override val pattern = """^@$name$""".toRegex()

        override fun decl() = this

        override fun compareTo(other: ReferenceSnyggRule): Int {
            return when (other) {
                is Defines -> 0 // same
                is ReferenceSnyggAnnotationRule -> decl().name.compareTo(other.decl().name)
                is ReferenceSnyggElementRule -> -1 // annotations always come first
            }
        }

        override fun toString(): String {
            return "@defines"
        }

        /**
         * Attempts to parse the given string into a `defines` annotation rule instance, or `null` if the given string
         * does not represent a `defines` annotation rule.
         *
         * @param str The string to parse into a `defines` annotation rule instance.
         * @return A `defines` annotation rule instance or `null`.
         *
         * @since 0.5.0-alpha01
         */
        fun fromOrNull(str: String): Defines? {
            pattern.matchEntire(str) ?: return null
            return Defines
        }
    }

    data class Font(val fontName: String) : ReferenceSnyggAnnotationRule {
        override fun decl() = Companion

        override fun compareTo(other: ReferenceSnyggRule): Int {
            return when (other) {
                is Font -> fontName.compareTo(other.fontName)
                is ReferenceSnyggAnnotationRule -> decl().name.compareTo(other.decl().name)
                is ReferenceSnyggElementRule -> -1 // annotations always come first
            }
        }

        override fun toString(): String {
            return "@font `$fontName`"
        }

        companion object : SnyggSpecDecl.RuleDecl {
            override val name = "font"
            override val pattern = """^@$name `(?<fontName>[a-zA-Z0-9\s-]+)`$""".toRegex()

            /**
             * Attempts to parse the given string into a `font` annotation rule instance, or `null` if the given string
             * does not represent a `font` annotation rule.
             *
             * @param str The string to parse into a `font` annotation rule instance.
             * @return A `font` annotation rule instance or `null`.
             *
             * @since 0.5.0-alpha01
             */
            fun fromOrNull(str: String): Font? {
                val match = pattern.matchEntire(str) ?: return null
                return Font(match.groups["fontName"]!!.value)
            }
        }
    }
}

/**
 * A core element in the Snygg styling system, this rule allows targeting specific elements with specific attributes
 * and selectors.
 *
 * @property elementName The element name this rule targets, it can be seen similarly to a CSS class.
 * @property attributes The attributes this rule targets.
 * @property selector The selector this rule targets, or [ReferenceSnyggSelector.NONE] for not specified.
 *
 * @since 0.5.0-alpha01
 */
data class ReferenceSnyggElementRule(
    val elementName: String,
    val attributes: ReferenceSnyggAttributes = ReferenceSnyggAttributes(),
    val selector: ReferenceSnyggSelector = ReferenceSnyggSelector.NONE,
) : ReferenceSnyggRule {
    init {
        requireNotNull(ELEMENT_NAME_REGEX.matchEntire(elementName)) { "element name is invalid" }
    }

    override fun decl() = Companion

    override fun compareTo(other: ReferenceSnyggRule): Int {
        if (other !is ReferenceSnyggElementRule) {
            return 1 // annotations always come first
        }
        val elemDiff = elementName.compareTo(other.elementName)
        if (elemDiff != 0) {
            return elemDiff
        }
        if (selector != ReferenceSnyggSelector.NONE || other.selector != ReferenceSnyggSelector.NONE) {
            if (selector == ReferenceSnyggSelector.NONE) {
                return -1
            }
            if (other.selector == ReferenceSnyggSelector.NONE) {
                return 1
            }
            val selectorDiff = selector.compareTo(other.selector)
            if (selectorDiff != 0) {
                return selectorDiff
            }
        }
        val attrDiff = attributes.compareTo(other.attributes)
        return attrDiff
    }

    override fun toString() = buildString {
        append(elementName)
        append(attributes)
        append(selector)
    }

    companion object : SnyggSpecDecl.RuleDecl {
        override val name = "element"
        private val ELEMENT_NAME_REGEX = """(?<elementName>[a-zA-Z0-9-]+)""".toRegex()

        private val ATTRIBUTES_REGEX = """(?<attributesRaw>(?:${ReferenceSnyggAttributes.ATTRIBUTE_REGEX})+)?""".toRegex()
        private val SELECTOR_REGEX = """(?<selectorRaw>:pressed|:focus|:hover|:disabled)?""".toRegex()
        override val pattern = """^$ELEMENT_NAME_REGEX$ATTRIBUTES_REGEX$SELECTOR_REGEX$""".toRegex()

        /**
         * Attempts to parse the given string into an element rule instance, or `null` if the given string
         * does not represent an element rule instance.
         *
         * @param str the string to parse into an element rule instance
         * @return an element rule instance or `null`
         *
         * @since 0.5.0-alpha01
         */
        fun fromOrNull(str: String): ReferenceSnyggElementRule? {
            val result = pattern.matchEntire(str) ?: return null
            val elementName = result.groups["elementName"]!!.value // cannot be null logically
            val attributesRaw = result.groups["attributesRaw"]?.value
            val selectorRaw = result.groups["selectorRaw"]?.value

            return ReferenceSnyggElementRule(
                elementName = elementName,
                attributes = ReferenceSnyggAttributes.from(attributesRaw ?: ""),
                selector = ReferenceSnyggSelector.from(selectorRaw ?: ""),
            )
        }
    }
}

/**
 * A Snygg selector describes the interaction state of a component. Within stylesheets, this can be used in element
 * rules to target specific interaction states of elements for styling. Within the UI implementation this is used to
 * pass the current interaction state to Snygg to allow for correct style resolving.
 *
 * @property id The id of the selector.
 *
 * @since 0.5.0-alpha01
 */
enum class ReferenceSnyggSelector(val id: String) {
    /**
     * No interaction is active. Only used within UI implementation, is not serialized.
     *
     * @since 0.5.0-alpha01
     */
    NONE("none"),

    /**
     * Pressed interaction.
     *
     * @since 0.5.0-alpha01
     */
    PRESSED("pressed"),

    /**
     * Focus interaction.
     *
     * @since 0.5.0-alpha01
     */
    FOCUS("focus"),

    /**
     * Hover interaction.
     *
     * @since 0.5.0-alpha01
     */
    HOVER("hover"),

    /**
     * Disabled state. Used for inputs and buttons.
     *
     * @since 0.5.0-alpha01
     */
    DISABLED("disabled");

    /**
     * Serializes the selector to a string. If [NONE], an empty string is returned.
     *
     * @return The serialized representation of this selector.
     *
     * @since 0.5.0-alpha01
     */
    override fun toString(): String {
        if (this == NONE) {
            return ""
        }
        return buildString {
            append(SELECTOR_COLON)
            append(id)
        }
    }

    companion object {
        private const val SELECTOR_COLON = ":"

        internal fun from(str: String): ReferenceSnyggSelector {
            if (str.isNotEmpty()) {
                val selector = str.substring(1)
                return entries.first { it.id == selector }
            }
            return NONE
        }
    }
}
