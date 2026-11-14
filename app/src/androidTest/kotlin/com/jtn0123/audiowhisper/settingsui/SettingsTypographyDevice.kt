package com.jtn0123.audiowhisper.settingsui

import android.graphics.Typeface
import android.util.TypedValue
import android.widget.TextView
import org.json.JSONObject

/** Static UI-label metrics only. Never inspect or export editor/clipboard contents. */
internal class SettingsTypographyDevice(private val h: SettingsHarness) {
    fun run(report: JSONObject) {
        h.launch("settings/keyboard")
        h.main {
            val text = TextView(h.instrumentation.targetContext).apply {
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                typeface = Typeface.create("sans-serif", Typeface.NORMAL)
            }
            val label = "Try Rust drawing for text keys, suggestions"
            report.put("sp14_pixels", text.paint.textSize).put("prefix_width_no_spacing", text.paint.measureText(label))
            text.letterSpacing = .25f/14f
            report.put("prefix_width_material_spacing", text.paint.measureText(label))
                .put("spacing_em", text.letterSpacing)
        }
    }
}
