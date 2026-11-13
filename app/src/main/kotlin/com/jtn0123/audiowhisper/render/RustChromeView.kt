package com.jtn0123.audiowhisper.render

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.view.MotionEvent
import android.view.View
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.RenderProtocol.IME_CHROME

/** Small, infrequently changing surfaces share Vello CPU and the native semantic provider.
 * Pixel buffers are cached between updates; no SurfaceView is needed in settings/extraction.
 */
internal class RustChromeView(context: Context) : View(context) {
    private var native: KeyboardRenderer? = null
    private var bitmap: Bitmap? = null
    private var theme: RendererTheme? = null
    private var kind = 0
    private var labels = emptyArray<String>()
    private var icons = emptyArray<String>()
    var action: (Int) -> Unit = {}
    var fieldLayout: (FloatArray) -> Unit = {}
    private val accessibility = RendererAccessibility(this, { native?.semanticSnapshot() ?: "{\"nodes\":[],\"mode\":\"\",\"shift\":\"\",\"popup\":false}" }, { id, op ->
        dispatch(native?.semanticAction(id, op) ?: intArrayOf()); redraw(); true
    }, { native?.semanticCancel(); redraw() })

    init { isClickable = true; importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES }
    fun update(theme: RendererTheme, kind: Int, labels: Array<String>, icons: Array<String>) {
        if (this.theme == theme && this.kind == kind && this.labels.contentEquals(labels) && this.icons.contentEquals(icons)) return
        this.theme = theme; this.kind = kind; this.labels = labels; this.icons = icons
        configure()
    }
    private fun configure() {
        val theme = theme ?: return
        if (width <= 0 || height <= 0) return
        val renderer = native ?: KeyboardRenderer(systemRendererFonts()).also { native = it }
        renderer.theme(theme.json, theme.paletteNames.toTypedArray(), theme.paletteColors.toIntArray(),
            floatArrayOf(theme.density, theme.fontScale, theme.fontMultiplier))
        renderer.chrome(floatArrayOf(width.toFloat(), height.toFloat(), theme.density, kind.toFloat()), labels, icons)
        fieldLayout(renderer.chromeInfo())
        redraw()
    }
    private fun redraw() {
        val native = native ?: return
        if (width <= 0 || height <= 0) return
        val image = bitmap?.takeIf { it.width == width && it.height == height }
            ?: Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888).also { bitmap?.recycle(); bitmap = it }
        image.setPixels(native.pixels(), 0, width, 0, 0, width, height)
        accessibility.refresh(); invalidate()
    }
    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) { super.onSizeChanged(w,h,oldw,oldh); configure() }
    override fun onDraw(canvas: Canvas) { super.onDraw(canvas); bitmap?.let { canvas.drawBitmap(it, 0f, 0f, null) } }
    override fun getAccessibilityNodeProvider() = accessibility
    override fun dispatchHoverEvent(event: MotionEvent) = accessibility.hover(event) || super.dispatchHoverEvent(event)
    override fun onTouchEvent(event: MotionEvent): Boolean {
        val native = native ?: return false
        if (accessibility.touch(event)) return true
        dispatch(native.touch(intArrayOf(event.actionMasked, event.actionIndex, event.getPointerId(0)),
            longArrayOf(event.downTime * 1_000_000, event.eventTimeNanos), floatArrayOf(event.x, event.y)))
        redraw(); if (event.actionMasked == MotionEvent.ACTION_UP) performClick(); return true
    }
    private fun dispatch(actions: IntArray) {
        for (i in actions.indices step 3) if (actions[i] == IME_CHROME) action(actions[i + 1])
    }
    override fun performClick(): Boolean { super.performClick(); return true }
    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow(); native?.close(); native = null; bitmap?.recycle(); bitmap = null
    }
    override fun onAttachedToWindow() { super.onAttachedToWindow(); configure() }
}
