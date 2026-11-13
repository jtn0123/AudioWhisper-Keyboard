package org.florisboard.lib.snygg.value

import com.jtn0123.audiowhisper.rust.SnyggValues
import kotlin.reflect.KClass

internal fun Float.bits(): String = toRawBits().toString()
internal fun String.float(): Float = Float.fromBits(toInt())

/** Conversion of plain native records into public managed objects, with no value grammar. */
abstract class SnyggNativeValueEncoder<V : SnyggValue>(
    val nativeId: Int,
    private val valueClass: KClass<V>,
    private val construct: (List<String>) -> V,
    private val destruct: (V) -> List<String>,
) : SnyggValueEncoder {
    final override val spec get() = NativeSnyggValues.specs(nativeId)[0]
    final override val alternativeSpecs get() = NativeSnyggValues.specs(nativeId).drop(1)
    final override fun defaultValue(): V = construct(NativeSnyggValues.call(3, listOf(nativeId.toString())).drop(1))
    final override fun serialize(v: SnyggValue) = runCatching {
        require(valueClass.isInstance(v))
        NativeSnyggValues.call(2, listOf(nativeId.toString(), nativeId.toString()) + payload(v)).single()
    }
    final override fun deserialize(v: String): Result<SnyggValue> = runCatching {
        construct(NativeSnyggValues.call(1, listOf(nativeId.toString(), v)).drop(1))
    }
    internal fun fromPayload(data: List<String>): V = construct(data)
    private fun payload(v: SnyggValue): List<String> = destruct(checkNotNull(valueClass.java.cast(v)))
}

internal object NativeSnyggValues {
    val patterns: List<String> by lazy { call(9, emptyList()) }
    private data class Metadata(val specs: List<SnyggValueSpec>, val keywords: List<String>)
    private val metadata = mutableMapOf<Int, Metadata>()
    private fun metadata(id: Int): Metadata = synchronized(metadata) {
        metadata.getOrPut(id) {
            val rows = call(0, listOf(id.toString())).iterator()
            val specs = List(rows.next().toInt()) { index ->
                NativeSpec(id, index, rows.next().ifEmpty { null }, rows.next().toRegex(), rows.next())
            }
            val keywords = List(rows.next().toInt()) { rows.next() }
            val publicSpecs = if (keywords.isEmpty()) specs else listOf(
                SnyggKeywordValueSpec(checkNotNull(specs[0].id), keywords).also { it.nativeBinding = id }
            )
            Metadata(publicSpecs, keywords)
        }
    }
    fun specs(id: Int): List<SnyggValueSpec> = metadata(id).specs
    fun keywords(id: Int): List<String> = metadata(id).keywords
    fun call(operation: Int, input: List<String>): List<String> {
        val rows = SnyggValues.call(operation, input.toTypedArray()).toList()
        if (rows.firstOrNull() == "!") throw when (rows[1]) {
            "state" -> IllegalStateException(rows[2])
            "argument" -> IllegalArgumentException(rows[2])
            "number" -> NumberFormatException(rows[2])
            else -> error("Invalid native value error")
        }
        return rows
    }
    // Order is the stable transport enum, checked against every reference encoder in tests.
    val encoders: List<SnyggValueEncoder> by lazy {
        listOf(SnyggInheritValue, SnyggUndefinedValue, SnyggYesValue, SnyggNoValue,
            SnyggDpSizeValue, SnyggSpSizeValue, SnyggPercentageSizeValue, SnyggStaticColorValue,
            SnyggDynamicLightColorValue, SnyggDynamicDarkColorValue, SnyggGenericFontFamilyValue,
            SnyggCustomFontFamilyValue, SnyggFontStyleValue, SnyggFontWeightValue, SnyggPaddingValue,
            SnyggRectangleShapeValue, SnyggCircleShapeValue, SnyggCutCornerDpShapeValue,
            SnyggCutCornerPercentShapeValue, SnyggRoundedCornerDpShapeValue, SnyggRoundedCornerPercentShapeValue,
            SnyggTextAlignValue, SnyggTextDecorationLineValue, SnyggTextMaxLinesValue,
            SnyggTextOverflowValue, SnyggUriValue, SnyggContentScaleValue, SnyggDefinedVarValue)
    }
    fun id(encoder: SnyggValueEncoder): Int = encoders.indexOf(encoder).also { require(it >= 0) }
    fun value(record: List<String>): SnyggValue = when (val encoder = encoders[record[0].toInt()]) {
        is SnyggNativeValueEncoder<*> -> encoder.fromPayload(record.drop(1))
        is SnyggEnumLikeValueEncoder<*> -> encoder.fromNative(record[1])
        else -> encoder.defaultValue()
    }
    fun parse(encoder: Int, index: Int, str: String, dstMap: SnyggIdToValueMap) {
        val rows = call(4, listOf(encoder.toString(), index.toString(), str))
        rows.drop(1).chunked(2).forEach { dstMap[it[0]] = it[1] }
    }
    fun pack(encoder: Int, index: Int, srcMap: SnyggIdToValueMap): String = call(5,
        listOf(encoder.toString(), index.toString(), srcMap.size.toString()) + srcMap.flatMap { listOf(it.key, it.value) }).single()
    private data class NativeSpec(
        val encoder: Int, val index: Int, override val id: String?,
        override val parsePattern: Regex, override val packTemplate: String,
    ) : SnyggValueSpec {
        override fun parse(str: String, dstMap: SnyggIdToValueMap) = NativeSnyggValues.parse(encoder, index, str, dstMap)
        override fun pack(srcMap: SnyggIdToValueMap): String = NativeSnyggValues.pack(encoder, index, srcMap)
    }
}
