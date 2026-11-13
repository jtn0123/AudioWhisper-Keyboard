package org.florisboard.lib.snygg

import com.jtn0123.audiowhisper.rust.SnyggGrammar
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.encodeToJsonElement

/** Allocation/codec adapter only. Values remain opaque to Rust until stage 2.4. */
internal object SnyggGrammarAdapter {
    val metadata = SnyggGrammar.call(10, emptyArray())

    fun call(op: Int, input: Array<String>): Array<String> = SnyggGrammar.call(op, input).also(::check)

    private fun check(rows: Array<String>) {
        if (rows.firstOrNull() != "!") return
        when (rows[1]) {
            "number" -> rows[2].toInt() // Preserve the platform's NumberFormatException, including its message.
            "missing" -> throw SnyggMissingSchemaException()
            "invalid" -> throw SnyggInvalidSchemaException(rows[2])
            "unsupported" -> throw SnyggUnsupportedSchemaException(rows[2])
            "rule" -> throw SnyggInvalidRuleException(rows[2])
            else -> error("Invalid Snygg error packet")
        }
    }

    fun errorMessage(kind: Int, value: String = "") = call(14, arrayOf(kind.toString(), value)).single()

    fun pack(attrs: Map<String, List<String>>): Array<String> = buildList {
        add(attrs.size.toString())
        for ((key, values) in attrs) { add(key); add(values.size.toString()); addAll(values) }
    }.toTypedArray()

    fun pack(rule: SnyggRule): Array<String> = when (rule) {
        SnyggAnnotationRule.Defines -> arrayOf("0")
        is SnyggAnnotationRule.Font -> arrayOf("1", rule.fontName)
        is SnyggElementRule -> arrayOf("2", rule.elementName, rule.selector.ordinal.toString()) + pack(rule.attributes)
    }

    fun attributes(rows: Iterator<String>): SnyggAttributes = SnyggAttributes(buildMap {
        repeat(rows.next().toInt()) {
            val key = rows.next()
            put(key, List(rows.next().toInt()) { rows.next() })
        }
    })

    private fun rule(packet: Array<String>): SnyggRule? {
        check(packet)
        if (packet.isEmpty()) return null
        val rows = packet.iterator()
        return when (rows.next()) {
            "0" -> SnyggAnnotationRule.Defines
            "1" -> SnyggAnnotationRule.Font(rows.next())
            "2" -> {
                val name = rows.next()
                val selector = SnyggSelector.entries[rows.next().toInt()]
                SnyggElementRule(name, attributes(rows), selector)
            }
            else -> error("Invalid Snygg rule packet")
        }
    }

    fun parse(str: String, filter: Int = 0) = rule(call(0, arrayOf(filter.toString(), str)))
    fun format(rule: SnyggRule) = call(1, pack(rule)).single()
    fun compare(a: SnyggRule, b: SnyggRule) = call(2, pack(a) + pack(b)).single().toInt()
    fun format(attrs: SnyggAttributes) = call(4, pack(attrs)).single()
    fun compare(a: SnyggAttributes, b: SnyggAttributes) = call(5, pack(a) + pack(b)).single().toInt()

    fun edit(attrs: SnyggAttributes, pairs: Array<out Pair<String, Any>>, mode: Int): SnyggAttributes {
        val input = arrayOf(mode.toString()) + pack(attrs) + arrayOf(pairs.size.toString()) +
            pairs.flatMap { (k, v) -> listOf(k, v.toString()) }.toTypedArray()
        return attributes(call(6, input).iterator())
    }

    fun fromJson(json: String, config: SnyggJsonConfiguration): SnyggStylesheet {
        // Keep kotlinx's codec diagnostics and String decoding (including numeric schema coercion).
        val obj = config.json.decodeFromString<JsonObject>(json)
        val schema: String? = obj["\$schema"]?.let { config.json.decodeFromJsonElement(it) }
        val flags = (if (config.ignoreMissingSchema) 1 else 0) or (if (config.ignoreInvalidSchema) 2 else 0) or
            (if (config.ignoreUnsupportedSchema) 4 else 0) or (if (config.ignoreInvalidRules) 8 else 0)
        val plan = call(11, arrayOf(flags.toString(), if (schema != null) "1" else "0", schema.orEmpty()) + obj.keys)
        val rows = plan.iterator()
        val resolvedSchema = rows.next()
        val slots = mutableListOf<Pair<SnyggRule, SnyggPropertySet>>()
        val values = obj.values.toList()
        repeat(rows.next().toInt()) {
            val source = rows.next().toInt()
            val slot = rows.next().toInt()
            val rule = requireNotNull(rule(Array(rows.next().toInt()) { rows.next() }))
            val value = SnyggPropertySet.fromJsonElement(rule, config, values[source])
            val pair = rule to value
            if (slot == slots.size) slots.add(pair) else slots[slot] = slots[slot].first to value
        }
        return SnyggStylesheet(resolvedSchema, slots.toMap())
    }

    fun toJson(sheet: SnyggStylesheet, config: SnyggJsonConfiguration): String {
        val keys = call(12, arrayOf(sheet.rules.size.toString()) + sheet.rules.keys.flatMap { pack(it).toList() })
        val values = listOf(config.json.encodeToJsonElement(sheet.schema)) + sheet.rules.map { (r, p) -> p.toJsonElement(r, config) }
        val obj = JsonObject(keys.zip(values).toMap<String, JsonElement>())
        return config.json.encodeToString(obj)
    }
}
