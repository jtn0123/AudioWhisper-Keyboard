package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsValue as V
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.CaseSelector
import dev.patrickgold.florisboard.ime.keyboard.ShiftStateSelector
import dev.patrickgold.florisboard.ime.keyboard.VariationSelector
import dev.patrickgold.florisboard.ime.keyboard.LayoutDirectionSelector
import dev.patrickgold.florisboard.ime.keyboard.CharWidthSelector
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.AutoTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.MultiTextKeyData

/** Type construction/deconstruction for the existing Android renderer's DTOs. */
internal object SettingsKeys {
    fun key(v: V): AbstractKeyData {
        fun child(name: String) = v[name].let { if(it == V.Null) null else key(it) }
        fun required(name: String) = checkNotNull(child(name))
        return when(v["$"].text()) {
            "case_selector" -> CaseSelector(required("lower"),required("upper"))
            "shift_state_selector" -> ShiftStateSelector(child("unshifted"),child("shifted"),child("shiftedManual"),child("shiftedAutomatic"),child("capsLock"))
            "variation_selector" -> VariationSelector(child("default"),child("email"),child("uri"),child("normal"),child("password"))
            "layout_direction_selector" -> LayoutDirectionSelector(required("ltr"),required("rtl"))
            "char_width_selector" -> CharWidthSelector(child("full"),child("half"))
            else -> {
                val type=KeyType.fromString(v["type"].text())
                val label=v["label"].text();val group=v["groupId"].number().toInt()
                val popup=v["popup"].let { p -> if(p == V.Null) null else PopupSet(
                    p["main"].let { if(it == V.Null) null else key(it) },p["relevant"].array().map(::key)) }
                when(v["$"].text()) {
                    "multi_text_key" -> MultiTextKeyData(type,v["codePoints"].array().map { it.number().toInt() }.toIntArray(),label,group,popup)
                    "auto_text_key" -> AutoTextKeyData(type,v["code"].number().toInt(),label,group,popup)
                    else -> TextKeyData(type,v["code"].number().toInt(),label,group,popup)
                }
            }
        }
    }
    fun value(k: AbstractKeyData?): V {
        if(k == null) return V.Null
        fun fields(type: String, vararg pairs: Pair<String,V>) = V.obj("$" to V.Text(type),*pairs)
        return when(k) {
            is CaseSelector -> fields("case_selector","lower" to value(k.lower),"upper" to value(k.upper))
            is ShiftStateSelector -> fields("shift_state_selector","unshifted" to value(k.unshifted),"shifted" to value(k.shifted),
                "shiftedManual" to value(k.shiftedManual),"shiftedAutomatic" to value(k.shiftedAutomatic),"capsLock" to value(k.capsLock))
            is VariationSelector -> fields("variation_selector","default" to value(k.default),"email" to value(k.email),"uri" to value(k.uri),"normal" to value(k.normal),"password" to value(k.password))
            is LayoutDirectionSelector -> fields("layout_direction_selector","ltr" to value(k.ltr),"rtl" to value(k.rtl))
            is CharWidthSelector -> fields("char_width_selector","full" to value(k.full),"half" to value(k.half))
            is KeyData -> {
                val tag = when(k) {is AutoTextKeyData -> "auto_text_key";is MultiTextKeyData -> "multi_text_key";else -> "text_key"}
                val code = if(k is MultiTextKeyData) "codePoints" to V.Array(k.codePoints.map { V.Number(it.toLong()) }) else "code" to V.Number(k.code.toLong())
                fields(tag,"type" to V.Text(k.type.toString()),code,"label" to V.Text(k.label),"groupId" to V.Number(k.groupId.toLong()),
                    "popup" to (k.popup?.let { V.obj("main" to value(it.main),"relevant" to V.Array(it.relevant.map(::value))) } ?: V.Null))
            }
            else -> error("Unknown platform key DTO")
        }
    }
}
