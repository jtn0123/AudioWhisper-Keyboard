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

class SnyggStylesheetEditor(
    private val schema: String,
    initRules: Map<SnyggRule, SnyggPropertySet>? = null,
    comparator: Comparator<SnyggRule>? = null,
) {
    val rules = sortedMapOf<SnyggRule, SnyggPropertySetEditor>(comparator ?: DefaultRuleComparator)

    init {
        if (initRules != null) {
            rules.putAll(
                initRules.mapValues { (_, propertySet) -> propertySet.edit() }
            )
        }
    }

    fun defines(propertySetBlock: SnyggSinglePropertySetEditor.() -> Unit) {
        val propertySetEditor = SnyggSinglePropertySetEditor()
        propertySetBlock(propertySetEditor)
        val rule = SnyggAnnotationRule.Defines
        rules[rule] = propertySetEditor
    }

    fun font(fontName: String, propertySetBlock: SnyggMultiplePropertySetsEditor.() -> Unit) {
        val propertySetEditor = SnyggMultiplePropertySetsEditor()
        propertySetBlock(propertySetEditor)
        val rule = SnyggAnnotationRule.Font(fontName)
        rules[rule] = propertySetEditor
    }

    operator fun String.invoke(
        vararg attributes: Pair<String, List<Any>>,
        selector: SnyggSelector = SnyggSelector.NONE,
        propertySetBlock: SnyggSinglePropertySetEditor.() -> Unit,
    ) {
        val propertySetEditor = SnyggSinglePropertySetEditor()
        propertySetBlock(propertySetEditor)
        val rule = SnyggElementRule(
            elementName = this,
            attributes = SnyggAttributes.of(*attributes),
            selector = selector,
        )
        rules[rule] = propertySetEditor
    }

    fun build(): SnyggStylesheet {
        val rulesMap = rules.mapValues { (_, propertySetEditor) -> propertySetEditor.build() }
        return SnyggStylesheet(schema, rulesMap)
    }

    /** Applies the native add/rename/conflict plan without losing editor object identities. */
    fun updateRule(old: SnyggRule, new: SnyggRule, allowAdd: Boolean = false): Boolean {
        val plan = SnyggCompilerAdapter.ruleEdit(rules.keys, old, new, allowAdd)
        return when (plan[0]) {
            "unchanged" -> true
            "duplicate", "missing" -> false
            "replace" -> {
                val key = rules.keys.elementAt(plan[1].toInt())
                val set = checkNotNull(rules.remove(key))
                rules[new] = set
                true
            }
            "add" -> {
                rules[new] = when (SnyggSpec.propertySetSpecOf(new)!!.type) {
                    SnyggSpecDecl.PropertySet.Type.SINGLE_SET -> SnyggSinglePropertySetEditor()
                    SnyggSpecDecl.PropertySet.Type.MULTIPLE_SETS -> SnyggMultiplePropertySetsEditor()
                }
                true
            }
            else -> error("Invalid native editor plan")
        }
    }

    companion object {
        fun compareRules(a: SnyggRule, b: SnyggRule, elementOrder: List<String>): Int =
            SnyggCompilerAdapter.call(7, SnyggGrammarAdapter.pack(a).toList() +
                SnyggGrammarAdapter.pack(b) + elementOrder.size.toString() + elementOrder).single().toInt()
    }

    private object DefaultRuleComparator : Comparator<SnyggRule> {
        override fun compare(a: SnyggRule, b: SnyggRule): Int {
            return a.compareTo(b)
        }
    }
}
