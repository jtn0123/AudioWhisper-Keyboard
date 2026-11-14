package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsRules
import com.jtn0123.audiowhisper.rust.SettingsValue as V
import dev.patrickgold.florisboard.ime.core.Subtype

object SettingsSubtypes {
    data class Plan(val changed: Boolean, val active: Long, val writeActive: Boolean, val selected: Subtype, val list: List<Subtype>)
    fun plan(op: String, list: List<Subtype>, active: Long, item: Subtype = Subtype.DEFAULT,
             time: Long = 0, target: Long = active, selected: Subtype = Subtype.DEFAULT, order: List<Int> = emptyList()): Plan {
        val result = SettingsRules.subtype(V.obj("op" to V.Text(op),"list" to V.Array(list.map(SettingsCodecs::value)),
            "active" to V.Number(active),"item" to SettingsCodecs.value(item),"time" to V.Number(time),
            "target" to V.Number(target),"selected" to SettingsCodecs.value(selected),"order" to V.Array(order.map { V.Number(it.toLong()) })))
        return Plan(result["changed"].number() != 0L,result["active"].number(),result["writeActive"].number() != 0L,
            SettingsCodecs.subtype(result["selected"]),result["list"].array().map(SettingsCodecs::subtype))
    }
}
