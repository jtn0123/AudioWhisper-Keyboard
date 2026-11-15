package com.jtn0123.audiowhisper.settingsui

import android.graphics.Paint
import android.graphics.Typeface
import android.graphics.text.TextRunShaper
import android.text.TextPaint
import android.util.TypedValue
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

/** Static UI-label metrics only. Never inspect or export editor/clipboard contents. */
internal class SettingsTypographyDevice(private val h: SettingsHarness) {
    private val label = "Try Rust drawing for text keys, suggestions"

    fun run(report: JSONObject) {
        h.main {
            val context = h.instrumentation.targetContext
            val sp14 = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, 14f, context.resources.displayMetrics)
            report.put("sp14_pixels", sp14)
            val runs = JSONArray()
            for (config in listOf("compose-static", "linear")) for (size in listOf(36f, 36.25f, 36.5f, sp14, 37f, 42f)) {
                for (spacing in listOf(0f, .25f / 14f)) runs.put(shape(config, size, spacing, size == sp14))
            }
            report.put("runs", runs)
            // Floor-size rule across sizes and weights; ink bitmaps show the drawn size.
            val sweep = JSONArray()
            for (weight in listOf(400, 500)) for (base in listOf(26, 31, 36, 42, 57, 63, 73)) for (fraction in listOf(0f, .5f, .75f, .99f)) {
                val p = paint("compose-static", base + fraction, 0f).apply { typeface = Typeface.create(Typeface.DEFAULT, weight, false) }
                val metrics = p.fontMetrics
                sweep.put(JSONObject().put("weight", weight).put("size", (base + fraction).toDouble())
                    .put("advance", p.getRunAdvance(label, 0, label.length, 0, label.length, false, label.length).toDouble())
                    .put("ascent", metrics.ascent.toDouble()).put("descent", metrics.descent.toDouble()))
            }
            report.put("sweep", sweep)
            val bitmaps = listOf(36f, sp14).map { size ->
                android.graphics.Bitmap.createBitmap(760, 64, android.graphics.Bitmap.Config.ARGB_8888).also {
                    it.eraseColor(android.graphics.Color.WHITE)
                    android.graphics.Canvas(it).drawText(label, 4f, 44f, paint("compose-static", size, 0f).apply { color = android.graphics.Color.BLACK })
                }
            }
            report.put("ink_36_equals_36_75", bitmaps[0].sameAs(bitmaps[1]))
            File(context.filesDir, "settings-typography-ink.png").outputStream().use { bitmaps[1].compress(android.graphics.Bitmap.CompressFormat.PNG, 100, it) }
            report.put("prefix_width_no_spacing", runs.getJSONObject(6).getDouble("advance"))
            report.put("prefix_width_material_spacing", runs.getJSONObject(7).getDouble("advance"))
        }
    }

    private fun paint(config: String, size: Float, spacing: Float) = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
        typeface = Typeface.create("sans-serif", Typeface.NORMAL)
        textSize = size
        letterSpacing = spacing
        // Compose TextMotion.Static: font hinting, no linear metrics or subpixel positioning.
        if (config == "linear") {
            flags = flags or Paint.LINEAR_TEXT_FLAG or Paint.SUBPIXEL_TEXT_FLAG
            hinting = Paint.HINTING_OFF
        } else {
            flags = flags and (Paint.LINEAR_TEXT_FLAG or Paint.SUBPIXEL_TEXT_FLAG).inv()
            hinting = Paint.HINTING_ON
        }
    }

    private fun shape(config: String, size: Float, spacing: Float, glyphs: Boolean): JSONObject {
        val paint = paint(config, size, spacing)
        val shaped = TextRunShaper.shapeTextRun(label, 0, label.length, 0, label.length, 0f, 0f, false, paint)
        val out = JSONObject().put("config", config).put("size", size.toDouble()).put("spacing_em", spacing.toDouble())
            .put("advance", shaped.advance.toDouble()).put("measure", paint.measureText(label).toDouble())
            .put("run_advance", paint.getRunAdvance(label, 0, label.length, 0, label.length, false, label.length).toDouble())
        if (glyphs) {
            val font = shaped.getFont(0)
            out.put("font_file", font.file?.let(File::getName)).put("ttc_index", font.ttcIndex)
                .put("axes", JSONArray(font.axes.orEmpty().map { "${it.tag}=${it.styleValue}" }))
                .put("glyph_ids", JSONArray((0 until shaped.glyphCount()).map(shaped::getGlyphId)))
                .put("glyph_x", JSONArray((0 until shaped.glyphCount()).map { shaped.getGlyphX(it).toDouble() }))
                .put("fonts_distinct", (0 until shaped.glyphCount()).map { shaped.getFont(it) }.distinct().size)
        }
        return out
    }
}
