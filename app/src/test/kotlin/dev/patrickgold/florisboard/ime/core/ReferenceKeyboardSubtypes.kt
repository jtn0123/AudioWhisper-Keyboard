package dev.patrickgold.florisboard.ime.core

import com.jtn0123.audiowhisper.rust.SettingsRules
import com.jtn0123.audiowhisper.rust.SettingsValue as V
import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.florisboard.lib.kotlin.collectLatestIn

/**
 * Main's `SubtypeManager` at 61f1513fb, frozen for the subtype trace: Kotlin decoded the stored
 * list into `Subtype`s, asked Rust's settings plan to repair it and to choose the active subtype,
 * and looked a subtype's currency set up in the loaded map. Only its settings moved behind
 * [SubtypeStore] and its scope and currency sets became parameters; `getSubtypeById`'s Rust index
 * op, since retired, is its Kotlin equivalent.
 */
internal class ReferenceKeyboardSubtypes(
    private val store: SubtypeStore,
    private val currencySets: () -> Map<ExtensionComponentName, CurrencySet>?,
    private val scope: CoroutineScope,
) {
    private val _subtypesFlow = MutableStateFlow(listOf<Subtype>())
    val subtypesFlow = _subtypesFlow.asStateFlow()
    var subtypes
        get() = subtypesFlow.value
        private set(v) { _subtypesFlow.value = v }
    private val _activeSubtypeFlow = MutableStateFlow(Subtype.DEFAULT)
    val activeSubtypeFlow = _activeSubtypeFlow.asStateFlow()
    var activeSubtype
        get() = activeSubtypeFlow.value
        private set(v) { _activeSubtypeFlow.value = v }

    init {
        store.lists.collectLatestIn(scope) { raw ->
            val list = ReferenceSubtypeCodecs.subtypes(raw)
            val plan = ReferenceSubtypePlans.plan("load", list, store.activeId())
            subtypes = plan.list
            evaluateActiveSubtype(plan.list)
            if (plan.changed) persistNewSubtypeList(plan.list)
        }
    }
    private fun persistNewSubtypeList(list: List<Subtype>) = scope.launch {
        store.setList(ReferenceSubtypeCodecs.subtypes(list))
    }
    private fun evaluateActiveSubtype(list: List<Subtype>) = scope.launch {
        val plan = ReferenceSubtypePlans.plan("evaluate", list, store.activeId())
        if (plan.writeActive) store.setActiveId(plan.active)
        activeSubtype = plan.selected
    }
    fun getCurrencySet(subtypeToSearch: Subtype): CurrencySet =
        currencySets()?.get(subtypeToSearch.currencySet) ?: CurrencySet.Fallback
    fun getSubtypeById(id: Long): Subtype? = subtypes.firstOrNull { it.id == id }
    fun switchToPrevSubtype() = switchSubtype("previous")
    fun switchToNextSubtype() = switchSubtype("next")
    private fun switchSubtype(direction: String) = scope.launch {
        val plan = ReferenceSubtypePlans.plan(direction, subtypes, activeSubtype.id, selected = activeSubtype)
        store.setActiveId(plan.active)
        activeSubtype = plan.selected
    }
    fun switchToSubtypeById(id: Long) = scope.launch {
        val selected = getSubtypeById(id)
        if (selected != null) { activeSubtype = selected; store.setActiveId(id) }
    }
}

/** Main's `SettingsSubtypes` at 61f1513fb, frozen. */
private object ReferenceSubtypePlans {
    data class Plan(val changed: Boolean, val active: Long, val writeActive: Boolean, val selected: Subtype, val list: List<Subtype>)
    fun plan(op: String, list: List<Subtype>, active: Long, item: Subtype = Subtype.DEFAULT,
             time: Long = 0, target: Long = active, selected: Subtype = Subtype.DEFAULT, order: List<Int> = emptyList()): Plan {
        val result = SettingsRules.subtype(V.obj("op" to V.Text(op),"list" to V.Array(list.map(ReferenceSubtypeCodecs::value)),
            "active" to V.Number(active),"item" to ReferenceSubtypeCodecs.value(item),"time" to V.Number(time),
            "target" to V.Number(target),"selected" to ReferenceSubtypeCodecs.value(selected),"order" to V.Array(order.map { V.Number(it.toLong()) })))
        return Plan(result["changed"].number() != 0L,result["active"].number(),result["writeActive"].number() != 0L,
            ReferenceSubtypeCodecs.subtype(result["selected"]),result["list"].array().map(ReferenceSubtypeCodecs::subtype))
    }
}

/** Main's `SettingsCodecs` subtype projection at 61f1513fb, frozen. */
internal object ReferenceSubtypeCodecs {
    fun subtypes(raw: String) = SettingsRules.decode("subtypes", raw).array().map(::subtype)
    fun subtypes(values: List<Subtype>) = SettingsRules.encode("subtypes", V.Array(values.map(::value)))
    fun subtype(v: V): Subtype {
        val m = v["layoutMap"]
        fun component(key: String) = ExtensionComponentName.from(v[key].text())
        fun layout(key: String) = ExtensionComponentName.from(m[key].text())
        return Subtype(v["id"].number(), FlorisLocale.fromTag(v["primaryLocale"].text()),
            v["secondaryLocales"].array().map { FlorisLocale.fromTag(it.text()) },
            SubtypeNlpProviderMap(v["nlpProviders"]["spelling"].text(), v["nlpProviders"]["suggestion"].text()),
            component("composer"), component("currencySet"), component("punctuationRule"), component("popupMapping"),
            SubtypeLayoutMap(layout("characters"),layout("symbols"),layout("symbols2"),layout("numeric"),
                layout("numericAdvanced"),layout("numericRow"),layout("phone"),layout("phone2")))
    }
    fun value(s: Subtype): V = V.obj(
        "id" to V.Number(s.id), "primaryLocale" to V.Text(s.primaryLocale.languageTag()),
        "secondaryLocales" to V.Array(s.secondaryLocales.map { V.Text(it.languageTag()) }),
        "nlpProviders" to V.obj("spelling" to V.Text(s.nlpProviders.spelling), "suggestion" to V.Text(s.nlpProviders.suggestion)),
        "composer" to V.Text(s.composer.toString()), "currencySet" to V.Text(s.currencySet.toString()),
        "punctuationRule" to V.Text(s.punctuationRule.toString()), "popupMapping" to V.Text(s.popupMapping.toString()),
        "layoutMap" to V.obj("characters" to V.Text(s.layoutMap.characters.toString()), "symbols" to V.Text(s.layoutMap.symbols.toString()),
            "symbols2" to V.Text(s.layoutMap.symbols2.toString()), "numeric" to V.Text(s.layoutMap.numeric.toString()),
            "numericAdvanced" to V.Text(s.layoutMap.numericAdvanced.toString()), "numericRow" to V.Text(s.layoutMap.numericRow.toString()),
            "phone" to V.Text(s.layoutMap.phone.toString()), "phone2" to V.Text(s.layoutMap.phone2.toString())),
    )
}
