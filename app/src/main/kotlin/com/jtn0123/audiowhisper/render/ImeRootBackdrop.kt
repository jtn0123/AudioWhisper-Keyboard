package com.jtn0123.audiowhisper.render

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.ImageDecoder
import android.graphics.PixelFormat
import android.graphics.Rect
import android.graphics.drawable.AnimatedImageDrawable
import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Drawable
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.core.graphics.createBitmap
import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeBackdrop as B
import com.jtn0123.audiowhisper.rust.ImeThemeRule
import com.jtn0123.audiowhisper.rust.ImeThemeWindow as W
import com.jtn0123.audiowhisper.rust.WireFlag
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** Compose drew an animated background 30 times a second. */
private const val FRAME_MS = 1000L / 30

/**
 * The keyboard window's background: the colour and image Rust's `WINDOW` rule names, drawn where
 * it says on a surface of its own, as Compose's `SnyggSurfaceView` drew them. A surface, because
 * the keyboard's own surface sits behind the window and clears whatever the window draws there.
 */
internal class ImeBackdrop(context: Context) : SurfaceView(context), SurfaceHolder.Callback {
    private var theme: RendererTheme? = null
    private var dir: File? = null
    private var answer = emptyArray<String>()
    private var asked = emptyList<Any?>()
    private var source: Pair<File?, String>? = null
    private var image: Drawable? = null
    private var frame: Bitmap? = null
    private var scope = MainScope()
    private var animating: Job? = null

    init {
        setSurfaceLifecycle(SURFACE_LIFECYCLE_FOLLOWS_ATTACHMENT)
        setZOrderOnTop(false)
        holder.setFormat(PixelFormat.TRANSPARENT)
        holder.addCallback(this)
    }

    /**
     * Draws [theme]'s background, its image found in [dir], the theme's unpacked directory.
     * Returns the navigation bar's flags (light buttons, enforced contrast) when they change.
     */
    fun update(theme: RendererTheme, dir: File?): Pair<Boolean, Boolean>? {
        val before = answer.takeIf { it.isNotEmpty() }?.let { it[B.LIGHT_BARS] to it[B.CONTRAST] }
        this.theme = theme
        this.dir = dir
        ask()
        val after = answer[B.LIGHT_BARS] to answer[B.CONTRAST]
        return if (after == before) null else WireFlag.decode(after.first) to WireFlag.decode(after.second)
    }

    /** Asks Rust again only when something it reads changed, and loads an image it newly names. */
    private fun ask() {
        val theme = theme ?: return
        val size = image?.let { it.intrinsicWidth to it.intrinsicHeight } ?: (0 to 0)
        val key = listOf(theme.json, theme.mode, theme.shift, theme.paletteColors, size, width, height, dir)
        if (key == asked) return
        asked = key
        val row = W.row(W.STYLESHEET to theme.json, W.MODE to theme.mode, W.SHIFT to theme.shift,
            W.IMAGE_WIDTH to size.first.toString(), W.IMAGE_HEIGHT to size.second.toString(),
            W.SURFACE_WIDTH to width.toString(), W.SURFACE_HEIGHT to height.toString())
        answer = ImeTheme.rules(ImeThemeRule.WINDOW, *row, *theme.paletteNames.toTypedArray(),
            *theme.paletteColors.map { it.toString() }.toTypedArray())
        if (source != dir to answer[B.IMAGE]) load(answer[B.IMAGE])
        draw()
    }

    /** Loads the image at [named] inside the theme's directory, never outside it, off the main thread. */
    private fun load(named: String) {
        source = dir to named
        stop()
        image = null
        val base = dir ?: return
        if (named.isEmpty()) return
        scope.launch {
            image = withContext(Dispatchers.IO) { decode(base, named) }
            asked = emptyList()
            ask()
        }
    }

    private fun decode(base: File, named: String): Drawable? = runCatching {
        val root = base.canonicalFile
        val file = File(root, named).canonicalFile
        check(file.path.startsWith(root.path + File.separator) && file.isFile)
        ImageDecoder.decodeDrawable(ImageDecoder.createSource(file)) { decoder, _, _ ->
            decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
        }.also { (it as? BitmapDrawable)?.bitmap?.prepareToDraw() }
    }.getOrNull()

    /** Draws the colour and image once, or keeps drawing an animated image's frames. */
    private fun draw() {
        val image = image
        if (image is AnimatedImageDrawable) {
            if (animating?.isActive == true) return
            image.start()
            animating = scope.launch {
                while (isActive) {
                    paint()
                    delay(FRAME_MS)
                }
            }
        } else {
            paint()
        }
    }

    private fun stop() {
        animating?.cancel()
        animating = null
        (image as? AnimatedImageDrawable)?.stop()
    }

    private fun paint() {
        val surface = holder.surface
        if (answer.isEmpty() || !surface.isValid) return
        val canvas = surface.lockCanvas(null)
        try {
            canvas.drawColor(answer[B.BACKGROUND].toInt())
            val bitmap = bitmap() ?: return
            val dst = Rect(answer[B.LEFT].toInt(), answer[B.TOP].toInt(), answer[B.RIGHT].toInt(), answer[B.BOTTOM].toInt())
            if (!dst.isEmpty) canvas.drawBitmap(bitmap, null, dst, null)
        } finally {
            surface.unlockCanvasAndPost(canvas)
        }
    }

    /** The image as Compose drew it: a bitmap of its own size, an animation's current frame. */
    private fun bitmap(): Bitmap? = when (val image = image) {
        is BitmapDrawable -> image.bitmap
        null -> null
        else -> {
            val (w, h) = image.intrinsicWidth to image.intrinsicHeight
            if (w <= 0 || h <= 0) {
                null
            } else {
                val made = frame?.takeIf { it.width == w && it.height == h } ?: createBitmap(w, h)
                frame = made
                made.eraseColor(0)
                image.setBounds(0, 0, w, h)
                image.draw(Canvas(made))
                made
            }
        }
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        ask()
    }

    override fun surfaceCreated(holder: SurfaceHolder) = paint()

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) = paint()

    override fun surfaceDestroyed(holder: SurfaceHolder) = Unit

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        scope = MainScope()
        // What loaded or animated while detached was cancelled; the next update starts again.
        source = null
        asked = emptyList()
    }

    override fun onDetachedFromWindow() {
        stop()
        scope.cancel()
        super.onDetachedFromWindow()
    }
}
