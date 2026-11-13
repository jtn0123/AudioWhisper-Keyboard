package org.florisboard.lib.snygg

import kotlin.test.assertEquals

/** No production parser/formatter/comparator is used to construct the reference's rule model. */
internal fun ReferenceSnyggRule.production(): SnyggRule = when (this) {
    ReferenceSnyggAnnotationRule.Defines -> SnyggAnnotationRule.Defines
    is ReferenceSnyggAnnotationRule.Font -> SnyggAnnotationRule.Font(fontName)
    is ReferenceSnyggElementRule -> SnyggElementRule(elementName, SnyggAttributes(attributes.toMap()), SnyggSelector.entries[selector.ordinal])
}

internal fun ReferenceSnyggRule.snapshot(): List<Any> = when (this) {
    ReferenceSnyggAnnotationRule.Defines -> listOf("defines")
    is ReferenceSnyggAnnotationRule.Font -> listOf("font", fontName)
    is ReferenceSnyggElementRule -> listOf("element", elementName, attributes.toMap(), selector.ordinal)
}

internal fun SnyggRule.snapshot(): List<Any> = when (this) {
    SnyggAnnotationRule.Defines -> listOf("defines")
    is SnyggAnnotationRule.Font -> listOf("font", fontName)
    is SnyggElementRule -> listOf("element", elementName, attributes.toMap(), selector.ordinal)
}

internal fun <A, B> sameResult(label: String, a: Result<A>, b: Result<B>, expected: (A) -> Any?, actual: (B) -> Any?) {
    assertEquals(a.isSuccess, b.isSuccess, label)
    if (a.isFailure) {
        assertEquals(a.exceptionOrNull()!!.javaClass.name.replace(".ReferenceSnygg", ".Snygg"), b.exceptionOrNull()!!.javaClass.name, label)
        assertEquals(a.exceptionOrNull()!!.message?.replace("org.florisboard.lib.snygg.reference.ReferenceSnygg", "org.florisboard.lib.snygg.value.Snygg"), b.exceptionOrNull()!!.message, label)
    } else {
        assertEquals(expected(a.getOrThrow()), actual(b.getOrThrow()), label)
    }
}

internal fun checkRule(input: String) {
    val a = runCatching { ReferenceSnyggRule.fromOrNull(input) }
    val b = runCatching { SnyggRule.fromOrNull(input) }
    sameResult(input, a, b, { it?.snapshot() }, { it?.snapshot() })
    sameResult(input, a, b, { it?.toString() }, { it?.toString() })
    if (a.isSuccess && b.isSuccess && a.getOrThrow() != null) {
        val serialized = a.getOrThrow().toString()
        sameResult("round trip $input", runCatching { ReferenceSnyggRule.fromOrNull(serialized) },
            runCatching { SnyggRule.fromOrNull(serialized) }, { it?.snapshot() }, { it?.snapshot() })
    }
}

internal fun checkSheet(input: String, config: SnyggJsonConfiguration) {
    val a = ReferenceSnyggStylesheet.fromJson(input, config)
    val b = SnyggStylesheet.fromJson(input, config)
    sameResult(input, a, b, { listOf(it.schema, it.rules.map { (r, v) -> r.snapshot() to v }) },
        { listOf(it.schema, it.rules.map { (r, v) -> r.snapshot() to v }) })
    if (a.isSuccess && b.isSuccess) {
        val ref = a.getOrThrow()
        val prod = b.getOrThrow()
        assertEquals(ref.rules.keys.sorted().map { it.snapshot() }, prod.rules.keys.sorted().map { it.snapshot() })
        val sa = ref.toJson(config)
        val sb = prod.toJson(config)
        sameResult("serialized $input", sa, sb, { it }, { it })
        if (sa.isSuccess && sb.isSuccess) {
            sameResult("round trip $input", ReferenceSnyggStylesheet.fromJson(sa.getOrThrow(), config),
                SnyggStylesheet.fromJson(sb.getOrThrow(), config), { it.toJson(config).getOrThrow() }, { it.toJson(config).getOrThrow() })
        }
    }
}
