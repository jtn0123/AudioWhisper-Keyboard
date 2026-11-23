package dev.patrickgold.florisboard.ime.nlp

import androidx.compose.ui.graphics.vector.ImageVector
import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.suggest.LatinDictionaryEngine
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.editor.EditorRange
import java.security.MessageDigest
import java.util.Random
import kotlin.test.fail

/**
 * A plain editor for the trace: its text and selection, and the word at the cursor composing once
 * typing or a backspace reaches it; a committed word stops composing and leaves a phantom space.
 */
internal class TraceEditor {
    private val text = StringBuilder()
    private var start = 0
    private var end = 0
    private var composingOn = false
    var phantom = false; private set
    /** The editor shows only the text near the cursor, as a long field does. */
    var windowed = false

    fun content(): EditorContent {
        val word = composing()
        var offset = if (windowed) maxOf(0, start - 30) else 0
        if (word != null) offset = minOf(offset, word.first)
        val range = word?.let { EditorRange(it.first - offset, it.last + 1 - offset) } ?: EditorRange.Unspecified
        return EditorContent(text.substring(offset), offset, EditorRange(start - offset, end - offset), range, range)
    }

    private fun composing(): IntRange? {
        if (!composingOn || start != end) return null
        var a = start
        while (a > 0 && text[a - 1].isLetter()) a--
        var b = start
        while (b < text.length && text[b].isLetter()) b++
        return if (a < b) a until b else null
    }

    val composingText: String get() = composing()?.let { text.substring(it.first, it.last + 1) }.orEmpty()

    private fun replace(from: Int, to: Int, with: String) {
        text.replace(from, to, with)
        start = from + with.length
        end = start
    }

    fun letter(c: String) {
        if (phantom && start > 0 && text[start - 1].isLetter()) replace(start, end, " ")
        phantom = false; composingOn = true
        replace(start, end, c)
    }

    /** A candidate went in: it replaces the word composing, or follows what is there. */
    fun commit(word: String) {
        val current = composing()
        if (current != null) replace(current.first, current.last + 1, word)
        else replace(start, end, if (start > 0 && text[start - 1].isLetterOrDigit()) " $word" else word)
        composingOn = false; phantom = true
    }

    fun separator(s: String) { phantom = false; composingOn = false; replace(start, end, s) }

    fun backspace() {
        when {
            start != end -> replace(start, end, "")
            start > 0 -> replace(start - 1, start, "")
        }
        phantom = false; composingOn = true
    }

    /** EditorOp.REVERT; false when the text is not there to replace. */
    fun revert(revert: TypingEvents.Revert): Boolean {
        if (start != end || start < revert.length) return false
        replace(start - revert.length, start, revert.text)
        phantom = revert.phantomSpace; composingOn = true
        return true
    }

    fun moveTo(position: Int) { start = position.coerceIn(0, text.length); end = start; composingOn = true; phantom = false }

    fun select(random: Random) {
        if (text.isEmpty()) return
        val a = random.nextInt(text.length)
        start = a; end = minOf(text.length, a + 1 + random.nextInt(6)); composingOn = false; phantom = false
    }

    fun newField(initial: String) { text.setLength(0); text.append(initial); start = text.length; end = start; composingOn = false; phantom = false }

    val length get() = text.length
}

/** A candidate that is not a word, as an emoji or a clipboard item is. */
private class OtherCandidate(override val text: CharSequence) : SuggestionCandidate {
    override val secondaryText: CharSequence? = null
    override val confidence = 0.0
    override val isEligibleForAutoCommit = false
    override val isEligibleForUserRemoval = true
    override val icon: ImageVector? = null
    override val sourceProvider: SuggestionProvider? = null
}

/**
 * A seeded run of synthetic typing through both sides at once: words typed letter by letter with
 * typos, spaces and punctuation that auto-correct, strip picks and removals, swipes, backspaces
 * that undo corrections, the undo key, new fields, setting changes, save timers that fire or die
 * with the process, and service recreation. After each event the sides' records must be equal,
 * and the reference's go on [tape].
 */
