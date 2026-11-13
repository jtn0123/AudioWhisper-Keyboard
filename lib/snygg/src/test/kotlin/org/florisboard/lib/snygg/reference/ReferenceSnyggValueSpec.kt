/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
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

package org.florisboard.lib.snygg.reference

import androidx.annotation.CallSuper
import org.florisboard.lib.kotlin.curlyFormat

private val AngledGroupNameRegex = """<[a-zA-Z0-9]+>""".toRegex()
private val WhitespaceRegex = """\s*""".toRegex()

interface ReferenceSnyggValueSpec {
    val id: String?

    val parsePattern: Regex

    val packTemplate: String

    @CallSuper
    fun parse(str: String, dstMap: ReferenceSnyggIdToValueMap) {
        val match = parsePattern.matchEntire(str)
        checkNotNull(match) { "$str does not match pattern $parsePattern" }
        val groupNames = AngledGroupNameRegex.findAll(parsePattern.toString()).map { angleGroup ->
            angleGroup.value.substring(1, angleGroup.value.length - 1)
        }
        for (groupName in groupNames) {
            match.groups[groupName]?.let { group ->
                dstMap.add(groupName to group.value)
            }
        }
    }

    @CallSuper
    fun pack(srcMap: ReferenceSnyggIdToValueMap): String {
        val raw = packTemplate.curlyFormat { arg ->
            srcMap[arg] ?: error("$arg not provided in mapping")
        }
        check(parsePattern.matchEntire(raw) != null) { "cannot serialize" }
        return raw
    }
}

fun ReferenceSnyggValueSpec(block: ReferenceSnyggValueSpecBuilder.() -> ReferenceSnyggValueSpec): ReferenceSnyggValueSpec {
    return block(ReferenceSnyggValueSpecBuilder.Instance)
}

private inline fun buildPattern(builderAction: StringBuilder.() -> Unit): Regex {
    return buildString {
        append(WhitespaceRegex)
        builderAction()
        append(WhitespaceRegex)
    }.replace("$WhitespaceRegex$WhitespaceRegex", WhitespaceRegex.toString()).toRegex()
}

data class ReferenceSnyggIntValueSpec(
    override val id: String,
    val prefix: String?,
    val suffix: String?,
    val unit: String?,
    val numberPattern: Regex?,
) : ReferenceSnyggValueSpec {
    init {
        require(id.isNotBlank()) { "id cannot be blank" }
    }

    override val parsePattern = buildPattern {
        if (prefix != null) {
            append(prefix)
        }
        val numberPatternStr = numberPattern?.toString() ?: "0|[+-]?[1-9][0-9]*"
        append("(?<$id>$numberPatternStr)")
        if (suffix != null) {
            append(suffix)
        }
        if (unit != null) {
            append(unit)
        }
    }

    override val packTemplate = buildString {
        if (prefix != null) {
            append(prefix)
        }
        append("{$id}")
        if (suffix != null) {
            append(suffix)
        }
        if (unit != null) {
            append(unit)
        }
    }
}

data class ReferenceSnyggFloatValueSpec(
    override val id: String,
    val prefix: String?,
    val suffix: String?,
    val unit: String?,
    val numberPattern: Regex?,
) : ReferenceSnyggValueSpec {
    init {
        require(id.isNotBlank()) { "id cannot be blank" }
    }

    override val parsePattern = buildPattern {
        if (prefix != null) {
            append(prefix)
        }
        val numberPatternStr = numberPattern ?: "(?:0|[1-9][0-9]*)(?:[.][0-9]*)?"
        append("(?<$id>$numberPatternStr)")
        if (suffix != null) {
            append(suffix)
        }
        if (unit != null) {
            append(unit)
        }
    }

    override val packTemplate = buildString {
        if (prefix != null) {
            append(prefix)
        }
        append("{$id}")
        if (suffix != null) {
            append(suffix)
        }
        if (unit != null) {
            append(unit)
        }
    }
}

data class ReferenceSnyggKeywordValueSpec(
    override val id: String,
    val keywords: List<String>,
) : ReferenceSnyggValueSpec {
    init {
        require(id.isNotBlank()) { "id cannot be blank" }
    }

    override val parsePattern = buildPattern {
        append("(?<$id>")
        append(keywords.joinToString("|"))
        append(")")
    }

    override val packTemplate = buildString {
        append("{$id}")
    }
}

