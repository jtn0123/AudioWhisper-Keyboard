package com.jtn0123.audiowhisper.settingsui

import android.graphics.Paint
import android.graphics.Typeface
import android.graphics.text.TextRunShaper
import android.text.Layout
import android.text.StaticLayout
import android.text.TextPaint
import org.json.JSONArray
import org.json.JSONObject

/**
 * Records how Android lays out the tracked Roboto-Light (fastlane/staging/fonts, packaged as a
 * test asset) as Compose's static UI text: glyph ids and pen positions per StaticLayout line.
 * The host test `android_text_matches_device_fixture` replays these. Static strings only.
 */
internal class SettingsTypographyFixture(private val h: SettingsHarness) {
    private val samples = listOf(
        "Try Rust drawing for text keys, suggestions",
        "First fine office files, auto-capitalization and 3-4 split-keyboard rows.",
        "Supercalifragilisticexpialidocious words break where they must",
        "Line one\nLine two after a hard break",
    )
    private val spacingEm = .25f / 14f

    fun run(report: JSONObject) {
        h.main {
            val font = Typeface.createFromAsset(h.instrumentation.context.assets, "Roboto-Light.ttf")
            val cases = JSONArray()
            for (size in listOf(36.75f, 42f, 57.75f, 63f, 73.5f, 84f)) for (spacing in listOf(0f, spacingEm)) {
                for (text in samples) for (width in listOf(0, 420, 900)) {
                    cases.put(case(font, text, size, spacing, width))
                }
            }
            report.put("font", "Roboto-Light.ttf").put("cases", cases)
            // Does drawing trim letter spacing at line edges? Wide spacing makes half a gap visible.
            val ink = JSONObject()
            for (spacing in listOf(0f, .5f)) ink.put(spacing.toString(), edgeInk(font, spacing))
            report.put("edge_ink", ink)
        }
    }

    private fun case(font: Typeface, text: String, size: Float, spacing: Float, width: Int): JSONObject {
        val paint = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
            typeface = font
            textSize = size
            letterSpacing = spacing
            // Compose TextMotion.Static.
            flags = flags and (Paint.LINEAR_TEXT_FLAG or Paint.SUBPIXEL_TEXT_FLAG).inv()
            hinting = Paint.HINTING_ON
        }
        val layout = StaticLayout.Builder.obtain(text, 0, text.length, paint, if (width == 0) 100_000 else width)
            .setAlignment(Layout.Alignment.ALIGN_NORMAL).setIncludePad(false)
            .setBreakStrategy(Layout.BREAK_STRATEGY_SIMPLE)
            .setHyphenationFrequency(Layout.HYPHENATION_FREQUENCY_NONE).build()
        val lines = JSONArray()
        for (line in 0 until layout.lineCount) {
            val start = layout.getLineStart(line)
            var end = layout.getLineEnd(line)
            val shapeEnd = if (end > start && text[end - 1] == '\n') end - 1 else end
            val ids = JSONArray()
            val xs = JSONArray()
            if (shapeEnd > start) {
                val shaped = TextRunShaper.shapeTextRun(text, start, shapeEnd - start, start, shapeEnd - start, 0f, 0f, false, paint)
                for (g in 0 until shaped.glyphCount()) {
                    ids.put(shaped.getGlyphId(g))
                    xs.put(shaped.getGlyphX(g).toDouble())
                }
            }
            if (end > text.length) end = text.length
            // Caret x at each offset StaticLayout draws from (TextLine applies the edge flags).
            val carets = JSONArray()
            for (offset in start until shapeEnd) {
                if (Character.isLowSurrogate(text[offset])) continue
                carets.put(JSONArray().put(utf8(text, offset)).put(layout.getPrimaryHorizontal(offset).toDouble()))
            }
            lines.put(JSONObject().put("start", utf8(text, start)).put("end", utf8(text, end))
                .put("max", layout.getLineMax(line).toDouble()).put("ids", ids).put("x", xs).put("carets", carets))
        }
        return JSONObject().put("text", text).put("size", size.toDouble()).put("spacing_em", spacing.toDouble())
            .put("width", width).put("lines", lines)
    }

    private fun edgeInk(font: Typeface, spacing: Float): JSONArray {
        val paint = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
            typeface = font
            textSize = 42f
            letterSpacing = spacing
            color = android.graphics.Color.BLACK
            flags = flags and (Paint.LINEAR_TEXT_FLAG or Paint.SUBPIXEL_TEXT_FLAG).inv()
            hinting = Paint.HINTING_ON
        }
        val text = "Hello"
        val layout = StaticLayout.Builder.obtain(text, 0, text.length, paint, 100_000).setIncludePad(false).build()
        val bitmap = android.graphics.Bitmap.createBitmap(640, 80, android.graphics.Bitmap.Config.ARGB_8888)
        bitmap.eraseColor(android.graphics.Color.WHITE)
        android.graphics.Canvas(bitmap).apply { translate(20f, 10f) }.also(layout::draw)
        val columns = (0 until bitmap.width).filter { x -> (0 until bitmap.height).any { bitmap.getPixel(x, it) != android.graphics.Color.WHITE } }
        return JSONArray().put(columns.first() - 20).put(columns.last() - 20).put(layout.getLineMax(0).toDouble())
    }

    private fun utf8(text: String, utf16: Int) = text.substring(0, utf16).toByteArray(Charsets.UTF_8).size
}
