package com.jtn0123.audiowhisper.rust

/**
 * The shift key and auto-capitalisation, in Rust (lib/rust/crates/shift): the shift state each
 * event leaves, whether a key's press or release is a double tap, and whether a second space types
 * a full stop.
 */
object ShiftRules {
    /** The keyboard's shift state, in `InputShiftState`'s order. */
    enum class State { UNSHIFTED, SHIFTED_MANUAL, SHIFTED_AUTOMATIC, CAPS_LOCK }

    /** What happened. [REEVALUATE] is the cursor or editor changing; [KEY_TYPED], a typing key's release. */
    enum class Event { REEVALUATE, SHIFT_DOWN, SHIFT_UP, CAPS_LOCK, SHIFT_CANCEL, KEY_TYPED }

    /** How the shift key reaches caps lock, in `CapitalizationBehavior`'s order. */
    enum class Behavior { CAPSLOCK_BY_DOUBLE_TAP, CAPSLOCK_BY_CYCLE }

    /** What the editor capitalises at the cursor, in `InputAttributes.CapsMode`'s order. */
    enum class CapsMode { NONE, ALL, WORDS, SENTENCES }

    /**
     * The state [event] leaves [state] in, given what Kotlin knew: [shiftPressed] and [anyPressed]
     * keys down, [consecutiveDown] (a shift press soon after the last), [uninterrupted] (shift was the
     * last key down), and whether the user ([autoCapitalization]) and language ([localeCapitalizes])
     * want capitals. Each event reads only some of them.
     */
    fun next(
        state: State,
        event: Event,
        behavior: Behavior = Behavior.CAPSLOCK_BY_DOUBLE_TAP,
        capsMode: CapsMode = CapsMode.NONE,
        shiftPressed: Boolean = false,
        anyPressed: Boolean = false,
        consecutiveDown: Boolean = false,
        uninterrupted: Boolean = false,
        autoCapitalization: Boolean = false,
        localeCapitalizes: Boolean = false,
    ): State {
        val facts = bit(shiftPressed, SHIFT_PRESSED) or bit(anyPressed, ANY_PRESSED) or
            bit(consecutiveDown, CONSECUTIVE_DOWN) or bit(uninterrupted, UNINTERRUPTED) or
            bit(autoCapitalization, AUTO_CAPITALIZATION) or bit(localeCapitalizes, LOCALE_CAPITALIZES)
        return State.entries[ShiftNative.next(state.ordinal, event.ordinal, behavior.ordinal, capsMode.ordinal, facts)]
    }

    /** Whether a key [code] at [now] follows the last such event, [lastCode] at [lastTime], within [timeout] ms. */
    fun isConsecutive(lastCode: Int, lastTime: Long, code: Int, now: Long, timeout: Long): Boolean =
        ShiftNative.isConsecutive(lastCode, lastTime, code, now, timeout)

    /** Whether the two characters before the cursor, [text], are a space a second space turns into ". ". */
    fun isDoubleSpacePeriod(text: String): Boolean = ShiftNative.isDoubleSpacePeriod(text)

    private fun bit(set: Boolean, bit: Int) = if (set) bit else 0

    // The bits awnative/src/shift.rs reads.
    private const val SHIFT_PRESSED = 1
    private const val ANY_PRESSED = 2
    private const val CONSECUTIVE_DOWN = 4
    private const val UNINTERRUPTED = 8
    private const val AUTO_CAPITALIZATION = 16
    private const val LOCALE_CAPITALIZES = 32
}
