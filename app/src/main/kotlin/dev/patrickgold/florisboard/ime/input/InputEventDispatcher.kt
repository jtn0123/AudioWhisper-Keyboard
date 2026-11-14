/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.input

import android.os.SystemClock
import android.view.ViewConfiguration
import com.jtn0123.audiowhisper.rust.InputDispatch
import com.jtn0123.audiowhisper.rust.InputProtocol.*
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlinx.coroutines.*

/** Clock, scheduling and receiver execution only. Rust owns the pressed-key lifecycle. */
class InputEventDispatcher private constructor(repeatableKeyCodes: IntArray) {
    companion object {
        private val doubleTapTimeout = ViewConfiguration.getDoubleTapTimeout().toLong()
        private val keyRepeatDelay = ViewConfiguration.getKeyRepeatDelay().toLong()
        fun new(repeatableKeyCodes: IntArray = intArrayOf()) = InputEventDispatcher(repeatableKeyCodes)
    }
    private val prefs by FlorisPreferenceStore
    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    private val native = InputDispatch().also { it.call(INIT, *repeatableKeyCodes.map(Int::toLong).toLongArray()) }
    // Managed key/callback ownership only; membership decisions come from Rust.
    private val jobs = mutableMapOf<Int, PressedKeyInfo>()
    var keyEventReceiver: InputKeyEventReceiver? = null

    @Synchronized fun sendDown(data: KeyData, onLongPress: () -> Boolean = {false},
        onRepeat: () -> Boolean = {true}, scheduleHold: Boolean = true): PressedKeyInfo? {
        val now = SystemClock.uptimeMillis()
        val down = native.call(DOWN, data.code.toLong(), now, if(scheduleHold) 1 else 0)
        if (down.isEmpty()) return null
        val generation = down[4]
        val timing = native.call(DELAYS, data.code.toLong(), prefs.keyboard.longPressDelay.get().toLong(), keyRepeatDelay, 0)
        val info = PressedKeyInfo(now)
        jobs[data.code] = info
        DeleteTiming.event(data.code, now, "down")
        if (down[3] != 0L) info.job = scope.launch {
            DeleteTiming.wait(data.code, now, "hold", timing[0], prefs.keyboard.longPressDelay.get().toLong())
            delay(timing[0])
            val handled = withContext(Dispatchers.Main) { onLongPress() }
            // Like the legacy coroutine, sample repeat preferences after the hold callback.
            val words = prefs.gestures.deleteKeyLongPress.get() == SwipeAction.DELETE_WORD
            val plan = native.call(HOLD_RESULT, data.code.toLong(), if(handled) 1 else 0, if(words) 1 else 0, generation)
            if (plan.isNotEmpty()) {
                val repeatDelay = native.call(DELAYS, data.code.toLong(), 0, keyRepeatDelay, if(words) 1 else 0)[1]
                val repeatData = if (plan[2] == 0L) data else checkNotNull(TextKeyData.getCodeInfoAsTextKeyData(plan[1].toInt()))
                while (isActive) {
                    val accepted = withContext(Dispatchers.Main) { onRepeat() }
                    DeleteTiming.event(data.code, now, "attempt")
                    val emit = native.call(REPEAT_RESULT, data.code.toLong(), if(accepted) 1 else 0, generation)[0] != 0L
                    DeleteTiming.event(data.code, now, if (emit) "tick" else "skipped")
                    if (emit) keyEventReceiver?.onInputKeyRepeat(repeatData)
                    DeleteTiming.wait(data.code, now, "repeat", repeatDelay, keyRepeatDelay)
                    delay(repeatDelay)
                }
            }
        }
        keyEventReceiver?.onInputKeyDown(data)
        native.call(ACK_DOWN,data.code.toLong(),now)
        return info
    }
    @Synchronized fun sendUp(data: KeyData) {
        val plan=native.call(UP,data.code.toLong(),SystemClock.uptimeMillis())
        jobs.remove(data.code)?.let {
            it.cancelJobs()
            DeleteTiming.event(data.code, it.eventTimeDown, "up")
        }
        if (plan.isEmpty()) return
        if (plan[0].toInt()==EMIT_UP) {
            keyEventReceiver?.onInputKeyUp(data)
            native.call(ACK_UP,data.code.toLong(),SystemClock.uptimeMillis())
        } else keyEventReceiver?.onInputKeyCancel(data)
    }
    @Synchronized fun sendDownUp(data: KeyData) {
        native.call(DOWN_UP,data.code.toLong(),SystemClock.uptimeMillis())
        jobs.remove(data.code)?.cancelJobs()
        val now=SystemClock.uptimeMillis()
        keyEventReceiver?.onInputKeyDown(data)
        native.call(ACK_DOWN,data.code.toLong(),now)
        keyEventReceiver?.onInputKeyUp(data)
        native.call(ACK_UP,data.code.toLong(),now)
    }
    @Synchronized fun sendCancel(data: KeyData) {
        val plan=native.call(CANCEL,data.code.toLong(),SystemClock.uptimeMillis())
        jobs.remove(data.code)?.cancelJobs()
        if (plan.isNotEmpty()) keyEventReceiver?.onInputKeyCancel(data)
    }
    fun facts(code: Int): LongArray = native.call(FACTS,code.toLong(),SystemClock.uptimeMillis(),doubleTapTimeout)
    fun isPressed(code: Int)=facts(code)[0]!=0L
    fun isAnyPressed()=facts(0)[1]!=0L
    fun isConsecutiveDown(data: KeyData)=facts(data.code)[2]!=0L
    fun isConsecutiveUp(data: KeyData)=facts(data.code)[3]!=0L
    fun isUninterruptedEventSequence(data: KeyData)=facts(data.code)[4]!=0L
    fun isRepeatable(data: KeyData)=facts(data.code)[5]!=0L
    fun isRepeatableCodeLastDown()=facts(0)[6]!=0L
    @Synchronized fun close() {keyEventReceiver=null;scope.cancel();jobs.clear();native.close()}
    data class PressedKeyInfo(val eventTimeDown: Long, var job: Job?=null) {
        fun cancelJobs() {job?.cancel()}
    }
}
interface InputKeyEventReceiver {
    fun onInputKeyDown(data: KeyData)
    fun onInputKeyUp(data: KeyData)
    fun onInputKeyRepeat(data: KeyData)
    fun onInputKeyCancel(data: KeyData)
}