data class ReferenceSnyggStringValueSpec(
    override val id: String,
    private val pattern: Regex,
) : ReferenceSnyggValueSpec {
    init {
        require(id.isNotBlank()) { "id cannot be blank" }
    }

    override val parsePattern = buildPattern {
        append("(?<$id>")
        append(pattern)
        append(")")
    }

    override val packTemplate = buildString {
        append("{$id}")
    }
}

data class ReferenceSnyggFunctionValueSpec(
    val name: String,
    val innerSpec: ReferenceSnyggValueSpec,
) : ReferenceSnyggValueSpec {
    override val id = null

    override val parsePattern = buildPattern {
        append(name)
        append("[(]")
        append(innerSpec.parsePattern)
        append("[)]")
    }

    override val packTemplate = buildString {
        append(name)
        append("(")
        append(innerSpec.packTemplate)
        append(")")
    }
}

data class ReferenceSnyggListValueSpec(
    val separator: String,
    val valueSpecs: List<ReferenceSnyggValueSpec>,
) : ReferenceSnyggValueSpec {
    override val id = null

    override val parsePattern = buildPattern {
        for ((index, valueSpec) in valueSpecs.withIndex()) {
            if (index != 0) append(separator)
            append(valueSpec.parsePattern)
        }
    }

    override val packTemplate = buildString {
        for ((index, valueSpec) in valueSpecs.withIndex()) {
            if (index != 0) append(separator)
            append(valueSpec.packTemplate)
        }
    }
}

object ReferenceSnyggNothingValueSpec : ReferenceSnyggValueSpec {
    override val id = null

    override val parsePattern = buildPattern {}

    override val packTemplate = buildString {}
}

class ReferenceSnyggValueFormatListBuilder {
    private val valueFormats = mutableListOf<ReferenceSnyggValueSpec>()

    operator fun ReferenceSnyggValueSpec.unaryPlus() {
        valueFormats.add(this)
    }

    fun build() = valueFormats.toList()
}

class ReferenceSnyggValueSpecBuilder {
    companion object {
        val Instance = ReferenceSnyggValueSpecBuilder()
    }

    inline fun spacedList(
        valueFormatsBlock: ReferenceSnyggValueFormatListBuilder.() -> Unit,
    ) = ReferenceSnyggListValueSpec(
        separator = " ",
        valueSpecs = ReferenceSnyggValueFormatListBuilder().let { valueFormatsBlock(it); it.build() },
    )

    inline fun commaList(
        valueFormatsBlock: ReferenceSnyggValueFormatListBuilder.() -> Unit,
    ) = ReferenceSnyggListValueSpec(
        separator = ",",
        valueSpecs = ReferenceSnyggValueFormatListBuilder().let { valueFormatsBlock(it); it.build() },
    )

    inline fun function(
        name: String,
        innerSpecBuilder: ReferenceSnyggValueSpecBuilder.() -> ReferenceSnyggValueSpec,
    ) = ReferenceSnyggFunctionValueSpec(name, innerSpecBuilder(Instance))

    fun keywords(
        id: String,
        keywords: List<String>,
    ) = ReferenceSnyggKeywordValueSpec(id, keywords)

    fun string(
        id: String,
        regex: Regex,
    ) = ReferenceSnyggStringValueSpec(id, regex)

    fun int(
        id: String,
        prefix: String? = null,
        suffix: String? = null,
        unit: String? = null,
        numberPattern: Regex? = null,
    ) = ReferenceSnyggIntValueSpec(id, prefix, suffix, unit, numberPattern)

    fun float(
        id: String,
        prefix: String? = null,
        suffix: String? = null,
        unit: String? = null,
        numberPattern: Regex? = null,
    ) = ReferenceSnyggFloatValueSpec(id, prefix, suffix, unit, numberPattern)

    fun percentageInt(
        id: String,
    ) = int(id, unit = "%", numberPattern = """100|[1-9]?[0-9]""".toRegex())

    fun percentageFloat(
        id: String,
    ) = float(id, unit = "%", numberPattern = """100(?:[.]0*)?|[1-9]?[0-9](?:[.][0-9]*)?""".toRegex())

    fun nothing() = ReferenceSnyggNothingValueSpec
}
