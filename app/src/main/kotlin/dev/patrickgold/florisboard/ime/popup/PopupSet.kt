/*
 * Copyright (C) 2020-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.popup

import com.jtn0123.audiowhisper.rust.PopupOrder
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import kotlinx.serialization.Serializable

/**
 * A popup set for a single key. This set describes, if the key has a [main] character and other [relevant] popups.
 *
 * Note that a hint character cannot and should not be set in a json extended popup file, rather it
 * should only be dynamically set by the LayoutManager.
 *
 * The order in which these defined popups will be shown depends on the current [KeyHintConfiguration].
 */
@Serializable
open class PopupSet<T : AbstractKeyData>(
    open val main: T? = null,
    open val relevant: List<T> = listOf()
) {
    private val popupKeys: PopupKeys<T> by lazy {
        PopupKeys(null, listOfNotNull(main), relevant)
    }

    open fun getPopupKeys(keyHintConfiguration: KeyHintConfiguration): PopupKeys<T> {
        return popupKeys
    }
}

@Suppress("UNCHECKED_CAST")
class MutablePopupSet<T : AbstractKeyData>(
    override var main: T? = null,
    override val relevant: ArrayList<T> = arrayListOf(),
    var symbolHint: T? = null,
    var numberHint: T? = null,
    private val symbolPopups: ArrayList<T> = arrayListOf(),
    private val numberPopups: ArrayList<T> = arrayListOf(),
    private val configCache: MutableMap<KeyHintConfiguration, PopupKeys<T>> = mutableMapOf()
) : PopupSet<T>(main, relevant) {

    fun clear() {
        symbolHint = null
        numberHint = null
        main = null
        relevant.clear()
        symbolPopups.clear()
        numberPopups.clear()
        configCache.clear()
    }

    override fun getPopupKeys(keyHintConfiguration: KeyHintConfiguration): PopupKeys<T> {
        return configCache.getOrPut(keyHintConfiguration) {
            initPopupList(keyHintConfiguration)
        }
    }

    // AudioWhisper: which keys go where for the hint settings is decided in Rust
    // (lib/rust/crates/popup/src/order.rs), which answers with the keys' numbers.
    private fun initPopupList(keyHintConfiguration: KeyHintConfiguration): PopupKeys<T> {
        val localMain = main
        val localRelevant = relevant
        val localSymbolHint = symbolHint
        val localNumberHint = numberHint
        val order = PopupOrder.order(
            main = localMain != null,
            relevant = localRelevant.size,
            symbolHint = localSymbolHint != null,
            numberHint = localNumberHint != null,
            symbolPopups = symbolPopups.size,
            numberPopups = numberPopups.size,
            symbolMode = mode(keyHintConfiguration.symbolHintMode),
            numberMode = mode(keyHintConfiguration.numberHintMode),
            mergeHintPopups = keyHintConfiguration.mergeHintPopups,
        )
        val key = { number: Int ->
            val listed = number - PopupOrder.FIRST_LISTED
            val symbolPopup = listed - localRelevant.size
            val numberPopup = symbolPopup - symbolPopups.size
            when {
                number == PopupOrder.MAIN -> checkNotNull(localMain)
                number == PopupOrder.SYMBOL_HINT -> checkNotNull(localSymbolHint)
                number == PopupOrder.NUMBER_HINT -> checkNotNull(localNumberHint)
                symbolPopup < 0 -> localRelevant[listed]
                numberPopup < 0 -> symbolPopups[symbolPopup]
                else -> numberPopups[numberPopup]
            }
        }
        return PopupKeys(
            hint = if (order.hint == PopupOrder.NO_HINT) null else key(order.hint),
            prioritized = order.prioritized.map(key),
            // Where Kotlin handed over one of the set's own lists, still that list: keys merged
            // later show in it too.
            other = when (order.ownList) {
                PopupOrder.OWN_RELEVANT -> localRelevant
                PopupOrder.OWN_SYMBOL_POPUPS -> symbolPopups
                PopupOrder.OWN_NUMBER_POPUPS -> numberPopups
                else -> order.other.map(key)
            },
        )
    }

    private fun mode(mode: KeyHintMode) = when (mode) {
        KeyHintMode.DISABLED -> PopupOrder.Mode.DISABLED
        KeyHintMode.HINT_PRIORITY -> PopupOrder.Mode.HINT_PRIORITY
        KeyHintMode.ACCENT_PRIORITY -> PopupOrder.Mode.ACCENT_PRIORITY
        KeyHintMode.SMART_PRIORITY -> PopupOrder.Mode.SMART_PRIORITY
    }

    fun merge(other: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator) {
        mergeInternal(other, evaluator, relevant, true)
    }

    fun mergeSymbolHint(hintPopups: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator) {
        mergeInternal(hintPopups, evaluator, symbolPopups)
    }

    fun mergeNumberHint(hintPopups: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator) {
        mergeInternal(hintPopups, evaluator, numberPopups)
    }

    // AudioWhisper: where each merged key goes is decided in Rust (lib/rust/crates/popup/src/order.rs).
    private fun mergeInternal(other: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator, targetList: MutableList<T>, useMain: Boolean = false) {
        val computed = other.relevant.map { it.compute(evaluator) as? T }
        val otherMain = other.main?.let { it.compute(evaluator) as? T }
        val steps = PopupOrder.merge(
            computed = BooleanArray(computed.size) { computed[it] != null },
            mainComputed = otherMain != null,
            useMain = useMain,
            hasMain = main != null,
        )
        for (step in steps) {
            when {
                step == PopupOrder.SET_MAIN -> main = otherMain
                step < computed.size -> targetList.add(checkNotNull(computed[step]))
                else -> targetList.add(checkNotNull(otherMain))
            }
        }
    }
}

/**
 * A fully configured collection of popup keys. It contains a list of keys to be prioritized
 * during rendering (ordered by relevance descending) by showing those keys close to the
 * popup spawning point.
 *
 * The keys contain a separate [hint] key to ease rendering the hint label, but the hint, if
 * present, also occurs in the [prioritized] list.
 *
 * The popup keys can be accessed like an array with the addition that negative indexes defined
 * within this companion object are allowed (as long as the corresponding [prioritized] list
 * contains the corresponding amount of keys.
 */
class PopupKeys<T>(
    val hint: T?,
    val prioritized: List<T>,
    val other: List<T>
) : Collection<T> {
    companion object {
        const val FIRST_PRIORITIZED = -1
        const val SECOND_PRIORITIZED = -2
        const val THIRD_PRIORITIZED = -3
    }

    override val size: Int
        get() = prioritized.size + other.size

    override fun contains(element: T): Boolean {
        return prioritized.contains(element) || other.contains(element)
    }

    override fun containsAll(elements: Collection<T>): Boolean {
        return (prioritized + other).containsAll(elements)
    }

    override fun isEmpty(): Boolean {
        return prioritized.isEmpty() && other.isEmpty()
    }

    override fun iterator(): Iterator<T> {
        return (prioritized + other).listIterator()
    }

    fun getOrNull(index: Int): T? {
        if (index >= other.size || index < -prioritized.size) {
            return null
        }
        return when (index) {
            FIRST_PRIORITIZED -> prioritized[0]
            SECOND_PRIORITIZED -> prioritized[1]
            THIRD_PRIORITIZED -> prioritized[2]
            else -> other.getOrNull(index)
        }
    }

    operator fun get(index: Int): T {
        val item = getOrNull(index)
        if (item == null) {
            throw IndexOutOfBoundsException(
                "Specified index $index is not an valid entry in this PopupKeys!"
            )
        } else {
            return item
        }
    }
}
