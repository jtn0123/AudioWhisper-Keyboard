package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.golden.GoldenTape
import dev.patrickgold.florisboard.ime.editor.InputAttributes
import dev.patrickgold.florisboard.ime.input.CapitalizationBehavior
import dev.patrickgold.florisboard.ime.input.InputShiftState
import kotlin.random.Random

/**
 * Asks the shift key in Rust ([ShiftLogic], and the double tap of `InputEventDispatcher`) what the
 * Kotlin it replaced, `ReferenceShift`, was asked, and records each answer on [tape], whose golden
 * file holds that reference's answers (docs/phase6-kotlin-references.md). It also counts how often
 * each outcome was seen, so a caller can tell the runs reached them all.
 */
class ShiftComparison(private val tape: GoldenTape) {
    val seen = mutableMapOf<String, Int>()

    enum class Event { REEVALUATE, SHIFT_DOWN, SHIFT_UP, CAPS_LOCK, SHIFT_CANCEL, KEY_TYPED }

    /** Everything an event might read; each reads only some. */
    data class Facts(
        val behavior: CapitalizationBehavior,
        val capsMode: InputAttributes.CapsMode,
        val shiftPressed: Boolean,
        val anyPressed: Boolean,
        val consecutiveDown: Boolean,
        val uninterrupted: Boolean,
        val autoCapitalization: Boolean,
        val localeCapitalizes: Boolean,
    )

    private fun check(what: String, rust: Any?) = tape.answer(what, rust)

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    private fun rust(state: InputShiftState, event: Event, facts: Facts): InputShiftState = when (event) {
        Event.REEVALUATE -> ShiftLogic.reevaluate(
            state, facts.shiftPressed, facts.autoCapitalization, facts.localeCapitalizes, facts.capsMode,
        )
        Event.SHIFT_DOWN -> ShiftLogic.shiftDown(state, facts.behavior, facts.consecutiveDown)
        Event.SHIFT_UP -> ShiftLogic.shiftUp(state, facts.anyPressed, facts.uninterrupted)
        Event.CAPS_LOCK -> ShiftLogic.capsLock(state)
        Event.SHIFT_CANCEL -> ShiftLogic.shiftCancel(state)
        Event.KEY_TYPED -> ShiftLogic.keyTyped(state, facts.shiftPressed)
    }

    private fun facts(behavior: CapitalizationBehavior, capsMode: InputAttributes.CapsMode, bits: Int) = Facts(
        behavior, capsMode, bits and 1 != 0, bits and 2 != 0, bits and 4 != 0, bits and 8 != 0,
        bits and 16 != 0, bits and 32 != 0,
    )

    /** Every event from every state, under each behaviour and caps mode and every set of facts. */
    fun everyCase() {
        for (state in InputShiftState.entries) for (event in Event.entries) {
            for (behavior in CapitalizationBehavior.entries) for (capsMode in InputAttributes.CapsMode.entries) {
                for (bits in 0 until 64) {
                    val facts = facts(behavior, capsMode, bits)
                    val rust = rust(state, event, facts)
                    check("$event", rust)
                    see("every $event ${state.name} -> ${rust.name}")
                }
            }
        }
    }

    /**
     * Random runs of shift presses, releases, cancels, caps locks, typed keys and cursor moves, the
     * state carried from one event to the next, with random facts.
     */
    fun runs(random: Random, runs: Int, steps: Int) = repeat(runs) {
        var rust = InputShiftState.entries.random(random)
        val behavior = CapitalizationBehavior.entries.random(random)
        repeat(steps) {
            val event = EVENT_MIX.random(random)
            val facts = facts(behavior, InputAttributes.CapsMode.entries.random(random), random.nextInt(64))
            val from = rust
            rust = rust(rust, event, facts)
            check("$event", rust)
            see("run $event ${from.name} -> ${rust.name}")
        }
    }

    /** The double tap's clock: same and other codes, gaps either side of the timeout, wrapping clocks. */
    fun doubleTaps(random: Random, count: Int) = repeat(count) {
        val lastCode = CODES.random(random).takeIf { it != 0 } ?: random.nextInt()
        val code = if (random.nextInt(4) == 0) CODES.random(random) else lastCode
        val lastTime = TIMES.random(random).takeIf { it != 0L } ?: random.nextLong()
        val timeout = TIMEOUTS.random(random).takeIf { it != 0L } ?: random.nextLong()
        val gap = when (random.nextInt(3)) {
            0 -> random.nextLong()
            1 -> timeout + random.nextLong(-2, 3)
            else -> GAPS.random(random)
        }
        val now = lastTime + gap
        val rust = ShiftLogic.isConsecutive(lastCode, lastTime, code, now, timeout)
        check("isConsecutive", rust)
        see("consecutive: $rust")
    }

    /** Every UTF-16 unit before and after each partner, then random texts of up to three odd units. */
    fun doubleSpaces(random: Random, texts: Int) {
        for (unit in 0..0xFFFF) for (partner in PARTNERS) {
            spaces("$partner${unit.toChar()}")
            spaces("${unit.toChar()}$partner")
        }
        repeat(texts) {
            spaces(String(CharArray(random.nextInt(4)) { if (random.nextBoolean()) PARTNERS.random(random) else random.nextInt(0x10000).toChar() }))
        }
    }

    private fun spaces(text: String) {
        val rust = ShiftLogic.isDoubleSpacePeriod(text)
        check("isDoubleSpacePeriod", rust)
        see("double space ${text.length}: $rust")
    }

    private companion object {
        /** Shift's own events more often than the rest, so locks and toggles build up. */
        val EVENT_MIX = Event.entries + listOf(Event.SHIFT_DOWN, Event.SHIFT_DOWN, Event.SHIFT_UP, Event.REEVALUATE)

        // -11 is KeyCode.SHIFT and 32 the space; 0 means a random code.
        val CODES = listOf(-11, -12, 32, 33, Int.MIN_VALUE, Int.MAX_VALUE, 0)
        val TIMES = listOf(1_000L, Long.MAX_VALUE, Long.MIN_VALUE, Long.MAX_VALUE - 299, -1L, 0L)
        val TIMEOUTS = listOf(300L, 1L, -1L, Long.MAX_VALUE, Long.MIN_VALUE, 0L)
        val GAPS = listOf(0L, 1L, 299L, 300L, 301L, -1L, -300L, Long.MAX_VALUE, Long.MIN_VALUE)

        /**
         * Sentence ends, ASCII spaces and controls, the separators only `isWhitespace` takes, the
         * spaces only Android's `\s` takes (NEL, no-break, ideographic, line separator), a zero-width
         * space, a byte-order mark and both halves of a surrogate pair.
         */
        val PARTNERS = " a.!?‽,\t\n\u000B\u000C\r\u001C\u001F\u0085      　​﻿😀".toList()
    }
}
