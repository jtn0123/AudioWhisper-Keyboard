package com.jtn0123.audiowhisper.settingsui

import androidx.activity.ComponentActivity
import androidx.lifecycle.lifecycleScope
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import com.jtn0123.audiowhisper.settings.SettingsSubtypes
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.keyboard.LayoutType
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.FlorisLocale
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.florisboard.lib.android.showLongToast

/** What the native language screens need from the keyboard, as one `LANGUAGES` report: its locales named as Java
 * names them, its layouts, composers, currency sets, popup mappings, presets and language packs. Also deletes a
 * language pack, which needs the extension manager. Rust edits and writes the subtype list itself. */
internal class SettingsLanguages(private val activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    private val keyboardManager by activity.keyboardManager()
    private val extensionManager by activity.extensionManager()
    private val prefs by FlorisPreferenceStore
    private var wanted = false
    init {
        // Once asked, a later change (resources parsed, a pack imported or deleted) reaches the screens too.
        keyboardManager.resources.anyChanged.observe(activity) { if (wanted) load() }
        extensionManager.languagePacks.observe(activity) { if (wanted) load() }
    }
    /** A `languages` or `deleteLanguagePack` effect. */
    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiLanguageEffect.LOAD -> { wanted = true; load() }
            SettingsUiLanguageEffect.DELETE_PACK -> delete(p[1])
        }
    }
    private fun load() {
        activity.lifecycleScope.launch { report(withContext(Dispatchers.Default) { rows() }) }
    }
    /** As LanguagePackManagerScreen deleted: a failure is a long toast, and the list is read again either way. */
    private fun delete(id: String) {
        val ext = extensionManager.getExtensionById(id) ?: return load()
        runCatching { extensionManager.delete(ext) }.onFailure { error ->
            activity.lifecycleScope.launch {
                activity.showLongToast(R.string.error__snackbar_message, "error_message" to error.localizedMessage)
            }
        }
        load()
    }
    private fun rows(): Array<String> {
        val out = mutableListOf<String>()
        val raw = prefs.localization.subtypes.get()
        val subtypes = loadedSubtypes(raw, prefs.localization.activeSubtypeId.get())
        out += subtypesRow(raw, subtypes)
        for (locale in FlorisLocale.extendedAvailableLocales(activity)) out += languageRow(SettingsUiLanguageKind.LOCALE, locale)
        val system = activity.resources.configuration.locales
        for (n in 0 until system.size()) out += languageRow(SettingsUiLanguageKind.SYSTEM, FlorisLocale.from(system.get(n)))
        val resources = keyboardManager.resources
        val presets = resources.subtypePresets.value.orEmpty()
        for (locale in (subtypes.map { it.primaryLocale } + presets.map { it.locale }).distinct()) {
            out += languageRow(SettingsUiLanguageKind.NAME, locale)
        }
        for (type in LayoutType.entries) for ((name, layout) in resources.layouts.value?.get(type).orEmpty()) {
            out += componentRow(SettingsUiLanguageKind.LAYOUT, name.toString(), layout.label, type.id)
        }
        for ((name, it) in resources.composers.value.orEmpty()) out += componentRow(SettingsUiLanguageKind.COMPOSER, name.toString(), it.label)
        for ((name, it) in resources.currencySets.value.orEmpty()) out += componentRow(SettingsUiLanguageKind.CURRENCY, name.toString(), it.label)
        for ((name, it) in resources.popupMappings.value.orEmpty()) out += componentRow(SettingsUiLanguageKind.POPUP, name.toString(), it.label)
        for (preset in presets) out += presetRow(preset.locale, preset.toSubtype())
        // As LanguagePackManagerScreen grouped them: by extension id, its items sorted by label.
        for ((id, items) in extensionManager.languagePacks.value.orEmpty().associate { it.meta.id to it.items }) {
            val ext = extensionManager.getExtensionById(id) ?: continue
            out += componentRow(SettingsUiLanguageKind.PACK, id, ext.meta.title, extensionManager.canDelete(ext).toString())
            for (item in items.sortedBy { it.label }) out += componentRow(SettingsUiLanguageKind.PACK_ITEM, id, item.label)
        }
        return out.toTypedArray()
    }
}

/** The subtypes as SubtypeManager reads `raw`: its `load`, which replaces any it cannot use. */
internal fun loadedSubtypes(raw: String, active: Long) = SettingsSubtypes.plan("load", SettingsCodecs.subtypes(raw), active).list

/** The report's first row: the stored text, and the list as SubtypeManager read it. */
internal fun subtypesRow(raw: String, loaded: List<Subtype>) =
    componentRow(SettingsUiLanguageKind.SUBTYPES, raw, detail = SettingsCodecs.subtypes(loaded))

/** A locale of `kind`, named as the Compose screens named it: in the app's language, its own and English. */
internal fun languageRow(kind: String, locale: FlorisLocale) = SettingsUiLanguage.row(SettingsUiLanguage.KIND to kind,
    SettingsUiLanguage.ID to locale.languageTag(), SettingsUiLanguage.LABEL to locale.displayName(),
    SettingsUiLanguage.NATIVE to locale.displayName(locale), SettingsUiLanguage.ENGLISH to locale.displayName(FlorisLocale.ENGLISH),
    SettingsUiLanguage.DETAIL to locale.localeTag(), SettingsUiLanguage.BASE to locale.base.toString(),
    SettingsUiLanguage.LANGUAGE to locale.language)

/** A component, pack or pack item: its id, label and one more value. */
internal fun componentRow(kind: String, id: String, label: String = "", detail: String = "") = SettingsUiLanguage.row(
    SettingsUiLanguage.KIND to kind, SettingsUiLanguage.ID to id, SettingsUiLanguage.LABEL to label, SettingsUiLanguage.NATIVE to "",
    SettingsUiLanguage.ENGLISH to "", SettingsUiLanguage.DETAIL to detail, SettingsUiLanguage.BASE to "", SettingsUiLanguage.LANGUAGE to "")

/** A preset: the subtype it makes, and the locale's base and language, which a picked locale is matched by. */
internal fun presetRow(locale: FlorisLocale, subtype: Subtype) = SettingsUiLanguage.row(
    SettingsUiLanguage.KIND to SettingsUiLanguageKind.PRESET, SettingsUiLanguage.ID to locale.languageTag(), SettingsUiLanguage.LABEL to "",
    SettingsUiLanguage.NATIVE to "", SettingsUiLanguage.ENGLISH to "", SettingsUiLanguage.DETAIL to SettingsCodecs.subtype(subtype),
    SettingsUiLanguage.BASE to locale.base.toString(), SettingsUiLanguage.LANGUAGE to locale.language)
