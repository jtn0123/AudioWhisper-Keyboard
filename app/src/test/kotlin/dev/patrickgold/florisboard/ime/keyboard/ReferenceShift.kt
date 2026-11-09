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

package dev.patrickgold.florisboard.ime.keyboard

import dev.patrickgold.florisboard.ime.editor.InputAttributes
import dev.patrickgold.florisboard.ime.input.CapitalizationBehavior
import dev.patrickgold.florisboard.ime.input.InputShiftState

private val DoubleSpacePeriodMatcher = """([^.!?‽\s]\s)""".toRegex()

/**
 * The Kotlin that lib/rust/crates/shift replaced, kept to test against: `KeyboardManager`'s shift
 * handlers, its post-key unshift and double-space test, and `InputEventDispatcher`'s double tap.
 * Copied as it was; what it asked the dispatcher, the preferences and the editor it now takes as
 * arguments, and the state it set is [inputShiftState].
 */
internal class ReferenceShift(var inputShiftState: InputShiftState) {
    fun reevaluateInputShiftState(
        isShiftPressed: Boolean,
        autoCapitalization: Boolean,
        supportsCapitalization: Boolean,
        activeCursorCapsMode: InputAttributes.CapsMode,
    ) {
        if (inputShiftState != InputShiftState.CAPS_LOCK && !isShiftPressed) {
            val shift = autoCapitalization
                && supportsCapitalization
                && activeCursorCapsMode != InputAttributes.CapsMode.NONE
            inputShiftState = when {
                shift -> InputShiftState.SHIFTED_AUTOMATIC
                else -> InputShiftState.UNSHIFTED
            }
        }
    }

    fun handleShiftDown(capitalizationBehavior: CapitalizationBehavior, isConsecutiveDown: Boolean) {
        when (capitalizationBehavior) {
            CapitalizationBehavior.CAPSLOCK_BY_DOUBLE_TAP -> {
                if (isConsecutiveDown) {
                    inputShiftState = InputShiftState.CAPS_LOCK
                } else {
                    if (inputShiftState == InputShiftState.UNSHIFTED) {
                        inputShiftState = InputShiftState.SHIFTED_MANUAL
                    } else {
                        inputShiftState = InputShiftState.UNSHIFTED
                    }
                }
            }
            CapitalizationBehavior.CAPSLOCK_BY_CYCLE -> {
                inputShiftState = when (inputShiftState) {
                    InputShiftState.UNSHIFTED -> InputShiftState.SHIFTED_MANUAL
                    InputShiftState.SHIFTED_MANUAL -> InputShiftState.CAPS_LOCK
                    InputShiftState.SHIFTED_AUTOMATIC -> InputShiftState.UNSHIFTED
                    InputShiftState.CAPS_LOCK -> InputShiftState.UNSHIFTED
                }
            }
        }
    }

    fun handleShiftUp(isAnyPressed: Boolean, isUninterruptedEventSequence: Boolean) {
        if (inputShiftState != InputShiftState.CAPS_LOCK && !isAnyPressed &&
            !isUninterruptedEventSequence) {
            inputShiftState = InputShiftState.UNSHIFTED
        }
    }

    fun handleCapsLock() {
        inputShiftState = InputShiftState.CAPS_LOCK
    }

    fun handleShiftCancel() {
        inputShiftState = InputShiftState.UNSHIFTED
    }

    /** The end of `onInputKeyUp` after a key that types, and of the GIF search's typing. */
    fun afterKeyTyped(isShiftPressed: Boolean) {
        if (inputShiftState != InputShiftState.CAPS_LOCK && !isShiftPressed) {
            inputShiftState = InputShiftState.UNSHIFTED
        }
    }

    companion object {
        /** `handleSpace`'s test of the two characters before the cursor. */
        fun isDoubleSpacePeriod(text: String): Boolean =
            text.length == 2 && DoubleSpacePeriodMatcher.matches(text)

        /** `InputEventDispatcher.isConsecutiveDown` and `isConsecutiveUp`, given the clock and the last event. */
        fun isConsecutive(lastCode: Int, lastTime: Long, code: Int, now: Long, doubleTapTimeout: Long): Boolean =
            lastCode == code && (now - lastTime) < doubleTapTimeout
    }
}
