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
 * @see [SnyggAnnotationRule.Defines]
 * @see [SnyggAnnotationRule.Font]
 * @see [SnyggElementRule]
 */
sealed interface SnyggRule : Comparable<SnyggRule> {
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
    override fun compareTo(other: SnyggRule): Int

    /**
     * Serializes the Snygg rule to a string. This method never fails.
     *
     * @return The serialized representation of the Snygg rule instance.
     * @since 0.5.0-alpha01
     */
    override fun toString(): String

    companion object {
        /**
         * Attempts to parse the given string into a `SnyggRule` instance or `null` if no match is found.
         *
         * @param str The string to parse into a `SnyggRule`.
         * @return A `SnyggRule` instance if the string matches any supported rule type, or `null` if no match is found.
         * @since 0.5.0-alpha01
         */
        fun fromOrNull(str: String): SnyggRule? = SnyggGrammarAdapter.parse(str)
    }
}

/**
 * Annotation rule base interface. See the specific implementations for details.
 *
 * @since 0.5.0-alpha01
 *
 * @see [SnyggAnnotationRule.Defines]
 * @see [SnyggAnnotationRule.Font]
 */
sealed interface SnyggAnnotationRule : SnyggRule {
    data object Defines : SnyggAnnotationRule, SnyggSpecDecl.RuleDecl {
        override val name: String = "defines"

        override val pattern = SnyggGrammarAdapter.metadata[0].toRegex()

        override fun decl() = this

        override fun compareTo(other: SnyggRule): Int = SnyggGrammarAdapter.compare(this, other)

        override fun toString(): String = SnyggGrammarAdapter.format(this)

        /**
         * Attempts to parse the given string into a `defines` annotation rule instance, or `null` if the given string
         * does not represent a `defines` annotation rule.
         *
         * @param str The string to parse into a `defines` annotation rule instance.
         * @return A `defines` annotation rule instance or `null`.
         *
         * @since 0.5.0-alpha01
         */
        fun fromOrNull(str: String): Defines? = SnyggGrammarAdapter.parse(str, 1) as Defines?
    }

    data class Font(val fontName: String) : SnyggAnnotationRule {
        override fun decl() = Companion

        override fun compareTo(other: SnyggRule): Int = SnyggGrammarAdapter.compare(this, other)

        override fun toString(): String = SnyggGrammarAdapter.format(this)

        companion object : SnyggSpecDecl.RuleDecl {
            override val name = "font"
            override val pattern = SnyggGrammarAdapter.metadata[1].toRegex()

            /**
             * Attempts to parse the given string into a `font` annotation rule instance, or `null` if the given string
             * does not represent a `font` annotation rule.
             *
             * @param str The string to parse into a `font` annotation rule instance.
             * @return A `font` annotation rule instance or `null`.
             *
             * @since 0.5.0-alpha01
             */
            fun fromOrNull(str: String): Font? = SnyggGrammarAdapter.parse(str, 2) as Font?
        }
    }
}

/**
 * A core element in the Snygg styling system, this rule allows targeting specific elements with specific attributes
 * and selectors.
 *
 * @property elementName The element name this rule targets, it can be seen similarly to a CSS class.
 * @property attributes The attributes this rule targets.
 * @property selector The selector this rule targets, or [SnyggSelector.NONE] for not specified.
 *
 * @since 0.5.0-alpha01
 */
data class SnyggElementRule(
    val elementName: String,
    val attributes: SnyggAttributes = SnyggAttributes(),
    val selector: SnyggSelector = SnyggSelector.NONE,
) : SnyggRule {
    init {
        require(SnyggGrammarAdapter.call(9, arrayOf(elementName)).single() == "1") { "element name is invalid" }
    }

    override fun decl() = Companion

    override fun compareTo(other: SnyggRule): Int = SnyggGrammarAdapter.compare(this, other)

    override fun toString() = SnyggGrammarAdapter.format(this)

    companion object : SnyggSpecDecl.RuleDecl {
        override val name = "element"
        override val pattern = SnyggGrammarAdapter.metadata[3].toRegex()

        /**
         * Attempts to parse the given string into an element rule instance, or `null` if the given string
         * does not represent an element rule instance.
         *
         * @param str the string to parse into an element rule instance
         * @return an element rule instance or `null`
         *
         * @since 0.5.0-alpha01
         */
        fun fromOrNull(str: String): SnyggElementRule? = SnyggGrammarAdapter.parse(str, 3) as SnyggElementRule?
    }
}

data class SnyggAttributes internal constructor(
    private val attributes: Map<String, List<String>> = emptyMap(),
) : Map<String, List<String>> by attributes, Comparable<SnyggAttributes> {
    override fun compareTo(other: SnyggAttributes): Int = SnyggGrammarAdapter.compare(this, other)

    /**
     * Serializes the attributes to a string.
     *
     * @return The serialized representation of the attributes.
     *
     * @since 0.5.0-alpha01
     */
    override fun toString() = SnyggGrammarAdapter.format(this)

    /**
     * Copies the attributes, including the given key-value pairs. Duplicate key-value pairs are ignored.
     *
     * @param pairs The list of key-value pairs to include in the copy.
     * @return A new copy of the attribute mapping.
     *
     * @since 0.5.0-alpha01
     */
    fun including(vararg pairs: Pair<String, Any>): SnyggAttributes = SnyggGrammarAdapter.edit(this, pairs, 0)

    /**
     * Copies the attributes, excluding the given key-value pairs. Ignores non-present pairs.
     *
     * @param pairs The list of key-value pairs to exclude in the copy.
     * @return A new copy of the attribute mapping.
     *
     * @since 0.5.0-alpha01
     */
    fun excluding(vararg pairs: Pair<String, Any>): SnyggAttributes = SnyggGrammarAdapter.edit(this, pairs, 1)

    /**
     * Copies the attributes, toggling the given key-value pairs. Existing key-value pairs will be removed,
     * non-existing will be added.
     *
     * @param pairs The list of key-value pairs to toggle in the copy.
     * @return A new copy of the attribute mapping.
     *
     * @since 0.5.0-alpha01
     */
    fun toggling(vararg pairs: Pair<String, Any>): SnyggAttributes = SnyggGrammarAdapter.edit(this, pairs, 2)

    companion object {
        internal val ATTRIBUTE_REGEX = SnyggGrammarAdapter.metadata[2].toRegex()
        internal fun from(str: String) = SnyggGrammarAdapter.attributes(SnyggGrammarAdapter.call(3, arrayOf(str)).iterator())
        internal fun of(vararg pairs: Pair<String, List<Any>>) = SnyggGrammarAdapter.attributes(
            SnyggGrammarAdapter.call(13, SnyggGrammarAdapter.pack(pairs.associate { (k, v) -> k to v.map { it.toString() } })).iterator(),
        )
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
enum class SnyggSelector(val id: String) {
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
    override fun toString(): String = SnyggGrammarAdapter.call(8, arrayOf(ordinal.toString())).single()

    companion object {
        internal fun from(str: String): SnyggSelector {
            val result = SnyggGrammarAdapter.call(7, arrayOf(str))
            if (result.isEmpty()) throw NoSuchElementException("Collection contains no element matching the predicate.")
            return entries[result.single().toInt()]
        }
    }
}
