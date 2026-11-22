package com.jtn0123.audiowhisper.rust

/** Whole settings messages, copied UTF-16. No native lifetime or Android callbacks. */
object SettingsRules {
    fun call(op: SettingsOp, vararg values: String): List<String> = SettingsNative.call(op.code, arrayOf(*values)).toList()
    /** Each preference's [SettingsCatalogue] row, by key. */
    val catalogue: Map<String, List<String>> by lazy { call(SettingsOp.CATALOGUE).chunked(SettingsCatalogue.SIZE).associateBy { it[SettingsCatalogue.KEY] } }
    fun default(key: String) = catalogue.getValue(key)[SettingsCatalogue.DEFAULT]
    fun minimum(key: String) = catalogue.getValue(key)[SettingsCatalogue.MIN]
    fun maximum(key: String) = catalogue.getValue(key)[SettingsCatalogue.MAX]
    fun step(key: String) = catalogue.getValue(key)[SettingsCatalogue.STEP]
    fun clamp(key: String, value: String) = call(SettingsOp.CLAMP, key, value)[0]
    fun enumIndex(key: String, value: String) = call(SettingsOp.ENUM_INDEX, key, value)[0].toInt()
    fun decode(kind: String, json: String) = SettingsValue.unpack(call(SettingsOp.DECODE, kind, json).iterator())
    fun encode(kind: String, value: SettingsValue) = call(SettingsOp.ENCODE, kind, *value.pack().toTypedArray())[0]
    fun defaultSubtype() = SettingsValue.unpack(call(SettingsOp.DEFAULT_SUBTYPE).iterator())
    fun subtype(request: SettingsValue) = SettingsValue.unpack(call(SettingsOp.SUBTYPE, *request.pack().toTypedArray()).iterator())
    fun preset(locale: String, language: String, presets: List<Pair<String, String>>) =
        call(SettingsOp.PRESET, locale, language, *presets.flatMap { listOf(it.first, it.second) }.toTypedArray())[0].toInt()
    fun color(value: String) = call(SettingsOp.COLOR, value)[0]
}

/** Transport values only: field/default/validation/encoding decisions belong to Rust. */
sealed interface SettingsValue {
    data object Null : SettingsValue
    data class Text(val value: String) : SettingsValue
    data class Number(val value: Long) : SettingsValue
    data class Array(val values: List<SettingsValue>) : SettingsValue
    data class Object(val values: Map<String, SettingsValue>) : SettingsValue
    fun pack(): List<String> = when (this) {
        Null -> listOf(SettingsValueTag.NULL)
        is Text -> listOf(SettingsValueTag.TEXT, value)
        is Number -> listOf(SettingsValueTag.NUMBER, value.toString())
        is Array -> listOf(SettingsValueTag.ARRAY, values.size.toString()) + values.flatMap { it.pack() }
        is Object -> listOf(SettingsValueTag.OBJECT, values.size.toString()) + values.flatMap { listOf(it.key) + it.value.pack() }
    }
    operator fun get(key: String) = (this as Object).values.getValue(key)
    fun text() = (this as Text).value
    fun number() = (this as Number).value
    fun array() = (this as Array).values
    companion object {
        fun obj(vararg values: Pair<String, SettingsValue>) = Object(linkedMapOf(*values))
        fun unpack(input: Iterator<String>): SettingsValue = when (input.next()) {
            SettingsValueTag.NULL -> Null
            SettingsValueTag.TEXT -> Text(input.next())
            SettingsValueTag.NUMBER -> Number(input.next().toLong())
            SettingsValueTag.ARRAY -> Array(List(input.next().toInt()) { unpack(input) })
            SettingsValueTag.OBJECT -> Object(buildMap { repeat(input.next().toInt()) { put(input.next(), unpack(input)) } })
            else -> error("Invalid native settings value")
        }
    }
}
