package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsRules
import com.jtn0123.audiowhisper.rust.SettingsValue as V
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.core.SubtypeLayoutMap
import dev.patrickgold.florisboard.ime.core.SubtypeNlpProviderMap
import dev.patrickgold.florisboard.ime.media.emoji.Emoji
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHistory
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName

/** Android DTO projection only. Rust supplies every field, including nested defaults. */
object SettingsCodecs {
    fun subtypes(raw: String) = SettingsRules.decode("subtypes", raw).array().map(::subtype)
    fun subtypes(values: List<Subtype>) = SettingsRules.encode("subtypes", V.Array(values.map(::value)))
    fun subtype(raw: String) = subtype(SettingsRules.decode("subtype", raw))
    fun subtype(value: Subtype) = SettingsRules.encode("subtype", value(value))
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
    fun history(raw: String): EmojiHistory {
        val v = SettingsRules.decode("history", raw)
        fun emojis(key: String) = v[key].array().map { Emoji(it.text(), "", emptyList()) }
        return EmojiHistory(emojis("pinned"), emojis("recent"))
    }
    fun history(h: EmojiHistory) = SettingsRules.encode("history", V.obj(
        "pinned" to V.Array(h.pinned.map { V.Text(it.value) }), "recent" to V.Array(h.recent.map { V.Text(it.value) })))
    fun arrangement(raw: String): QuickActionArrangement {
        val v = SettingsRules.decode("arrangement",raw)
        return QuickActionArrangement(action(v["stickyAction"]),v["dynamicActions"].array().map { checkNotNull(action(it)) },
            v["hiddenActions"].array().map { checkNotNull(action(it)) })
    }
    fun arrangement(a: QuickActionArrangement) = SettingsRules.encode("arrangement", V.obj(
        "stickyAction" to value(a.stickyAction), "dynamicActions" to V.Array(a.dynamicActions.map(::value)),
        "hiddenActions" to V.Array(a.hiddenActions.map(::value))))
    private fun action(v: V): QuickAction? = when {
        v == V.Null -> null
        v["$"].text() == "insert_text" -> QuickAction.InsertText(v["data"].text())
        else -> QuickAction.InsertKey(SettingsKeys.key(v["data"]) as KeyData)
    }
    private fun value(a: QuickAction?): V = when(a) {
        null -> V.Null
        is QuickAction.InsertKey -> V.obj("$" to V.Text("insert_key"),"data" to SettingsKeys.value(a.data))
        is QuickAction.InsertText -> V.obj("$" to V.Text("insert_text"),"data" to V.Text(a.data))
    }
}
