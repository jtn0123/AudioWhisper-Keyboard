package com.jtn0123.audiowhisper.render

import android.content.Context
import android.graphics.Bitmap
import androidx.core.graphics.createBitmap
import android.graphics.Canvas
import android.view.MotionEvent
import android.view.View
import android.os.SystemClock
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.RenderProtocol.IME_CHROME
import com.jtn0123.audiowhisper.rust.RenderProtocol.PANEL_SUBTYPE

/** Small, infrequently changing surfaces share Vello CPU and the native semantic provider.
 * Pixel buffers are cached between updates; no SurfaceView is needed in settings/extraction.
 */
internal class RustChromeView(context: Context) : View(context) {
    private var native: KeyboardRenderer? = null
    private var bitmap: Bitmap? = null
    private var theme: RendererTheme? = null
    private var kind = 0
    private var labels = emptyArray<String>()
    private var side = 0
    private var subtypes: RendererSubtypes? = null
    private var subtypeBottomInset = 0f
    var subtypeAction: (Int?) -> Unit = {}
    var action: (Int) -> Unit = {}
    var fieldLayout: (FloatArray) -> Unit = {}
    private val accessibility = RendererAccessibility(this, { native?.semanticSnapshot() ?: "{\"nodes\":[],\"mode\":\"\",\"shift\":\"\",\"popup\":false}" }, { id, op ->
        dispatch(native?.semanticAction(id, op) ?: intArrayOf()); redraw(); true
    }, { native?.semanticCancel(); redraw() })

    init { isClickable = true; importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES }
    /** A chrome view of [kind] with its [labels]; the one-handed panel's [side] is 0 start, 1 end. */
    fun update(theme: RendererTheme, kind: Int, labels: Array<String>, side: Int = 0) {
        if (this.theme == theme && this.kind == kind && this.labels.contentEquals(labels) && this.side == side) return
        this.theme = theme; this.kind = kind; this.labels = labels; this.side = side
        configure()
    }
    fun updateSubtypes(theme: RendererTheme, data: RendererSubtypes, bottomInset: Float) {
        val old = subtypes
        if (this.theme == theme && subtypeBottomInset == bottomInset && old?.selected == data.selected && old.ids.contentEquals(data.ids) && old.labels.contentEquals(data.labels)) return
        this.theme = theme; subtypes = data; subtypeBottomInset = bottomInset; configure()
    }
    private val wakeup = Runnable {
        native?.let { dispatch(it.tick(SystemClock.uptimeNanos())); redraw() }
    }
    private fun schedule() {
        removeCallbacks(wakeup)
        val deadline = native?.nextDeadline() ?: -1
        if (deadline >= 0) postDelayed(wakeup, ((deadline - SystemClock.uptimeNanos() + 999_999) / 1_000_000).coerceAtLeast(0))
    }
    private fun configure() {
        val theme = theme ?: return
        if (width <= 0 || height <= 0) return
        val renderer = native ?: KeyboardRenderer(systemRendererFonts()).also { native = it }
        renderer.theme(theme.json, theme.paletteNames.toTypedArray(), theme.paletteColors.toIntArray(),
            floatArrayOf(theme.density, theme.fontScale, theme.fontMultiplier), theme.fonts.toTypedArray())
        val data = subtypes
        if (data == null) {
            renderer.chrome(floatArrayOf(width.toFloat(), height.toFloat(), theme.density, kind.toFloat(), side.toFloat()), labels, arrayOf(theme.mode, theme.shift))
            fieldLayout(renderer.chromeInfo())
        } else {
            renderer.update(floatArrayOf(width.toFloat(), height.toFloat(), 0f, 1f, 1f, 0f),
                arrayOf(theme.mode, theme.shift), intArrayOf(), floatArrayOf(), emptyArray(), emptyArray())
            renderer.panels(floatArrayOf(7f,theme.density,48f*theme.density,0f,50_000_000f,0f,500_000_000f),
                intArrayOf(-1,0,0),Array(6) { "" })
            renderer.subtypes(data.selected,data.ids,data.labels,subtypeBottomInset)
        }
        redraw()
    }
    private fun redraw() {
        val native = native ?: return
        if (width <= 0 || height <= 0) return
        val image = bitmap?.takeIf { it.width == width && it.height == height }
            ?: createBitmap(width, height, Bitmap.Config.ARGB_8888).also { bitmap?.recycle(); bitmap = it }
        image.setPixels(native.pixels(), 0, width, 0, 0, width, height)
        accessibility.refresh(); schedule(); invalidate()
    }
    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) { super.onSizeChanged(w,h,oldw,oldh); configure() }
    override fun onDraw(canvas: Canvas) { super.onDraw(canvas); bitmap?.let { canvas.drawBitmap(it, 0f, 0f, null) } }
    override fun getAccessibilityNodeProvider() = accessibility
    override fun dispatchHoverEvent(event: MotionEvent) = accessibility.hover(event) || super.dispatchHoverEvent(event)
    override fun onTouchEvent(event: MotionEvent): Boolean {
        val native = native ?: return false
        if (accessibility.touch(event)) return true
        val packet = event.rendererPacket()
        dispatch(native.touch(packet.metadata, packet.times, packet.coords))
        redraw(); if (event.actionMasked == MotionEvent.ACTION_UP) performClick(); return true
    }
    private fun dispatch(actions: IntArray) {
        for (i in actions.indices step 3) when (actions[i]) {
            IME_CHROME -> action(actions[i + 1])
            PANEL_SUBTYPE -> subtypeAction(if (actions[i + 2] != 0) actions[i + 1] else null)
        }
    }
    override fun performClick(): Boolean { super.performClick(); return true }
    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow(); removeCallbacks(wakeup); native?.close(); native = null; bitmap?.recycle(); bitmap = null
    }
    override fun onAttachedToWindow() { super.onAttachedToWindow(); configure() }
}
