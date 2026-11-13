package com.jtn0123.audiowhisper.render

import androidx.compose.ui.graphics.asAndroidPath
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.graphics.vector.VectorGroup
import androidx.compose.ui.graphics.vector.VectorPath

/** Existing Material key icons, flattened at quarter-pixel tolerance into plain path commands. */
internal fun iconPath(icon: ImageVector): String = buildString {
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
