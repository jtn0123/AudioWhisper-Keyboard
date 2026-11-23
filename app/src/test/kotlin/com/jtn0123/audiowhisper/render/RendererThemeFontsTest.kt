package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import org.florisboard.lib.snygg.SnyggStylesheet
import org.florisboard.lib.snygg.value.SnyggAssetResolver
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** The `@font` files Kotlin resolves reach the renderer through the generated boundary and draw its keys. */
class RendererThemeFontsTest {
    private val fixtures = File("../lib/rust/crates/render/tests/fixtures")
    private val roboto = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val stylesheet = SnyggStylesheet.fromJson("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "@font `Fixture`":[{"src":"uri(`flex:/fonts/regular.ttf`)"},{"src":"uri(`flex:/fonts/bold.ttf`)","font-weight":"bold"},
            {"src":"uri(`flex:/fonts/absent.ttf`)","font-style":"italic"}],
        "key":{"background":"#ffffff","foreground":"#000000","font-family":"`Fixture`","font-size":"24sp"}}""").getOrThrow()

    /** Resolves each URI to [files]' entry for it, and refuses the rest, as the app's resolver refuses a missing file. */
    private fun resolver(files: Map<String, File>) = object : SnyggAssetResolver {
        override fun resolveAbsolutePath(uri: String) = runCatching { checkNotNull(files[uri]).absolutePath }
    }

    private fun preview(fonts: List<String>) = KeyboardRenderer(arrayOf(roboto)).use { renderer ->
        renderer.theme(stylesheet.toJson().getOrThrow(), emptyArray(), intArrayOf(), floatArrayOf(2f, 1f, 1f), fonts.toTypedArray())
        renderer.keyPreview(104, arrayOf("character", "Hello", ""))
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

    @Test fun theResolvedFontDrawsTheKeyAndABrokenOneDrawsTheDefault() {
        val plain = preview(emptyList())
        val files = fontFiles(stylesheet, resolver(mapOf("flex:/fonts/regular.ttf" to File(fixtures, "fixture-script-regular-subset.ttf"))))
        val themed = preview(files)
        // The bubble fits its label, so a wider face can also change its size: every extra entry differs too.
        val differing = plain.zip(themed).count { (a, b) -> a != b } + kotlin.math.abs(plain.size - themed.size)
        assertTrue(differing > 200, "the theme font changed only $differing pixels")
        val broken = File.createTempFile("theme-font", ".ttf").apply { deleteOnExit(); writeBytes(ByteArray(512) { it.toByte() }) }
        assertTrue(plain.contentEquals(preview(listOf("flex:/fonts/regular.ttf", broken.absolutePath))))
        assertTrue(plain.contentEquals(preview(listOf("flex:/fonts/regular.ttf", File(fixtures, "missing.ttf").absolutePath))))
    }
}
