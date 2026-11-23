package com.jtn0123.audiowhisper.icons

import androidx.compose.ui.graphics.vector.PathParser
import org.w3c.dom.Element
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * lib/rust/crates/icons/data/sources.tsv is what Compose holds for each icon the Rust renderer
 * draws: the Material icon's nodes, or the vector drawable's path data as `vectorResource` parses
 * it. This writes the file from them to app/build/icons/sources.tsv and fails while the two
 * differ; copying that file over the crate's regenerates it.
 */
class IconSourcesTest {
    private val source = File("../lib/rust/crates/icons/data/sources.tsv")

    @Test fun sourcesAreTheVectorsComposeHolds() {
        val lines = source.readLines()
        val header = lines.takeWhile { it.startsWith("#") }
        val expected = header + IconVectors.names(source.readText()).map(::line)
        val written = File("build/icons/sources.tsv").apply { parentFile.mkdirs() }
        written.writeText(expected.joinToString("\n", postfix = "\n"))
        assertEquals(130, expected.size - header.size)
        assertEquals(expected, lines, "copy ${written.path} over ${source.path}")
    }

    private fun line(name: String): String {
        if (!name.startsWith("drawable.")) {
            val vector = IconVectors.material(name)
            return IconVectors.line(name, vector.viewportWidth, vector.viewportHeight, IconVectors.paths(vector))
        }
        val factory = DocumentBuilderFactory.newInstance().apply { isNamespaceAware = true }
        val file = File("src/main/res/drawable/${name.removePrefix("drawable.")}.xml")
        val root = factory.newDocumentBuilder().parse(file).documentElement
        fun attribute(element: Element, key: String) = element.getAttributeNS(ANDROID, key)
        val paths = buildList {
            fun walk(element: Element) {
                val children = element.childNodes
                for (i in 0 until children.length) {
                    val child = children.item(i) as? Element ?: continue
                    when (child.tagName) {
                        "path" -> add(PathParser().parsePathString(attribute(child, "pathData")).toNodes())
                        "group" -> walk(child)
                    }
                }
            }
            walk(root)
        }
        return IconVectors.line(name, attribute(root, "viewportWidth").toFloat(),
            attribute(root, "viewportHeight").toFloat(), paths)
    }

    private companion object {
        const val ANDROID = "http://schemas.android.com/apk/res/android"
    }
}
