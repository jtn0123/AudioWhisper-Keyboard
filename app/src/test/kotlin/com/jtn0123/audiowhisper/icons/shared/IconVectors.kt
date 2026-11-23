package com.jtn0123.audiowhisper.icons

import androidx.compose.material.icons.Icons
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathNode
import androidx.compose.ui.graphics.vector.VectorGroup
import androidx.compose.ui.graphics.vector.VectorPath

/**
 * The vectors the Rust `icons` crate holds (lib/rust/crates/icons/data/sources.tsv), as Compose
 * builds them, written in the file's syntax. Shared by the JVM check of that file and the device
 * dump of what Android flattened them to.
 */
object IconVectors {
    private val receivers = mapOf<String, Any>("Filled" to Icons.Filled, "Outlined" to Icons.Outlined,
        "AutoMirrored.Filled" to Icons.AutoMirrored.Filled, "AutoMirrored.Outlined" to Icons.AutoMirrored.Outlined)

    /** The one icon FlorisBoard adds to `Icons`, Backspace mirrored, in its own file. */
    private val added = mapOf("AutoMirrored.Filled.ForwardDelete" to "org.florisboard.lib.compose.icons.KeyboardForwardDeleteKt")

    /** Material icon [name] (`Filled.Search`): its file's getter, as `Icons.Filled.Search` calls it. */
    fun material(name: String): ImageVector {
        val style = name.substringBeforeLast('.')
        val icon = name.substringAfterLast('.')
        val receiver = receivers.getValue(style)
        val file = Class.forName(added[name] ?: "androidx.compose.material.icons.${style.lowercase()}.${icon}Kt")
        return file.getMethod("get$icon", receiver.javaClass).invoke(null, receiver) as ImageVector
    }

    /** The names in the file's text, skipping its comments. */
    fun names(file: String): List<String> =
        file.lines().filter { it.isNotEmpty() && !it.startsWith("#") }.map { it.substringBefore('\t') }

    /** Every path of [vector], in the order `iconPath` walked them. */
    fun paths(vector: ImageVector): List<List<PathNode>> = buildList {
        fun walk(group: VectorGroup) {
            for (node in group) when (node) {
                is VectorGroup -> walk(node)
                is VectorPath -> add(node.pathData)
            }
        }
        walk(vector.root)
    }

    /** [name]'s line of the file: the viewport, then each path. */
    fun line(name: String, width: Float, height: Float, paths: List<List<PathNode>>): String =
        (listOf(name, "$width", "$height") + paths.map(::svg)).joinToString("\t")

    /** [nodes] as the file writes them: each command's letter, then its numbers. */
    fun svg(nodes: List<PathNode>): String = nodes.joinToString(" ") { node ->
        val (command, args) = command(node)
        (listOf(command) + args.map { "$it" }).joinToString(" ")
    }

    private fun flag(on: Boolean) = if (on) 1f else 0f

    /** [node]'s letter and numbers; an arc's flags are 1 or 0. */
    fun command(node: PathNode): Pair<String, List<Float>> = when (node) {
        PathNode.Close -> "Z" to listOf()
        is PathNode.RelativeMoveTo -> "m" to listOf(node.dx, node.dy)
        is PathNode.MoveTo -> "M" to listOf(node.x, node.y)
        is PathNode.RelativeLineTo -> "l" to listOf(node.dx, node.dy)
        is PathNode.LineTo -> "L" to listOf(node.x, node.y)
        is PathNode.RelativeHorizontalTo -> "h" to listOf(node.dx)
        is PathNode.HorizontalTo -> "H" to listOf(node.x)
        is PathNode.RelativeVerticalTo -> "v" to listOf(node.dy)
        is PathNode.VerticalTo -> "V" to listOf(node.y)
        is PathNode.RelativeCurveTo -> "c" to listOf(node.dx1, node.dy1, node.dx2, node.dy2, node.dx3, node.dy3)
        is PathNode.CurveTo -> "C" to listOf(node.x1, node.y1, node.x2, node.y2, node.x3, node.y3)
        is PathNode.RelativeReflectiveCurveTo -> "s" to listOf(node.dx1, node.dy1, node.dx2, node.dy2)
        is PathNode.ReflectiveCurveTo -> "S" to listOf(node.x1, node.y1, node.x2, node.y2)
        is PathNode.RelativeQuadTo -> "q" to listOf(node.dx1, node.dy1, node.dx2, node.dy2)
        is PathNode.QuadTo -> "Q" to listOf(node.x1, node.y1, node.x2, node.y2)
        is PathNode.RelativeReflectiveQuadTo -> "t" to listOf(node.dx, node.dy)
        is PathNode.ReflectiveQuadTo -> "T" to listOf(node.x, node.y)
        is PathNode.RelativeArcTo -> "a" to listOf(node.horizontalEllipseRadius, node.verticalEllipseRadius, node.theta,
            flag(node.isMoreThanHalf), flag(node.isPositiveArc), node.arcStartDx, node.arcStartDy)
        is PathNode.ArcTo -> "A" to listOf(node.horizontalEllipseRadius, node.verticalEllipseRadius, node.theta,
            flag(node.isMoreThanHalf), flag(node.isPositiveArc), node.arcStartX, node.arcStartY)
    }

    /** FNV-1a over [text]'s UTF-8, as the Rust golden test hashes its own text. */
    fun fnv(text: String): String {
        var hash = -0x340d631b7bdddcdbL
        for (byte in text.toByteArray()) hash = (hash xor (byte.toLong() and 0xff)) * 0x100000001b3L
        return java.lang.Long.toHexString(hash).padStart(16, '0')
    }
}
