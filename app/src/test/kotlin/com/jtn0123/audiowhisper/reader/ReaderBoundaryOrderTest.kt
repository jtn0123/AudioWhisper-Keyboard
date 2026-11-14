package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ReaderTree
import com.jtn0123.audiowhisper.rust.ReaderWalk
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertSame

/** Freeze the actual pre-consolidation bridge's reads, including reads the older Kotlin walk skipped. */
class ReaderBoundaryOrderTest {
    private open class Tree : ReaderTree {
        val reads = mutableListOf<String>()
        override fun facts(id: Int): IntArray {
            reads += "facts$id"
            return intArrayOf(0, 0, 100, 100, if (id == 1) 1 else 0, 0, 1, 0, 0, 0, 0, 0, if (id == 0) 2 else 0)
        }
        override fun className(id: Int): String { reads += "class$id"; return "TextView" }
        override fun text(id: Int): String { reads += "text$id"; return if (id == 0) "synthetic root" else "" }
        override fun child(id: Int, index: Int): Int { reads += "child$id:$index"; return index + 1 }
    }
    private val visible = intArrayOf(0, 0, 100, 100)

    @Test fun passwordPruningAndParentFallbackPreserveEveryRead() {
        val tree = Tree()
        assertContentEquals(intArrayOf(-1, 3, 0, 0, 0, 100, 0), ReaderWalk.walk(tree, visible))
        assertEquals(listOf("facts0", "class0", "child0:0", "facts1", "class1", "child0:1",
            "facts2", "class2", "text2", "text0"), tree.reads)
    }

    @Test fun platformFailuresStopAtTheSameReadAndKeepTheOriginalException() {
        for (failClass in listOf(false, true)) {
            val marker = IllegalStateException("synthetic platform failure")
            val tree = object : Tree() {
                override fun facts(id: Int): IntArray = super.facts(id).also { if (id == 1 && !failClass) throw marker }
                override fun className(id: Int): String = super.className(id).also { if (id == 1 && failClass) throw marker }
            }
            assertSame(marker, assertFailsWith<IllegalStateException> { ReaderWalk.walk(tree, visible) })
            assertEquals(listOf("facts0", "class0", "child0:0", "facts1") + if (failClass) listOf("class1") else emptyList(), tree.reads)
        }
    }

    @Test fun invalidFactsFailBeforeAskingForClassOrText() {
        val tree = object : Tree() {
            override fun facts(id: Int): IntArray = super.facts(id).let { if (id == 1) it.copyOf(12) else it }
        }
        assertEquals("Node facts length", assertFailsWith<IllegalArgumentException> { ReaderWalk.walk(tree, visible) }.message)
        assertEquals(listOf("facts0", "class0", "child0:0", "facts1"), tree.reads)
    }

    @Test fun factsAreCopiedBeforeClassNameCanMutateTheSourceArray() {
        val fields = intArrayOf(0, 7, 100, 90, 0, 0, 1, 0, 0, 0, 0, 0, 0)
        val tree = object : Tree() {
            override fun facts(id: Int) = fields
            override fun className(id: Int): String { fields[1] = 99; fields[4] = 1; return "TextView" }
            override fun text(id: Int) = "synthetic"
        }
        assertContentEquals(intArrayOf(-1, 1, 0, 0, 7, 90, 0), ReaderWalk.walk(tree, visible))
    }

    @Test fun everyBitOfSignedGeometryAndClassUtf16IsRetained() {
        for (value in listOf(Int.MIN_VALUE, -1, 0, 65535, 65536, 0xD800DFFF.toInt(), Int.MAX_VALUE)) {
            val tree = object : Tree() {
                override fun facts(id: Int) = intArrayOf(0, value, 100, value.inv(), 0, 0, 1, 0, 0, 0, 0, 0, 0)
                override fun className(id: Int) = "android.webkit.WebView"
                override fun text(id: Int) = "synthetic"
            }
            assertContentEquals(intArrayOf(-1, 1, 1, 0, value, value.inv(), 0), ReaderWalk.walk(tree, visible))
        }
    }

    @Test fun capsStillFetchChildrenWithoutVisitingOrReadingTheirFacts() {
        var facts = 0
        var children = 0
        val tree = object : Tree() {
            override fun facts(id: Int): IntArray {
                facts++
                return intArrayOf(0, 0, 100, 100, 0, 0, 1, 0, 0, 0, 0, 0, if (id == 0) 20_100 else 0)
            }
            override fun child(id: Int, index: Int): Int { children++; return index + 1 }
            override fun text(id: Int) = ""
        }
        assertContentEquals(intArrayOf(-1, 20_000, 0), ReaderWalk.walk(tree, visible))
        assertEquals(20_000, facts)
        assertEquals(20_100, children)
        val marker = IllegalStateException("synthetic child failure after cap")
        val failing = object : ReaderTree by tree {
            override fun child(id: Int, index: Int) = tree.child(id, index).also { if (index == 20_050) throw marker }
        }
        assertSame(marker, assertFailsWith<IllegalStateException> { ReaderWalk.walk(failing, visible) })
    }
}