internal class TypingTrace(
    private val seed: Long,
    private val engines: Pair<LatinDictionaryEngine, LatinDictionaryEngine>,
    private val tape: GoldenTape,
) {
    private val random = Random(seed)
    private var clock = 1_000_000L
    private val now = { clock }
    private val environment = TraceEnvironment()
    private val editor = TraceEditor()
    private val disks = TraceDisk() to TraceDisk()
    private var reference: TypingSide = ReferenceSide(disks.first, environment, engines.first, now)
    private var branch: TypingSide = BranchSide(disks.second, environment, engines.second, now)
    private val sides get() = listOf(reference, branch)
    private var landscape = false
    private var target = ""
    private var typed = 0
    private val shown = ArrayDeque<SuggestionCandidate>()
    /** The last event auto-corrected a word, which a backspace often follows. */
    private var corrected = false
    private var last = emptyList<Pair<String, String>>()
    private val digest = MessageDigest.getInstance("SHA-256")
    /** How often each event and each effect came up. */
    val kinds = sortedMapOf<String, Int>()
    val effects = sortedMapOf<String, Int>()
    /** How many events changed each part of the record. */
    val changes = sortedMapOf<String, Int>()

    /** Runs [events] events, failing at the first that differs; the digest of every record. */
    fun run(events: Int): String {
        repeat(events) { step(it) }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    private fun step(index: Int) {
        clock += 15 + random.nextInt(400)
        sides.forEach { it.effects.log.clear() }
        fireDue()
        val kind = choose()
        corrected = false
        kinds.merge(kind, 1, Int::plus)
        act(kind)
        sides.forEach { it.flushKept() }
        compare(index, kind)
    }

    private fun fireDue() = sides.forEach { side ->
        val due = side.effects.timers.filter { it <= clock }.sorted()
        side.effects.timers.removeAll(due.toSet())
        for (at in due) { side.effects.log += "fired"; side.fire(at) }
    }

    private fun choose(): String {
        if (corrected && random.nextInt(5) < 2) return "backspace"
        val table = if (typed < target.length) TYPING else BETWEEN
        var r = random.nextInt(table.sumOf { it.second })
        for ((kind, weight) in table) { if (r < weight) return kind; r -= weight }
        error("unreachable")
    }

    private fun act(kind: String) {
        when (kind) {
            "letter" -> letter()
            "space" -> finish(null, " ")
            "punctuation" -> finish(".,!?"[random.nextInt(4)].code, null)
            "enter" -> each { clearUndo(); enter(editor.content()) }.also { editor.separator("\n"); endWord() }
            "backspace" -> backspace()
            "pick" -> pick(remove = false)
            "remove" -> pick(remove = true)
            "show" -> show()
            "swipe" -> swipe()
            "undo" -> each { clearUndo(); undoKey() }
            "clear" -> { val fields = environment.fields(editor.content()); each { configure(fields); clear() } }
            "cursor" -> { editor.moveTo(random.nextInt(editor.length + 1)); endWord(); suggestPass() }
            "select" -> { editor.select(random); endWord(); suggestPass() }
            "orientation" -> { landscape = !landscape; each { landscape(landscape) } }
            "setting" -> setting()
            "field" -> newField(recreated = false)
            "recreate" -> newField(recreated = true)
            "death" -> death()
            "idle" -> clock += 5_000 + random.nextInt(4_000)
        }
    }

    private inline fun each(action: TypingSide.() -> Unit) = sides.forEach(action)

    private fun log(text: String) = each { effects.log += text }

    private fun endWord() { target = ""; typed = 0 }

    /** Production configures on each editor change as it starts a suggestion pass; the trace does most times. */
    private fun suggestPass() {
        if (random.nextInt(10) < 7) environment.fields(editor.content()).let { fields -> each { configure(fields) } }
    }

    private fun letter() {
        if (typed >= target.length) { target = nextWord(); typed = 0 }
        val c = target[typed++].toString()
        val bias = (c.lowercase()[0].code * 37 % 21 - 10) / 50f
        val dx = bias + (random.nextFloat() - 0.5f) / 10f
        val dy = -bias / 2 + (random.nextFloat() - 0.5f) / 10f
        each { pressed(c.codePointAt(0), dx, dy); clearUndo(); tapTyped(c) }
        editor.letter(c)
        suggestPass()
    }

    /** A space ([separator]) or a punctuation mark ([trigger]) ends the word, auto-correcting it if offered. */
    private fun finish(trigger: Int?, separator: String?) {
        val content = editor.content()
        val requested = environment.fields(content)
        each { clearUndo(); if (trigger == null) noteKey() }
        val offered = offer(content.composingText)
        val chosen = sides.map { it.autoCommit(requested, trigger, editor.phantom, offered, environment.fields(content)) }
        val committed = sides.indices.map { sides[it].autoCorrect(content, trigger, chosen[it]) }
        sides.indices.forEach { sides[it].effects.log += "auto ${chosen[it]?.text} ${committed[it]}" }
        if (chosen[0] != null && committed[0]) editor.commit(chosen[0]!!.text.toString())
        corrected = committed[0] && chosen[0]!!.text.toString() != content.composingText
        if (trigger != null) each { tapTyped(String(Character.toChars(trigger))) }
        editor.separator(separator ?: String(Character.toChars(trigger!!)))
        endWord()
        suggestPass()
    }

    private fun backspace() {
        val content = editor.content()
        each { backspace(content.selection) }
        val reverts = sides.map { it.undoCorrection(content) }
        sides.indices.forEach { sides[it].effects.log += "revert ${reverts[it]}" }
        val undone = reverts[0] != null && random.nextInt(30) != 0 && editor.revert(reverts[0]!!)
        if (undone) {
            log("undone")
            each { correctionUndone() }
            endWord()
        } else {
            if (editor.composingText.isNotEmpty()) each { tapDeleted() }
            editor.backspace()
            typed = maxOf(0, typed - 1)
        }
        suggestPass()
    }

    /**
     * A tap on the strip as the keyboard shows it, the branch's row, or on a word shown before. The
     * row is a StateFlow, which keeps the old objects when a list equal to them is published, so a
     * tap can be on an object the session no longer holds; main's was a StateFlow too, but
     * [ReferenceNlpState]'s is not, so the tap is never taken from it.
     */
    private fun pick(remove: Boolean) {
        val current = branch.candidates
        val candidate = when {
            current.isNotEmpty() && random.nextInt(100) < 85 -> current[random.nextInt(current.size)]
            shown.isNotEmpty() -> shown[random.nextInt(shown.size)]
            else -> return log("nothing to pick")
        }
        val content = editor.content()
        val went = sides.map { if (remove) it.remove(candidate, content) else it.commit(candidate, content) }
        sides.indices.forEach { sides[it].effects.log += "picked ${went[it]}" }
        if (!remove && went[0]) { editor.commit(candidate.text.toString()); endWord(); suggestPass() }
    }

    private fun show() {
        val prefix = editor.composingText.lowercase()
        val words = if (prefix.isEmpty()) List(3) { WORDS[random.nextInt(WORDS.size)] }
            else (WORDS.filter { it.startsWith(prefix) }.take(2) + listOfNotNull(TYPOS[prefix], prefix)).distinct().take(3)
        val values: List<SuggestionCandidate> = words.map { WordSuggestionCandidate(it, isEligibleForAutoCommit = it == TYPOS[prefix]) } +
            if (random.nextInt(12) == 0) listOf(OtherCandidate("#${WORDS[random.nextInt(WORDS.size)]}")) else emptyList()
        values.forEach { shown.addLast(it); if (shown.size > 30) shown.removeFirst() }
        each { show(values) }
    }

    private fun swipe() {
        val word = cased(WORDS[random.nextInt(WORDS.size)])
        val content = editor.content()
        each { swipe(word, content) }
        editor.commit(word)
        endWord()
        suggestPass()
    }

    /** A setting changes; each is mostly as a user leaves it, so words are learned much of the time. */
    private fun setting() {
        when (random.nextInt(4)) {
            0 -> environment.learn = random.nextInt(8) != 0
            1 -> environment.incognito = random.nextInt(8) == 0
            2 -> environment.aim = random.nextInt(5) != 0
            else -> environment.loaded = random.nextInt(8) != 0
        }
        environment.fields(editor.content()).let { fields -> each { configure(fields) } }
    }

    /** A new field, or the same after the service was recreated: the session and its armed undo stay. */
    private fun newField(recreated: Boolean) {
        environment.inputType = INPUT_TYPES[random.nextInt(INPUT_TYPES.size)]
        editor.windowed = random.nextInt(4) == 0
        editor.newField(if (random.nextBoolean()) "" else List(random.nextInt(8) + 1) { WORDS[random.nextInt(WORDS.size)] }.joinToString(" ", postfix = " "))
        endWord()
        if (recreated) each { landscape(landscape) }
        each { field() }
        environment.fields(editor.content()).let { fields -> each { configure(fields) } }
    }

    /** The process dies: saves still waiting are lost, and a new one starts from what was written. */
    private fun death() {
        reference = ReferenceSide(disks.first, environment, engines.first, now)
        branch = BranchSide(disks.second, environment, engines.second, now)
        log("restarted")
        newField(recreated = true)
    }

    private fun nextWord(): String = cased(when (random.nextInt(10)) {
        in 0..5 -> WORDS[random.nextInt(WORDS.size)]
        in 6..7 -> TYPOS.keys.elementAt(random.nextInt(TYPOS.size))
        else -> MADE_UP[random.nextInt(MADE_UP.size)]
    })

    private fun cased(word: String) = when (random.nextInt(20)) {
        in 0..2 -> word.replaceFirstChar { it.uppercase() }
        3 -> word.uppercase()
        else -> word
    }

    /** What the provider offers to auto-correct [typed] to: its fix, sometimes itself or another word. */
    private fun offer(typed: String): SuggestionCandidate? {
        if (typed.isEmpty()) return null
        val fix = TYPOS[typed.lowercase()]?.let { if (typed[0].isUpperCase()) it.replaceFirstChar { c -> c.uppercase() } else it }
        val text = fix ?: when (random.nextInt(12)) { 0 -> typed; 1 -> WORDS[random.nextInt(WORDS.size)]; else -> return null }
        return WordSuggestionCandidate(text, confidence = 0.9, isEligibleForAutoCommit = true)
    }

    private fun compare(index: Int, kind: String) {
        val expected = reference.observe()
        val actual = branch.observe()
        for ((want, got) in expected.zip(actual)) {
            if (want.second != got.second) fail("seed $seed event $index ($kind): ${want.first} differs from main's fan-out")
        }
        expected.forEach { (name, value) -> digest.update("$name=$value\n".toByteArray()); tape.answer(name, value) }
        expected.zip(last).forEach { (now, before) -> if (now != before) changes.merge(now.first, 1, Int::plus) }
        last = expected
        reference.effects.log.forEach { effects.merge(it.substringBefore(' '), 1, Int::plus) }
    }

    private companion object {
        val TYPING = listOf("letter" to 760, "backspace" to 60, "pick" to 40, "show" to 50, "space" to 20,
            "punctuation" to 10, "swipe" to 10, "undo" to 5, "cursor" to 5, "idle" to 20, "setting" to 5, "death" to 2,
            "field" to 5, "recreate" to 3, "remove" to 5)
        val BETWEEN = listOf("letter" to 300, "space" to 180, "punctuation" to 60, "enter" to 30, "backspace" to 100,
            "pick" to 70, "show" to 50, "remove" to 10, "swipe" to 60, "undo" to 15, "clear" to 10, "cursor" to 15,
            "select" to 5, "orientation" to 3, "setting" to 15, "field" to 8, "recreate" to 4, "death" to 2, "idle" to 30)
        val WORDS = listOf("the", "and", "you", "that", "this", "with", "have", "from", "they", "hello", "world", "keyboard",
            "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "morning", "coffee", "river", "garden", "window")
        val MADE_UP = listOf("zorblax", "glorp", "flimbo", "quenta", "brix")
        val TYPOS = mapOf("teh" to "the", "hte" to "the", "adn" to "and", "yuo" to "you", "taht" to "that", "wrold" to "world")
        /** Text, a password, an e-mail address, a URL, a filter, and text that asks for no suggestions. */
        val INPUT_TYPES = listOf(1, 1, 1, 1, 0x4001, 0x4001, 0x8001, 0x8001, 0x81, 0x21, 0x11, 0xb1, 0x80001)
    }
}
