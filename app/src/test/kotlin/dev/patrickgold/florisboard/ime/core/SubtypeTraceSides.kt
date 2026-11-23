package dev.patrickgold.florisboard.ime.core

import com.jtn0123.audiowhisper.render.KeyGraphAssets
import com.jtn0123.audiowhisper.render.LayoutName
import com.jtn0123.audiowhisper.render.ReferenceSubtypeBuildRow
import com.jtn0123.audiowhisper.render.keyGraphBuild
import com.jtn0123.audiowhisper.render.subtypeFiles
import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel

/** What the trace reads of one keyboard's subtypes, and asks of them. */
internal interface TracedSubtypes {
    val list: List<Subtype>
    val active: Subtype
    fun next()
    fun previous()
    fun select(id: Long)
    fun currencySet(subtype: Subtype): CurrencySet

    /** The key graph's cache key for the active subtype. */
    fun cacheKey(): Int

    /** The active subtype's `KeyGraphBuild` row for [mode]. */
    fun row(mode: KeyboardMode, numberRow: Boolean, numberHints: Boolean): Array<String>
}

/** One keyboard in the trace: its subtypes over its own settings, scope and dispatcher, and what failed. */
internal class TraceSide(
    raw: String, id: Long, sets: () -> Map<ExtensionComponentName, CurrencySet>?,
    make: (SubtypeStore, () -> Map<ExtensionComponentName, CurrencySet>?, CoroutineScope) -> TracedSubtypes,
) {
    val store = TraceStore(raw, id)
    val dispatcher = QueueDispatcher()
    val failures = mutableListOf<String>()
    private val scope = CoroutineScope(dispatcher + Job() + CoroutineExceptionHandler { _, error ->
        failures += error.javaClass.name
    })
    val subtypes = make(store, sets, scope)

    /** The process ends: nothing it was doing runs. */
    fun end() {
        scope.cancel()
        dispatcher.clear()
    }
}

/** The files the key graph reads, from the bundled layouts and popup mappings. */
private val namedModifier = { name: LayoutName -> KeyGraphAssets.layouts[name]?.first?.modifier }
private val layoutText = { name: LayoutName -> KeyGraphAssets.layouts[name]?.second }
private val mappingText = { name: ExtensionComponentName -> KeyGraphAssets.mappings[name] }

/** Main's subtypes (frozen in ReferenceKeyboardSubtypes.kt) and its key graph build row. */
internal class MainSubtypes(store: SubtypeStore, sets: () -> Map<ExtensionComponentName, CurrencySet>?,
                            scope: CoroutineScope) : TracedSubtypes {
    private val manager = ReferenceKeyboardSubtypes(store, sets, scope)
    override val list get() = manager.subtypes
    override val active get() = manager.activeSubtype
    override fun next() { manager.switchToNextSubtype() }
    override fun previous() { manager.switchToPrevSubtype() }
    override fun select(id: Long) { manager.switchToSubtypeById(id) }
    override fun currencySet(subtype: Subtype) = manager.getCurrencySet(subtype)
    override fun cacheKey() = ReferenceSubtypeBuildRow.cacheKey(manager.activeSubtype)
    override fun row(mode: KeyboardMode, numberRow: Boolean, numberHints: Boolean) = ReferenceSubtypeBuildRow.row(
        mode, numberRow, numberHints, manager.activeSubtype, namedModifier, layoutText, mappingText)
}

/** The branch's subtypes: `SubtypeManager` over Rust's session, and the row `RustKeyGraphs` builds from its facts. */
internal class BranchSubtypes(store: SubtypeStore, sets: () -> Map<ExtensionComponentName, CurrencySet>?,
                              scope: CoroutineScope) : TracedSubtypes {
    val manager = SubtypeManager(store, sets, scope)
    override val list get() = manager.subtypes
    override val active get() = manager.activeSubtype
    override fun next() { manager.switchToNextSubtype() }
    override fun previous() { manager.switchToPrevSubtype() }
    override fun select(id: Long) { manager.switchToSubtypeById(id) }
    override fun currencySet(subtype: Subtype) = manager.getCurrencySet(subtype)
    override fun cacheKey() = manager.active.facts.key
    override fun row(mode: KeyboardMode, numberRow: Boolean, numberHints: Boolean) = keyGraphBuild(
        mode, numberRow, numberHints, subtypeFiles(manager.active.facts, namedModifier, layoutText, mappingText))
}
