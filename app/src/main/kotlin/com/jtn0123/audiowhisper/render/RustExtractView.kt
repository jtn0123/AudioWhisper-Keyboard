package com.jtn0123.audiowhisper.render

import android.content.Context
import android.inputmethodservice.ExtractEditText
import android.util.TypedValue
import android.widget.FrameLayout
import kotlin.math.roundToInt

/** The real ExtractEditText remains in Android's IME protocol and accessibility tree. */
internal class RustExtractView @JvmOverloads constructor(context: Context, private val editor: ExtractEditText = ExtractEditText(context)) : FrameLayout(context) {
    private var field = FloatArray(6)
    private val chrome = RustChromeView(context).also { view ->
        view.fieldLayout = { next ->
            if (field.indices.any { field[it].toRawBits() != next[it].toRawBits() }) {
                field = next
                editor.setTextColor(next[4].toRawBits())
                val color = next[4].toRawBits()
                editor.setHintTextColor((color and 0x00ffffff) or (((color ushr 24) * .6f).roundToInt() shl 24))
                editor.setTextSize(TypedValue.COMPLEX_UNIT_PX, next[5])
                requestLayout()
            }
        }
    }
    init { addView(chrome); addView(editor) }
    fun update(theme: RendererTheme, label: String, action: () -> Unit) {
        chrome.action = { action() }; chrome.update(theme, 1, arrayOf(label), emptyArray())
    }
    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        val width = MeasureSpec.getSize(widthMeasureSpec); val height = MeasureSpec.getSize(heightMeasureSpec)
        setMeasuredDimension(width, height)
        chrome.measure(MeasureSpec.makeMeasureSpec(width, MeasureSpec.EXACTLY), MeasureSpec.makeMeasureSpec(height, MeasureSpec.EXACTLY))
        editor.measure(MeasureSpec.makeMeasureSpec((field[2]-field[0]).roundToInt().coerceAtLeast(0), MeasureSpec.EXACTLY),
            MeasureSpec.makeMeasureSpec((field[3]-field[1]).roundToInt().coerceAtLeast(0), MeasureSpec.EXACTLY))
    }
    override fun onLayout(changed: Boolean, left: Int, top: Int, right: Int, bottom: Int) {
        chrome.layout(0,0,width,height)
        editor.layout(field[0].roundToInt(), field[1].roundToInt(), field[2].roundToInt(), field[3].roundToInt())
    }
}
