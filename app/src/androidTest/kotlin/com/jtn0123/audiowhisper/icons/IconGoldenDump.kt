package com.jtn0123.audiowhisper.icons

import android.content.Context
import androidx.compose.ui.graphics.asAndroidPath
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.graphics.vector.VectorGroup
import androidx.compose.ui.graphics.vector.VectorPath
import androidx.compose.ui.res.vectorResource
import androidx.test.platform.app.InstrumentationRegistry
import dev.patrickgold.florisboard.R
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The source of lib/rust/crates/icons/tests/fixtures/golden.tsv: what `iconPath` (deleted after
 * 29cb06dcd; [referenceIconPath] is it verbatim) made on Android of each icon the Rust crate holds.
 * scripts/icon-golden.sh runs it in an app built with -PiconsFixture=true, whose icons-fixture.pro
 * keeps the Compose icons R8 drops from the ordinary build.
 */
class IconGoldenDump {
    private val drawables = mapOf("ic_accessibility_one_handed" to R.drawable.ic_accessibility_one_handed,
        "ic_file_blank" to R.drawable.ic_file_blank, "ic_incognito" to R.drawable.ic_incognito,
        "ic_incognito_off" to R.drawable.ic_incognito_off, "ic_keyboard_keys" to R.drawable.ic_keyboard_keys,
        "ic_keyboard_char_width_switcher_full" to R.drawable.ic_keyboard_char_width_switcher_full,
        "ic_keyboard_char_width_switcher_half" to R.drawable.ic_keyboard_char_width_switcher_half)

    @Test fun dump() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val target = instrumentation.targetContext
        val file = instrumentation.context.assets.open("sources.tsv").bufferedReader().use { it.readText() }
        val golden = StringBuilder()
        val points = StringBuilder()
        for (line in file.lines().filter { it.isNotEmpty() && !it.startsWith("#") }) {
            val fields = line.split('\t')
            val vector = vector(fields[0], target)
            assertHolds(fields, vector)
            val text = referenceIconPath(vector)
            val count = text.split(' ').count { it == "M" || it == "L" }
            golden.append("${fields[0]}\t$count\t${IconVectors.fnv(text)}\t$text\n")
            points.append(fields[0]).append('\t').append(IconVectors.paths(vector).joinToString("|") { nodes ->
                PathParser().addPathNodes(nodes).toPath().asAndroidPath().approximate(0.25f)
                    .joinToString(" ") { "%08x".format(it.toRawBits()) }
            }).append('\n')
        }
        File(target.cacheDir, "icon-golden.tsv").writeText(golden.toString())
        File(target.cacheDir, "icon-points.tsv").writeText(points.toString())
    }

    /** The icon as the app held it: Compose's Material icon, or the drawable `vectorResource` read. */
    private fun vector(name: String, context: Context): ImageVector {
        val drawable = name.removePrefix("drawable.")
        if (drawable == name) return IconVectors.material(name)
        return ImageVector.vectorResource(null, context.resources, drawables.getValue(drawable))
    }

    /** The file's [fields] hold [vector]'s viewport and nodes, number for number. */
    private fun assertHolds(fields: List<String>, vector: ImageVector) {
        val name = fields[0]
        assertEquals(name, fields[1].toFloat().toRawBits(), vector.viewportWidth.toRawBits())
        assertEquals(name, fields[2].toFloat().toRawBits(), vector.viewportHeight.toRawBits())
        val held = IconVectors.paths(vector).map { nodes ->
            nodes.flatMap { node ->
                val (command, args) = IconVectors.command(node)
                listOf(command) + args.map { "%08x".format(it.toRawBits()) }
            }
        }
        val written = fields.drop(3).map { svg ->
            svg.split(' ').map { if (it.first().isLetter()) it else "%08x".format(it.toFloat().toRawBits()) }
        }
        assertEquals(name, written, held)
    }

    /** RendererIcons.kt's `iconPath` at 29cb06dcd, verbatim. */
    private fun referenceIconPath(icon: ImageVector): String = buildString {
        fun appendGroup(group: VectorGroup) {
            for (node in group) {
                when (node) {
                    is VectorGroup -> appendGroup(node)
                    is VectorPath -> {
                        val points = PathParser().addPathNodes(node.pathData).toPath().asAndroidPath().approximate(0.25f)
                        var fraction = -1f
                        for (i in points.indices step 3) {
                            val start = i == 0 || points[i] == fraction
                            if (start && i > 0) append("Z ")
                            append(if (start) "M " else "L ")
                            append(points[i + 1] * 24f / icon.viewportWidth).append(' ')
                            append(points[i + 2] * 24f / icon.viewportHeight).append(' ')
                            fraction = points[i]
                        }
                        append("Z ")
                    }
                }
            }
        }
        appendGroup(icon.root)
    }
}
