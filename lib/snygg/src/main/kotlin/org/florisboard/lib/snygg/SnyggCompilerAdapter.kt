package org.florisboard.lib.snygg

import androidx.compose.material3.ColorScheme
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.isSpecified
import androidx.compose.ui.unit.sp
import com.jtn0123.audiowhisper.rust.SnyggCompiler
import org.florisboard.lib.color.getColor
import org.florisboard.lib.snygg.value.*

/** Allocation and platform conversion only. Tokens retain caller-created Compose values. */
internal object SnyggCompilerAdapter {
    fun call(op: Int, input: List<String>): List<String> =
        SnyggCompiler.call(op, input.toTypedArray()).toList().also {
            if (it.firstOrNull() == "!") throw IllegalStateException(it[1])
        }
    private class Packet {
        val rows = mutableListOf<String>()
        val objects = mutableListOf<SnyggValue>()
        fun number(n: Int) { rows.add(n.toString()) }
        fun rule(r: SnyggRule) { rows.addAll(SnyggGrammarAdapter.pack(r)) }
        fun value(v: SnyggValue) {
            when (v) {
                SnyggUndefinedValue -> number(1)
                SnyggInheritValue -> number(2)
                is SnyggDefinedVarValue -> { number(3); rows.add(v.key) }
                is SnyggSpSizeValue if v.sp.isSpecified -> { number(if (v.sp.isEm) 7 else 4); rows.add(v.sp.value.toRawBits().toString()) }
                is SnyggDynamicLightColorValue -> { number(5); rows.add(v.colorName) }
                is SnyggDynamicDarkColorValue -> { number(6); rows.add(v.colorName) }
                else -> { number(0); number(objects.size); objects.add(v) }
            }
        }
        fun properties(p: Map<String, SnyggValue>) {
            number(p.size)
            p.forEach { (k, v) -> rows.add(k); value(v) }
        }
        fun properties(input: Iterator<String>): Map<String, SnyggValue> = buildMap {
            repeat(input.next().toInt()) {
                val key = input.next()
                val value = when (input.next()) {
                    "0" -> objects[input.next().toInt()]
                    "1" -> SnyggUndefinedValue
                    "2" -> SnyggInheritValue
                    "3" -> SnyggDefinedVarValue(input.next())
                    "4" -> SnyggSpSizeValue(Float.fromBits(input.next().toInt()).sp)
                    "7" -> SnyggSpSizeValue(Float.fromBits(input.next().toInt()).em)
                    "5" -> SnyggDynamicLightColorValue(input.next())
                    "6" -> SnyggDynamicDarkColorValue(input.next())
                    else -> error("Invalid Snygg compiler value")
                }
                put(key, value)
            }
        }
    }
    data class Compilation(val style: CompiledStyleData, val fonts: List<Pair<String, List<SnyggSinglePropertySet>>>)
    fun compile(sheet: SnyggStylesheet): Compilation {
        val p = Packet()
        p.number(sheet.rules.size)
        sheet.rules.forEach { (r, sets) ->
            p.rule(r)
            when (sets) {
                is SnyggSinglePropertySet -> { p.number(0); p.properties(sets.properties) }
                is SnyggMultiplePropertySets -> {
                    p.number(1); p.number(sets.sets.size)
                    sets.sets.forEach { p.properties(it.properties) }
                }
            }
        }
        val rows = call(0, p.rows).iterator()
        val style = linkedMapOf<String, MutableList<Pair<SnyggElementRule, SnyggSinglePropertySet>>>()
        repeat(rows.next().toInt()) {
            check(rows.next() == "2")
            val name = rows.next()
            val selector = SnyggSelector.entries[rows.next().toInt()]
            val rule = SnyggElementRule(name, SnyggGrammarAdapter.attributes(rows), selector)
            style.getOrPut(name) { mutableListOf() }.add(rule to SnyggSinglePropertySet(p.properties(rows)))
        }
        val fonts = List(rows.next().toInt()) {
            rows.next() to List(rows.next().toInt()) { SnyggSinglePropertySet(p.properties(rows)) }
        }
        return Compilation(style, fonts)
    }
    fun query(style: CompiledStyleData, name: String, attributes: SnyggQueryAttributes, selector: SnyggSelector,
        parent: SnyggSinglePropertySet, light: ColorScheme, dark: ColorScheme, multiplier: Float): SnyggSinglePropertySet {
        val p = Packet()
        val rules = style[name].orEmpty()
        p.number(rules.size)
        rules.forEach { (r, set) -> p.rule(r); p.properties(set.properties) }
        p.rows.add(name); p.number(selector.ordinal); p.number(attributes.size)
        attributes.forEach { (k, v) -> p.rows.add(k); p.rows.add(v.toString()) }
        p.properties(parent.properties)
        p.rows.add(multiplier.toRawBits().toString())
        // getColor materializes Compose colours; Rust chooses the palette/name and when to apply it.
        val names = (rules.flatMap { it.second.properties.values } + parent.properties.values).mapNotNull {
            when (it) {
                is SnyggDynamicLightColorValue -> it.colorName
                is SnyggDynamicDarkColorValue -> it.colorName
                else -> null
            }
        }.distinct()
        p.properties(names.associateWith { SnyggStaticColorValue(light.getColor(it)) })
        p.properties(names.associateWith { SnyggStaticColorValue(dark.getColor(it)) })
        return SnyggSinglePropertySet(p.properties(call(1, p.rows).iterator()))
    }
    fun apply(current: Map<String, SnyggValue>, style: SnyggSinglePropertySet?, parent: SnyggSinglePropertySet,
        multiplier: Float = 1f): Map<String, SnyggValue> {
        val p = Packet()
        p.properties(current)
        if (style != null) p.properties(style.properties)
        p.properties(parent.properties)
        if (style != null) p.rows.add(multiplier.toRawBits().toString())
        return p.properties(call(if (style == null) 3 else 2, p.rows).iterator())
    }
    fun properties(font: Boolean): List<Triple<String, Pair<Boolean, Boolean>, List<SnyggValueEncoder>>> {
        val rows = call(4, listOf(if (font) "1" else "0")).iterator()
        return List(rows.next().toInt()) {
            Triple(rows.next(), (rows.next() == "1") to (rows.next() == "1"),
                List(rows.next().toInt()) { NativeSnyggValues.encoders[rows.next().toInt()] })
        }
    }
    fun ruleEdit(rules: Collection<SnyggRule>, old: SnyggRule, new: SnyggRule, allowAdd: Boolean): List<String> {
        val p = Packet()
        p.number(rules.size); rules.forEach(p::rule); p.rule(old); p.rule(new); p.number(if (allowAdd) 1 else 0)
        return call(5, p.rows)
    }
    fun slot(keys: Collection<String>, key: String) = call(6, listOf(keys.size.toString()) + keys + key).single().toInt()
}
