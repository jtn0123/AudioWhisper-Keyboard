package com.jtn0123.audiowhisper.rust

/** Whole settings messages, copied UTF-16. No native lifetime or Android callbacks. */
object SettingsRules {
    fun call(op: Int, vararg values: String): List<String> = SettingsNative.call(op, arrayOf(*values)).toList()
    val catalogue: Map<String, List<String>> by lazy { call(0).chunked(7).associateBy { it[0] } }
    fun default(key: String) = catalogue.getValue(key)[2]
    fun minimum(key: String) = catalogue.getValue(key)[3]
    fun maximum(key: String) = catalogue.getValue(key)[4]
    fun step(key: String) = catalogue.getValue(key)[5]
    fun clamp(key: String, value: String) = call(2, key, value)[0]
    fun enumIndex(key: String, value: String) = call(1, key, value)[0].toInt()
    fun decode(kind: String, json: String) = SettingsValue.unpack(call(10, kind, json).iterator())
    fun encode(kind: String, value: SettingsValue) = call(11, kind, *value.pack().toTypedArray())[0]
    fun defaultSubtype() = SettingsValue.unpack(call(20).iterator())
    fun subtype(request: SettingsValue) = SettingsValue.unpack(call(21, *request.pack().toTypedArray()).iterator())
    fun preset(locale: String, language: String, presets: List<Pair<String, String>>) =
        call(22, locale, language, *presets.flatMap { listOf(it.first, it.second) }.toTypedArray())[0].toInt()
    fun editorValid(fields: List<String>) = call(23, *fields.toTypedArray())[0] == "1"
    fun color(value: String) = call(24, value)[0]
}
internal object SettingsNative {
    init { RustLibrary.load() }
    @JvmStatic external fun call(operation: Int, input: Array<String>): Array<String>
}

/** Transport values only: field/default/validation/encoding decisions belong to Rust. */
sealed interface SettingsValue {
    data object Null : SettingsValue
    data class Text(val value: String) : SettingsValue
    data class Number(val value: Long) : SettingsValue
    data class Array(val values: List<SettingsValue>) : SettingsValue
    data class Object(val values: Map<String, SettingsValue>) : SettingsValue
    fun pack(): List<String> = when (this) {
        Null -> listOf("z")
        is Text -> listOf("s", value)
        is Number -> listOf("n", value.toString())
        is Array -> listOf("a", values.size.toString()) + values.flatMap { it.pack() }
        is Object -> listOf("o", values.size.toString()) + values.flatMap { listOf(it.key) + it.value.pack() }
    }
    operator fun get(key: String) = (this as Object).values.getValue(key)
    fun text() = (this as Text).value
    fun number() = (this as Number).value
    fun array() = (this as Array).values
    companion object {
        fun obj(vararg values: Pair<String, SettingsValue>) = Object(linkedMapOf(*values))
        fun unpack(input: Iterator<String>): SettingsValue = when (input.next()) {
            "z" -> Null
            "s" -> Text(input.next())
            "n" -> Number(input.next().toLong())
            "a" -> Array(List(input.next().toInt()) { unpack(input) })
            "o" -> Object(buildMap { repeat(input.next().toInt()) { put(input.next(), unpack(input)) } })
            else -> error("Invalid native settings value")
        }
    }
}
