package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.rust.ShiftRules
import dev.patrickgold.florisboard.ime.editor.InputAttributes
import dev.patrickgold.florisboard.ime.input.CapitalizationBehavior
import dev.patrickgold.florisboard.ime.input.InputShiftState

/**
 * AudioWhisper: [KeyboardManager]'s shift key and auto-capitalisation, in Rust
 * (lib/rust/crates/shift): the shift state each event leaves, and whether a second space types a
 * full stop, and `InputEventDispatcher`'s double tap. [KeyboardManager] still asks which keys are
 * down and where the cursor is.
 */
internal object ShiftLogic {
    /**
     * Unless locked or [shiftPressed], [InputShiftState.SHIFTED_AUTOMATIC] where the editor
     * capitalises ([capsMode]) if the user and the language want it, and otherwise unshifted.
     */
    fun reevaluate(
        state: InputShiftState,
        shiftPressed: Boolean,
        autoCapitalization: Boolean,
        localeCapitalizes: Boolean,
        capsMode: InputAttributes.CapsMode,
    ): InputShiftState = next(
        state, ShiftRules.Event.REEVALUATE, capsMode = capsMode(capsMode), shiftPressed = shiftPressed,
        autoCapitalization = autoCapitalization, localeCapitalizes = localeCapitalizes,
    )

    /** A shift press: a lock, a toggle or a step of the cycle, as [behavior] says. */
    fun shiftDown(state: InputShiftState, behavior: CapitalizationBehavior, consecutiveDown: Boolean): InputShiftState =
        next(state, ShiftRules.Event.SHIFT_DOWN, behavior = behavior(behavior), consecutiveDown = consecutiveDown)

    /** A shift release: unshifted if other keys were typed while it was held and none is down now. */
    fun shiftUp(state: InputShiftState, anyPressed: Boolean, uninterrupted: Boolean): InputShiftState =
        next(state, ShiftRules.Event.SHIFT_UP, anyPressed = anyPressed, uninterrupted = uninterrupted)

    fun capsLock(state: InputShiftState): InputShiftState = next(state, ShiftRules.Event.CAPS_LOCK)

    fun shiftCancel(state: InputShiftState): InputShiftState = next(state, ShiftRules.Event.SHIFT_CANCEL)

    /** A key that types: shift lasts one key unless locked or [shiftPressed]. */
    fun keyTyped(state: InputShiftState, shiftPressed: Boolean): InputShiftState =
        next(state, ShiftRules.Event.KEY_TYPED, shiftPressed = shiftPressed)

    /**
     * Whether a key [code] at [now] follows the last such event, [lastCode] at [lastTime], within
     * [timeout] ms: `InputEventDispatcher`'s double tap.
     */
    fun isConsecutive(lastCode: Int, lastTime: Long, code: Int, now: Long, timeout: Long): Boolean =
        ShiftRules.isConsecutive(lastCode, lastTime, code, now, timeout)

    /** Whether [textBeforeCursor], two characters, is a space a second space turns into ". ". */
    fun isDoubleSpacePeriod(textBeforeCursor: String): Boolean = ShiftRules.isDoubleSpacePeriod(textBeforeCursor)

    private fun next(
        state: InputShiftState,
        event: ShiftRules.Event,
        behavior: ShiftRules.Behavior = ShiftRules.Behavior.CAPSLOCK_BY_DOUBLE_TAP,
        capsMode: ShiftRules.CapsMode = ShiftRules.CapsMode.NONE,
        shiftPressed: Boolean = false,
        anyPressed: Boolean = false,
        consecutiveDown: Boolean = false,
        uninterrupted: Boolean = false,
        autoCapitalization: Boolean = false,
        localeCapitalizes: Boolean = false,
    ): InputShiftState = state(
        ShiftRules.next(
            state(state), event, behavior, capsMode, shiftPressed, anyPressed, consecutiveDown, uninterrupted,
            autoCapitalization, localeCapitalizes,
        ),
    )

    private fun state(state: InputShiftState) = when (state) {
        InputShiftState.UNSHIFTED -> ShiftRules.State.UNSHIFTED
        InputShiftState.SHIFTED_MANUAL -> ShiftRules.State.SHIFTED_MANUAL
        InputShiftState.SHIFTED_AUTOMATIC -> ShiftRules.State.SHIFTED_AUTOMATIC
        InputShiftState.CAPS_LOCK -> ShiftRules.State.CAPS_LOCK
    }

    private fun state(state: ShiftRules.State) = when (state) {
        ShiftRules.State.UNSHIFTED -> InputShiftState.UNSHIFTED
        ShiftRules.State.SHIFTED_MANUAL -> InputShiftState.SHIFTED_MANUAL
        ShiftRules.State.SHIFTED_AUTOMATIC -> InputShiftState.SHIFTED_AUTOMATIC
        ShiftRules.State.CAPS_LOCK -> InputShiftState.CAPS_LOCK
    }

    private fun behavior(behavior: CapitalizationBehavior) = when (behavior) {
        CapitalizationBehavior.CAPSLOCK_BY_DOUBLE_TAP -> ShiftRules.Behavior.CAPSLOCK_BY_DOUBLE_TAP
        CapitalizationBehavior.CAPSLOCK_BY_CYCLE -> ShiftRules.Behavior.CAPSLOCK_BY_CYCLE
    }

    private fun capsMode(capsMode: InputAttributes.CapsMode) = when (capsMode) {
        InputAttributes.CapsMode.NONE -> ShiftRules.CapsMode.NONE
        InputAttributes.CapsMode.ALL -> ShiftRules.CapsMode.ALL
        InputAttributes.CapsMode.WORDS -> ShiftRules.CapsMode.WORDS
        InputAttributes.CapsMode.SENTENCES -> ShiftRules.CapsMode.SENTENCES
    }
}
