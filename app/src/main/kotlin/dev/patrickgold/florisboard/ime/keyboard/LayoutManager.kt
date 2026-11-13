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

import android.content.Context
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.popup.PopupMapping
import dev.patrickgold.florisboard.ime.popup.PopupMappingComponent
import dev.patrickgold.florisboard.ime.text.keyboard.TextKey
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboard
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogWarning
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.lib.io.ZipUtils
import dev.patrickgold.florisboard.lib.io.loadJsonAsset
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.cancel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.florisboard.lib.kotlin.DeferredResult
import org.florisboard.lib.kotlin.runCatchingAsync

private data class LTN(
    val type: LayoutType,
    val name: ExtensionComponentName,
)

data class CachedLayout(
    val type: LayoutType,
    val name: ExtensionComponentName,
    val meta: LayoutArrangementComponent,
    val arrangement: LayoutArrangement,
)

private data class CachedPopupMapping(
    val name: ExtensionComponentName,
    val meta: PopupMappingComponent,
    val mapping: PopupMapping,
)

/**
 * Class which manages layout loading and caching.
 */
class LayoutManager(context: Context) {
    private val prefs by FlorisPreferenceStore
    private val appContext by context.appContext()
    private val extensionManager by context.extensionManager()
    private val keyboardManager by context.keyboardManager()

    private val layoutCache: HashMap<LTN, DeferredResult<CachedLayout>> = hashMapOf()
    private val layoutCacheGuard: Mutex = Mutex(locked = false)
    private val popupMappingCache: HashMap<ExtensionComponentName, DeferredResult<CachedPopupMapping>> = hashMapOf()
    private val popupMappingCacheGuard: Mutex = Mutex(locked = false)
    private val ioScope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    /**
     * Loads the layout for the specified type and name.
     *
     * @return A deferred result for a layout.
     */
    private fun loadLayoutAsync(ltn: LTN?, allowNullLTN: Boolean) = ioScope.runCatchingAsync {
        if (!allowNullLTN) {
            requireNotNull(ltn) { "Invalid argument value for 'ltn': null" }
        }
        if (ltn == null) {
            return@runCatchingAsync null
        }
        layoutCacheGuard.withLock {
            val cached = layoutCache[ltn]
            if (cached != null) {
                return@withLock cached
            } else {
                val meta = keyboardManager.resources.layouts.value?.get(ltn.type)?.get(ltn.name)
                    ?: error("No indexed entry found for ${ltn.type} - ${ltn.name}")
                val ext = extensionManager.getExtensionById(ltn.name.extensionId)
                    ?: error("Extension ${ltn.name.extensionId} not found")
                val path = meta.arrangementFile(ltn.type)
                val layout = async {
                    runCatching {
                        val jsonStr = ZipUtils.readFileFromArchive(appContext, ext.sourceRef!!, path).getOrThrow()
                        val arrangement = loadJsonAsset<LayoutArrangement>(jsonStr).getOrThrow()
                        CachedLayout(ltn.type, ltn.name, meta, arrangement)
                    }
                }
                layoutCache[ltn] = layout
                return@withLock layout
            }
        }.await().getOrThrow()
    }

    private fun loadPopupMappingAsync(subtype: Subtype? = null) = ioScope.runCatchingAsync {
        val name = subtype?.popupMapping ?: extCorePopupMapping("default")
        popupMappingCacheGuard.withLock {
            val cached = popupMappingCache[name]
            if (cached != null) {
                return@withLock cached
            } else {
                val meta = keyboardManager.resources.popupMappings.value?.get(name)
                    ?: error("No indexed entry found for $name")
                val ext = extensionManager.getExtensionById(name.extensionId)
                    ?: error("Extension ${name.extensionId} not found")
                val path = meta.mappingFile()
                val popupMapping = async {
                    runCatching {
                        val jsonStr = ZipUtils.readFileFromArchive(appContext, ext.sourceRef!!, path).getOrThrow()
                        val mapping = loadJsonAsset<PopupMapping>(jsonStr).getOrThrow()
                        CachedPopupMapping(name, meta, mapping)
                    }
                }
                popupMappingCache[name] = popupMapping
                return@withLock popupMapping
            }
        }.await().getOrThrow()
    }

