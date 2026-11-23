package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import org.florisboard.lib.snygg.SnyggStylesheet
import org.florisboard.lib.snygg.value.SnyggAssetResolver
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** The `@font` files Kotlin resolves reach the renderer through the generated boundary and draw its labels. */
class RendererThemeFontsTest {
    private val fixtures = File("../lib/rust/crates/render/tests/fixtures")
    private val roboto = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val stylesheet = SnyggStylesheet.fromJson("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "@font `Fixture`":[{"src":"uri(`flex:/fonts/regular.ttf`)"},{"src":"uri(`flex:/fonts/bold.ttf`)","font-weight":"bold"},
            {"src":"uri(`flex:/fonts/absent.ttf`)","font-style":"italic"}],
        "extracted-landscape-input-action":{"background":"#ffffff","foreground":"#000000","font-family":"`Fixture`",
            "font-size":"24sp"}}""").getOrThrow()

    /** Resolves each URI to [files]' entry for it, and refuses the rest, as the app's resolver refuses a missing file. */
    private fun resolver(files: Map<String, File>) = object : SnyggAssetResolver {
        override fun resolveAbsolutePath(uri: String) = runCatching { checkNotNull(files[uri]).absolutePath }
    }

    /** The landscape extract view's action button labelled "Hello", which the renderer lays out and rasters offscreen. */
    private fun preview(fonts: List<String>) = KeyboardRenderer(arrayOf(roboto)).use { renderer ->
        renderer.theme(stylesheet.toJson().getOrThrow(), emptyArray(), intArrayOf(), floatArrayOf(2f, 1f, 1f), fonts.toTypedArray())
        renderer.chrome(floatArrayOf(400f, 120f, 2f, 1f, 0f), arrayOf("Hello"), arrayOf("characters", "unshifted"))
        renderer.pixels()
    }

    @Test fun eachResolvedSrcIsItsUriThenItsPathAndAnUnresolvedOneIsLeftOut() {
        val regular = File(fixtures, "fixture-script-regular-subset.ttf")
        val bold = File(fixtures, "fixture-script-bold-subset.ttf")
        val files = fontFiles(stylesheet, resolver(mapOf("flex:/fonts/regular.ttf" to regular, "flex:/fonts/bold.ttf" to bold)))
        assertEquals(listOf("flex:/fonts/regular.ttf", regular.absolutePath, "flex:/fonts/bold.ttf", bold.absolutePath), files)
        assertEquals(emptyList(), fontFiles(SnyggStylesheet.fromJson(
            """{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key":{"background":"#ffffff"}}""").getOrThrow(),
            resolver(emptyMap())))
    }

    @Test fun theResolvedFontDrawsTheLabelAndABrokenOneDrawsTheDefault() {
        val plain = preview(emptyList())
        val files = fontFiles(stylesheet, resolver(mapOf("flex:/fonts/regular.ttf" to File(fixtures, "fixture-script-regular-subset.ttf"))))
        val themed = preview(files)
        val differing = plain.zip(themed).count { (a, b) -> a != b }
        assertTrue(differing > 200, "the theme font changed only $differing pixels")
        val broken = File.createTempFile("theme-font", ".ttf").apply { deleteOnExit(); writeBytes(ByteArray(512) { it.toByte() }) }
        assertTrue(plain.contentEquals(preview(listOf("flex:/fonts/regular.ttf", broken.absolutePath))))
        assertTrue(plain.contentEquals(preview(listOf("flex:/fonts/regular.ttf", File(fixtures, "missing.ttf").absolutePath))))
    }
}
