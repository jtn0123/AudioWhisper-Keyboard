package com.jtn0123.audiowhisper.rust

/** Extracted from the frozen ReferenceNlpManager/ReferenceLatinLanguageProvider/inline/clipboard.
 * Generations replace legacy timestamps, stable dedup rejects repeated provider entries, and
 * password/settings eligibility covers every provider. These deliberate safety changes have
 * directed legacy-counterexample tests; the remaining rules are the original Kotlin decisions.
 */
internal class ReferenceNlp {
    var context = List(22) { "0" }
    var generation = 0L
    var externalGeneration = 0L
    var pending = false
    var due = 0L
    var automatic: List<String>? = null
    var candidates = emptyList<List<String>>()
    var clipboard = emptyList<List<String>>()
    var inlineGeneration = 0L
    var inlinePending = false
    var inline = emptyList<String>()
    var saveDue: Long? = null
    private fun flag(at: Int) = context[at] == "1"
    private val allowed: Boolean get() {
        val raw = context[0].toInt()
        val variation = raw and 0xff0
        val type = raw and 15
        return !(type == 1 && variation in listOf(0x80, 0x90, 0xe0) || type == 2 && variation == 0x10) &&
            !(type == 1 && raw and 0x80000 != 0)
    }
    private val latin get() = context[2] == LATIN && context[3] == "en" && flag(17)
    private val on get() = allowed && (flag(4) || flag(5))
    private val words get() = allowed && flag(4) && latin
    private val emoji get() = allowed && flag(5)
    private val auto get() = words && !(context[0].toInt() and 15 == 1 && context[0].toInt() and 0xff0 in listOf(0x10, 0x20, 0xd0))
    private val learning get() = auto && flag(6) && !flag(7) && !flag(1)
    private val selection get() = context[10] != context[11]
    private val query get() = on && flag(9) && !selection
    private val visible get() = if (!on) emptyList() else clipboard.ifEmpty { candidates }
    private fun invalidate() { generation++; pending = false; automatic = null; candidates = emptyList(); clipboard = emptyList() }
    private fun policy() = listOf(on, words, emoji, auto, learning, flag(4) || flag(5)).map { it.bit() }
    private fun filter(values: List<List<String>>): List<List<String>> {
        if (!query) return emptyList()
        val prefixed = context[16].isNotEmpty() && emoji && values.any { it[2] == "1" }
        return values.filter { v -> (v[2] != "0" || words && !prefixed) && (v[2] != "1" || emoji) }
            .distinctBy { it[2] to it[1] }.map { v -> v.take(3) + (v[3] == "1" && auto && v[2] == "0").bit() }
    }
    private fun snapshot(): List<String> {
        val expanded = if (!flag(8)) -1 else if (visible.isEmpty() && inline.isEmpty() || selection) 1 else 0
        return listOf(generation.toString(), expanded.toString(), (automatic ?: visible.firstOrNull { it[3] == "1" })?.get(0).orEmpty(), visible.size.toString()) +
            visible.map { it[0] } + inline.size.toString() + inline
    }
    fun call(op: String, vararg fields: String): List<String> {
        val f = fields.toList()
        fun bool(value: Boolean) = listOf(value.bit())
        return when (op) {
            "context" -> {
                if (context != f) { invalidate(); externalGeneration++; context = f; inlineGeneration++; inlinePending = false; inline = emptyList() }
                snapshot()
            }
            "beginFor" -> {
                call("context", *f.take(22).toTypedArray())
                call("begin", f[22]) + listOf(words.bit(), emoji.bit()) + snapshot()
            }
            "completeFor" -> {
                call("context", *f.take(22).toTypedArray())
                call("complete", *f.drop(22).toTypedArray()) + snapshot()
            }
            "candidatePlan" -> {
                val old = context; context = f.take(22)
                val allowed = auto && f[22] == "0"
                val result = f.drop(23).map { (allowed && it == "1").bit() }; context = old; result
            }
            "clipboardFor" -> { if (context == f.take(22)) clipboard = if (on) f.drop(22).chunked(4) else emptyList(); snapshot() }
            "externalBegin" -> { externalGeneration++; listOf(externalGeneration.toString()) }
            "externalValid" -> bool(f[0].toLong() == externalGeneration && query)
            "autoCandidate" -> {
                val same = f.take(22) == f.drop(22).take(22)
                call("context", *f.drop(22).take(22).toTypedArray())
                val candidate = f.drop(44)
                val accepted = same && auto && candidate[2] == "0" && candidate[3] == "1"
                if (accepted) automatic = candidate
                bool(accepted)
            }
            "policy" -> { val old = context; context = f; val result = policy(); context = old; result }
            "begin" -> { val a = automatic; invalidate(); automatic = a; pending = query; due = f[0].toLong() + 8; listOf(generation.toString(), if (pending) "8" else "-1") }
            "ready" -> bool(pending && f[0].toLong() == generation && f[1].toLong() >= due)
            "complete" -> {
                val accepted = pending && f[0].toLong() == generation
                if (accepted) { pending = false; candidates = filter(f.drop(1).chunked(4)) }
                bool(accepted)
            }
            "direct" -> { invalidate(); candidates = filter(f.chunked(4)); snapshot() }
            "clipboard" -> { clipboard = if (on) f.chunked(4) else emptyList(); snapshot() }
            "field" -> { externalGeneration++; invalidate(); inlineGeneration++; inlinePending = false; inline = emptyList(); snapshot() }
            "clear" -> { invalidate(); snapshot() }
            "snapshot" -> snapshot()
            "auto" -> bool(auto && f[1] == "0" && !continues(f[0].toInt()) && (visible.isEmpty() || visible.any { it[2] == "0" }))
            "pick" -> {
                val a = automatic?.get(0) == f[0]
                val accept = a || visible.any { it[0] == f[0] }
                if (a) automatic = null
                bool(accept)
            }
            "keep" -> bool(!flag(7))
            "learn" -> {
                val kind = f[0].toInt()
                if (!learning || kind == 0 && (continues(f[1].toInt()) || f[2].isEmpty() || f[5] == "0")) emptyList()
                else {
                    val replaces = f[5] == "1" && context[12] != context[13]
                    val before = if (kind == 2 || !replaces) f[6] else f[3]
                    listOf(f[0], f[2], if (kind == 2 || kind == 1 && !replaces) beforeNextWord(before) else before,
                        if (replaces) f[4] else context[10])
                }
            }
            "save" -> {
                val now = f[0].toLong()
                when {
                    saveDue != null && now >= saveDue!! -> { saveDue = null; listOf("0") }
                    saveDue == null -> { saveDue = now + 5000; listOf("5000") }
                    else -> listOf("-1")
                }
            }
            "inlineBegin" -> { inlineGeneration++; inlinePending = f[0].toInt() > 0 && flag(18); inline = emptyList(); listOf(inlineGeneration.toString(), inlinePending.bit(), "2000") }
            "inlineCancel" -> { if (f[0].toLong() == inlineGeneration) inlinePending = false; snapshot() }
            "inlineComplete" -> {
                val accept = f[0].toLong() == inlineGeneration && inlinePending
                if (accept) { inlinePending = false; inline = f.drop(1).chunked(2).sortedByDescending { it[1].toInt() }.map { it[0] } }
                bool(accept)
            }
            "next" -> listOf(beforeNextWord(f[0]))
            "language" -> bool(f[0] == "en")
            "force" -> bool(false)
            "spell" -> bool(f[0] == "en" && f[1].any(Char::isLetter) && f[1].none(Char::isDigit))
            "typingPolicy" -> {
                val old = context; context = f.take(22)
                val result = typing(f.drop(22)); context = old; result
            }
            "typing" -> typing(f)
            "clipboardPlan" -> clipboardPlan(f)
            else -> error("Unknown test operation $op")
        }
    }
    private fun typing(f: List<String>): List<String> {
        if (!query || !words || f[1].firstOrNull()?.isLetterOrDigit() == false) return emptyList()
        return listOf(if (f[1].isEmpty() && f[0].lastOrNull()?.isLetterOrDigit() == true) f[0] + " " else f[0], f[1], f[2])
    }
    private fun clipboardPlan(f: List<String>): List<String> {
        if (f[0] == "0" || f[1] == f[2] || f[3].isNotBlank() || f[4].isBlank() || f[5].toLong() >= f[6].toLong()) return emptyList()
        if (f[7] == "1") return listOf("-1")
        val matches = f.drop(8).chunked(3)
        return listOf("-1") + matches.indices.filter { i ->
            val match = matches[i]
            match[0] != f[4] && matches.take(i).all { prev -> prev[0] != match[0] &&
                (prev[1].toInt()..prev[2].toInt()).intersect(match[1].toInt()..match[2].toInt()).isEmpty() }
        }.map(Int::toString)
    }
    companion object {
        const val LATIN = "org.florisboard.nlp.providers.latin"
        fun Boolean.bit() = if (this) "1" else "0"
        fun continues(cp: Int) = Character.isDigit(cp) || cp in listOf(39, 0x2019, 45, 64)
        fun beforeNextWord(text: String) = if (text.isNotEmpty() && !text.last().isWhitespace()) "$text " else text
        fun context(text: String = "hel", input: Int = 1): List<String> = listOf(input.toString(), "0", LATIN, "en", "1", "1", "1", "0", "1", "1", text.length.toString(), text.length.toString(), "0", text.length.toString(), text, "fixture-en", "", "1", "1", "1", "0", "0")
    }
}
