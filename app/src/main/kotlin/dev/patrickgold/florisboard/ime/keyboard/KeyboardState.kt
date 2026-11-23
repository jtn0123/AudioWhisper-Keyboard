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

import com.jtn0123.audiowhisper.rust.KeyModels
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.unit.LayoutDirection
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import org.florisboard.lib.kotlin.collectLatestIn
import java.util.concurrent.atomic.AtomicInteger
import kotlin.contracts.InvocationKind
import kotlin.contracts.contract
import kotlin.properties.Delegates

/**
 * This class is a helper managing the state of the text input logic which
 * affects the keyboard view in rendering and layouting the keys.
 *
 * The state class can hold flags or small unsigned integers, all added up
 * at max 64-bit though.
 *
 * The structure of this 8-byte state register is as follows: (Lower 4 bytes are pretty experimental rn)
 *
 * <Byte 3> | <Byte 2> | <Byte 1> | <Byte 0> | Description
 * ---------|----------|----------|----------|---------------------------------
 *          |          |          |     1111 | Active [KeyboardMode]
 *          |          |          | 1111     | Active [KeyVariation]
 *          |          |       11 |          | InputShiftState
 *          |          |      1   |          | Is selection active (length > 0)
 *          |          |     1    |          | Is manual selection mode
 *          |          |    1     |          | Is manual selection mode (start)
 *          |          |   1      |          | Is manual selection mode (end)
 *          |          | 1        |          | Is incognito mode
 *          |        1 |          |          | Is quick actions overflow visible
 *          |       1  |          |          | Is quick actions editor visible
 *          |    1     |          |          | Is composing enabled
 *          |   1      |          |          | Is character half-width enabled
 *      111 |          |          |          | Ime Ui Mode
 *     1    |          |          |          | Layout Direction (0=LTR, 1=RTL)
 *
 * <Byte 7> | <Byte 6> | <Byte 5> | <Byte 4> | Description
 * ---------|----------|----------|----------|---------------------------------
 *          |          |          |        1 | Subtype selection dialog visible
 *
 * The resulting structure is only relevant during a runtime lifespan and
 * thus can easily be changed without worrying about destroying some saved state.
 *
 * @property rawValue The internal register used to store the flags and region ints that
 *  this keyboard state represents.
 */
open class KeyboardState protected constructor(open var rawValue: ULong) {
    companion object {
        const val STATE_ALL_ZERO: ULong =                   0uL

        fun new(value: ULong = STATE_ALL_ZERO) = KeyboardState(value)
    }

    fun snapshot(): KeyboardState {
        return new(rawValue)
    }

    private fun read(field: Int) = KeyModels.read(rawValue.toLong(), field)
    private fun write(field: Int, value: Int) {
        rawValue = KeyModels.write(rawValue.toLong(), field, value).toULong()
    }

    override fun hashCode(): Int {
        return rawValue.hashCode()
    }

    override fun equals(other: Any?): Boolean {
        return other is KeyboardState && other.rawValue == rawValue
    }

    override fun toString(): String {
        return "0x" + rawValue.toString(16).padStart(16, '0')
    }

    var keyVariation: KeyVariation
        get() = KeyVariation.fromInt(read(1))
        set(v) { write(1, v.toInt()) }

    var keyboardMode: KeyboardMode
        get() = KeyboardMode.fromInt(read(0))
        set(v) { write(0, v.toInt()) }

    var inputShiftState: InputShiftState
        get() = InputShiftState.fromInt(read(2))
        set(v) { write(2, v.toInt()) }

    var imeUiMode: ImeUiMode
        get() = ImeUiMode.fromInt(read(3))
        set(v) { write(3, v.toInt()) }

    var layoutDirection: LayoutDirection
        get() = if ((read(4) != 0)) LayoutDirection.Rtl else LayoutDirection.Ltr
        set(v) { write(4, v.ordinal) }

    /** [layoutDirection] as a flag, for the plain-View root. */
    var isRtl: Boolean
        get() = read(4) != 0
        set(v) { write(4, if (v) 1 else 0) }

