package org.florisboard.lib.snygg

import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import org.florisboard.lib.snygg.value.*

class SnyggEditorMatchTest {
    private fun reference(rule: SnyggRule): ReferenceSnyggRule = when (rule) {
        SnyggAnnotationRule.Defines -> ReferenceSnyggAnnotationRule.Defines
        is SnyggAnnotationRule.Font -> ReferenceSnyggAnnotationRule.Font(rule.fontName)
        is SnyggElementRule -> ReferenceSnyggElementRule(rule.elementName, ReferenceSnyggAttributes(rule.attributes.toMap()),
            ReferenceSnyggSelector.entries[rule.selector.ordinal])
    }
    /** Original ThemeEditorScreen onConfirmRule decision, without UI effects. */
    private fun ReferenceSnyggStylesheetEditor.update(old: SnyggRule, new: SnyggRule, allowAdd: Boolean): Boolean {
        return when {
            old == new -> true
            rules.contains(new) -> false
            else -> {
                val set = rules.remove(old)
                when {
                    set != null -> { rules[new] = set; true }
                    allowAdd -> {
                        rules[new] = when (ReferenceSnyggSpec.propertySetSpecOf(new)!!.type) {
                            SnyggSpecDecl.PropertySet.Type.SINGLE_SET -> ReferenceSnyggSinglePropertySetEditor()
                            SnyggSpecDecl.PropertySet.Type.MULTIPLE_SETS -> ReferenceSnyggMultiplePropertySetsEditor()
                        }; true
                    }
                    else -> false
                }
            }
        }
    }
    private val pretty = SnyggJsonConfiguration.of(prettyPrint = true)
    private fun check(a: ReferenceSnyggStylesheetEditor, b: SnyggStylesheetEditor) {
        val expected = a.build()
        val actual = b.build()
        assertEquals(expected.rules.toList(), actual.rules.toList())
        val ref = ReferenceSnyggStylesheet(expected.schema, expected.rules.mapKeys { reference(it.key) })
        for (config in listOf(SnyggJsonConfiguration.DEFAULT, pretty)) {
            sameResult("editor serialization/validation", ref.toJson(config), actual.toJson(config), { it }, { it })
            val aj = ref.toJson(config)
            val bj = actual.toJson(config)
            if (aj.isSuccess) sameResult("editor reload", ReferenceSnyggStylesheet.fromJson(aj.getOrThrow(), config),
                SnyggStylesheet.fromJson(bj.getOrThrow(), config),
                { it.rules.map { (r, p) -> r.snapshot() to p } }, { it.rules.map { (r, p) -> r.snapshot() to p } })
        }
    }
    @Test fun `seeded editor operation histories compare every snapshot serialization and validation`() {
        val random = Random(250511)
        val pool = listOf<SnyggRule>(SnyggAnnotationRule.Defines, SnyggAnnotationRule.Font("Test")) +
            (0..7).flatMap { i -> SnyggSelector.entries.map { SnyggElementRule("e$i", selector = it) } }
        val values = listOf<SnyggValue>(SnyggInheritValue, SnyggUndefinedValue, SnyggDefinedVarValue("--a"),
            SnyggDpSizeValue(4.dp), SnyggSpSizeValue(17.sp), SnyggStaticColorValue.defaultValue(), SnyggUriValue("flex:/a"))
        val starts = SnyggStylesheetMatchTest().sheets().map { SnyggStylesheet.fromJson(it.second).getOrThrow() }
        repeat(120) { run ->
            val sheet = starts[run % starts.size]
            val a = ReferenceSnyggStylesheetEditor(sheet.schema, sheet.rules)
            val b = sheet.edit()
            repeat(60) { step ->
                val keys = a.rules.keys.toList()
                val old = if (random.nextBoolean() && keys.isNotEmpty()) keys.random(random) else pool.random(random)
                val new = pool.random(random)
                when (random.nextInt(9)) {
                    0, 1 -> { val add = random.nextBoolean(); assertEquals(a.update(old, new, add), b.updateRule(old, new, add)) }
                    2 -> { assertEquals(a.rules.remove(old)?.build(), b.rules.remove(old)?.build()) }
                    3 -> {
                        // Copy must be independent when a later property edit changes only one rule.
                        val source = a.rules[old]?.build()
                        if (source != null && new !in a.rules) {
                            a.rules[new] = when (source) {
                                is SnyggSinglePropertySet -> ReferenceSnyggSinglePropertySetEditor(source.properties)
                                is SnyggMultiplePropertySets -> ReferenceSnyggMultiplePropertySetsEditor(source.sets)
                            }
                            b.rules[new] = source.edit()
                        }
                    }
                    4, 5, 6 -> {
                        val ra = a.rules[old]
                        val rb = b.rules[old]
                        if (ra is ReferenceSnyggSinglePropertySetEditor && rb is SnyggSinglePropertySetEditor) {
                            val name = listOf("foreground", "font-size", "--a", "unknown").random(random)
                            if (random.nextBoolean()) { val v = values.random(random); ra.properties[name] = v; rb.properties[name] = v }
                            else assertEquals(ra.properties.remove(name), rb.properties.remove(name))
                        } else if (ra is ReferenceSnyggMultiplePropertySetsEditor && rb is SnyggMultiplePropertySetsEditor) {
                            if (random.nextBoolean() || ra.sets.isEmpty()) {
                                val p = mapOf("src" to values.random(random)); ra.sets.add(ReferenceSnyggSinglePropertySetEditor(p)); rb.sets.add(SnyggSinglePropertySetEditor(p))
                            } else {
                                val i = random.nextInt(ra.sets.size); ra.sets.removeAt(i); rb.sets.removeAt(i)
                            }
                        }
                    }
                    7 -> {
                        // Reinsert in reverse order: default ordering must still be canonical.
                        val ar = a.rules.toList().reversed(); val br = b.rules.toList().reversed()
                        a.rules.clear(); b.rules.clear(); ar.forEach { a.rules[it.first] = it.second }; br.forEach { b.rules[it.first] = it.second }
                    }
                    else -> {
                        val ra = a.rules[old]; val rb = b.rules[old]
                        if (ra is ReferenceSnyggSinglePropertySetEditor && rb is SnyggSinglePropertySetEditor && ra.properties.isNotEmpty()) {
                            val ai = ra.properties.entries.iterator(); val bi = rb.properties.entries.iterator()
                            assertEquals(ai.next().value, bi.next().value); ai.remove(); bi.remove()
                        }
                    }
                }
                try { check(a, b) } catch (e: AssertionError) { throw AssertionError("run $run step $step", e) }
            }
        }
        println("Snygg editor: 7,200 edit steps, each compared as ordered models, compact/pretty JSON, reloads and exact validation errors")
    }
    @Test fun `display ordering matches the original comparator for named and unknown elements`() {
        val names = listOf("root", "window", "key", "key-hint", "smartbar")
        val rules = (names + listOf("aaa", "zzz")).flatMap { name -> SnyggSelector.entries.map { SnyggElementRule(name, selector = it) } } +
            listOf(SnyggAnnotationRule.Defines, SnyggAnnotationRule.Font("A"), SnyggAnnotationRule.Font("B"))
        fun reference(a: SnyggRule, b: SnyggRule): Int {
            if (a !is SnyggElementRule || b !is SnyggElementRule || a.elementName == b.elementName) return a.compareTo(b)
            val ai = names.indexOf(a.elementName); val bi = names.indexOf(b.elementName)
            return if (ai < 0 && bi < 0) a.elementName.compareTo(b.elementName) else if (bi < 0) -1 else if (ai < 0) 1 else ai.compareTo(bi)
        }
        for (a in rules) for (b in rules) assertEquals(reference(a, b), SnyggStylesheetEditor.compareRules(a, b, names))
    }
}