    /**
     * Merges the specified layouts (LTNs) and returns the computed layout.
     * The computed layout may looks like this:
     *   e e e e e e e e e e      e = extension
     *   c c c c c c c c c c      c = main
     *    c c c c c c c c c       m = mod
     *   m c c c c c c c c m
     *   m m m m m m m m m m
     *
     * @param keyboardMode The keyboard mode for the returning [TextKeyboard].
     * @param subtype The subtype used for populating the extended popups.
     * @param main The main layout type and name.
     * @param modifier The modifier (mod) layout type and name.
     * @param extension The extension layout type and name.
     * @return a [TextKeyboard] object, regardless of the specified LTNs or errors.
     */
    private suspend fun mergeLayouts(
        keyboardMode: KeyboardMode,
        subtype: Subtype,
        main: LTN? = null,
        modifier: LTN? = null,
        extension: LTN? = null,
    ): TextKeyboard {
        val extendedPopupsDefault = loadPopupMappingAsync()
        val extendedPopups = loadPopupMappingAsync(subtype)

        val mainLayoutResult = loadLayoutAsync(main, allowNullLTN = false).await()
        val mainLayout = mainLayoutResult.onFailure {
            flogWarning { "$keyboardMode - main - $it" }
        }.getOrNull()
        val modifierToLoad = if (mainLayout?.meta?.modifier != null) {
            val layoutType = LayoutAssembly.modifier(mainLayout.type)
            LTN(layoutType, mainLayout.meta.modifier)
        } else {
            modifier
        }
        val modifierLayoutResult = loadLayoutAsync(modifierToLoad, allowNullLTN = true).await()
        val modifierLayout = modifierLayoutResult.onFailure {
            flogWarning { "$keyboardMode - mod - $it" }
        }.getOrNull()
        val extensionLayoutResult = loadLayoutAsync(extension, allowNullLTN = true).await()
        val extensionLayout = extensionLayoutResult.onFailure {
            flogWarning { "$keyboardMode - ext - $it" }
        }.getOrNull()

        val rows = LayoutAssembly.merge(mainLayout?.arrangement, modifierLayout?.arrangement, extensionLayout?.arrangement)
        val computedArrangement = rows.map { row -> row.map(::TextKey).toTypedArray() }
        if (keyboardMode == KeyboardMode.CHARACTERS && rows.isNotEmpty()) {
            val symbols = computeKeyboardAsync(KeyboardMode.SYMBOLS, subtype).await().arrangement
                .map { row -> row.map { it.data } }
            val hints = LayoutAssembly.hints(keyboardMode, prefs.keyboard.hintedNumberRowEnabled.get(),
                rows, symbols, DefaultComputingEvaluator).iterator()
            computedArrangement.forEach { row -> row.forEach { key ->
                val (number, symbol) = hints.next()
                key.computedNumberHint = number
                key.computedSymbolHint = symbol
            } }
        }

        val array = Array(computedArrangement.size) { computedArrangement[it] }
        return TextKeyboard(
            arrangement = array,
            mode = keyboardMode,
            extendedPopupMapping = extendedPopups.await().onFailure {
                flogWarning(LogTopic.LAYOUT_MANAGER) { it.toString() }
            }.getOrNull()?.mapping,
            extendedPopupMappingDefault = extendedPopupsDefault.await().onFailure {
                flogWarning(LogTopic.LAYOUT_MANAGER) { it.toString() }
            }.getOrNull()?.mapping
        )
    }

    /**
     * Computes a layout for [keyboardMode] based on the given [subtype] and returns it.
     *
     * @param keyboardMode The keyboard mode for which the layout should be computed.
     * @param subtype The subtype which localizes the computed layout.
     */
    fun computeKeyboardAsync(
        keyboardMode: KeyboardMode,
        subtype: Subtype,
    ): Deferred<TextKeyboard> = ioScope.async {
        val (main, modifier, extension) = LayoutAssembly.plan(keyboardMode, prefs.keyboard.numberRow.get(), subtype.layoutMap::get)
            .map { request -> request?.let { (type, name) -> LTN(type, name) } }

        return@async mergeLayouts(keyboardMode, subtype, main, modifier, extension)
    }

    /**
     * Called when the application is destroyed. Used to cancel any pending coroutines.
     */
    fun onDestroy() {
        ioScope.cancel()
    }
}