    val isLowercase: Boolean
        get() = inputShiftState == InputShiftState.UNSHIFTED

    val isUppercase: Boolean
        get() = inputShiftState != InputShiftState.UNSHIFTED

    var isSelectionMode: Boolean
        get() = (read(5) != 0)
        set(v) { write(5, if (v) 1 else 0) }

    var isManualSelectionMode: Boolean
        get() = (read(6) != 0)
        set(v) { write(6, if (v) 1 else 0) }

    var isManualSelectionModeStart: Boolean
        get() = (read(7) != 0)
        set(v) { write(7, if (v) 1 else 0) }

    var isManualSelectionModeEnd: Boolean
        get() = (read(8) != 0)
        set(v) { write(8, if (v) 1 else 0) }

    var isCursorMode: Boolean
        get() = !isSelectionMode
        set(v) { isSelectionMode = !v }

    var isIncognitoMode: Boolean
        get() = (read(9) != 0)
        set(v) { write(9, if (v) 1 else 0) }

    var isActionsOverflowVisible: Boolean
        get() = (read(10) != 0)
        set(v) { write(10, if (v) 1 else 0) }

    var isActionsEditorVisible: Boolean
        get() = (read(11) != 0)
        set(v) { write(11, if (v) 1 else 0) }

    var isSubtypeSelectionVisible: Boolean
        get() = (read(12) != 0)
        set(v) { write(12, if (v) 1 else 0) }

    var isComposingEnabled: Boolean
        get() = (read(13) != 0)
        set(v) { write(13, if (v) 1 else 0) }

    var isCharHalfWidth: Boolean
        get() = (read(14) != 0)
        set(v) { write(14, if (v) 1 else 0) }

}

class ObservableKeyboardState private constructor(
    initValue: ULong,
    private val dispatchFlow: MutableStateFlow<KeyboardState> = MutableStateFlow(KeyboardState.new(initValue)),
) : KeyboardState(initValue) {

    companion object {
        const val BATCH_ZERO: Int = 0

        fun new(value: ULong = STATE_ALL_ZERO) = ObservableKeyboardState(value)
    }

    /**
     * Snapshots of this state, emitted whenever it changes outside of a batch edit.
     */
    val flow: StateFlow<KeyboardState> = dispatchFlow.asStateFlow()

    @Composable
    fun collectAsState(): State<KeyboardState> = flow.collectAsState()

    fun collectLatestIn(scope: CoroutineScope, action: suspend (value: KeyboardState) -> Unit) {
        flow.collectLatestIn(scope, action)
    }

    override var rawValue by Delegates.observable(initValue) { _, old, new -> if (old != new) dispatchState() }
    private val batchEditCount = AtomicInteger(BATCH_ZERO)

    init {
        dispatchState()
    }

    /**
     * Dispatches the new state to all observers if [batchEditCount] is [BATCH_ZERO] (= no active batch edits).
     */
    private fun dispatchState() {
        if (batchEditCount.get() == BATCH_ZERO) {
            dispatchFlow.value = this.snapshot()
        }
    }

    /**
     * Begins a batch edit. Any modifications done during an active batch edit will not be dispatched to observers
     * until [endBatchEdit] is called. At any time given there can be multiple active batch edits at once. This
     * method is thread-safe and can be called from any thread.
     */
    fun beginBatchEdit() {
        batchEditCount.incrementAndGet()
    }

    /**
     * Ends a batch edit. Will dispatch the current state if there are no more other batch edits active. This method is
     * thread-safe and can be called from any thread.
     */
    fun endBatchEdit() {
        batchEditCount.decrementAndGet()
        dispatchState()
    }

    /**
     * Performs a batch edit by executing the modifier [block]. Any exception that [block] throws will be caught and
     * re-thrown after correctly ending the batch edit.
     */
    inline fun batchEdit(block: (ObservableKeyboardState) -> Unit) {
        contract {
            callsInPlace(block, InvocationKind.EXACTLY_ONCE)
        }
        beginBatchEdit()
        try {
            block(this)
        } catch (e: Throwable) {
            throw e
        } finally {
            endBatchEdit()
        }
    }
}
