package com.jtn0123.audiowhisper.reader

import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityNodeInfo.AccessibilityAction

/**
 * What a walk of one window found: the app it belongs to, its text in reading order, the list that
 * scrolls (null if nothing does), the part of the screen the app shows above the keyboard, how
 * many nodes the walk looked at, and whether there is a web page in it.
 */
internal class Screen(
    val app: String?,
    val blocks: List<TextBlock>,
    val scroller: AccessibilityNodeInfo?,
    val visibleTop: Int,
    val visibleBottom: Int,
    val nodes: Int,
    val web: Boolean,
) {
    /** Every block is in sight: a short page, or a web page whose tree is still being built. */
    val allShown: Boolean get() = PageText.allShown(blocks, visibleTop, visibleBottom)
}

/**
 * Walks a window's accessibility tree for its text. Fields being typed in and passwords are left
 * out, and so are labels only an icon carries ("Back", "More options"), which aren't page text.
 * A node with text and children is read through its children: a web page puts its title on its
 * root, and a link's text both on the link and on the text inside it.
 *
 * The list that scrolls is the one going up and down that holds the most of the text; a carousel
 * or a pager going sideways is never it, and its pages out of sight are skipped.
 */
internal class ScreenWalk(private val visible: Rect) {
    private class Piece(val text: String, val node: AccessibilityNodeInfo, val bounds: Rect, val scroller: Int)
    private class Scroller(val node: AccessibilityNodeInfo, val parent: Int, val area: Long)

    private val pieces = ArrayList<Piece>()
    private val scrollers = ArrayList<Scroller>()
    private var nodes = 0
    private var web = false

    fun walk(root: AccessibilityNodeInfo): Screen {
        visit(root, scroller = NONE, sideways = false, depth = 0)
        val areas = LongArray(scrollers.size) { scrollers[it].area }
        val main = ScreenRules.mainScroller(IntArray(pieces.size) { pieces[it].scroller }, areas)
        val parents = IntArray(scrollers.size) { scrollers[it].parent }
        val list = scrollers.getOrNull(main)?.node
        val follow = FollowAlong(list, list?.let(::visiblePart) ?: visible)
        val blocks = PageText.blocks(pieces.map { piece ->
            val inList = ScreenRules.isWithin(parents, piece.scroller, main)
            TextBlock(piece.text, piece.bounds.top, piece.bounds.bottom, inList) {
                if (inList) follow.show(piece.node)
            }
        })
        return Screen(root.packageName?.toString(), blocks, list, visible.top, visible.bottom, nodes, web)
    }

    private fun visit(node: AccessibilityNodeInfo, scroller: Int, sideways: Boolean, depth: Int) {
        if (nodes >= MAX_NODES || depth > MAX_DEPTH) return
        nodes++
        if (node.isPassword || node.isEditable) return
        if (sideways && !node.isVisibleToUser) return
        if (!web && WEB_VIEW.contentEquals(node.className)) web = true
        val bounds = Rect().also(node::getBoundsInScreen)
        var inner = scroller
        var across = sideways
        if (node.isScrollable) {
            if (isSideways(node)) {
                across = true
            } else {
                scrollers += Scroller(node, scroller, ScreenRules.visibleArea(bounds.box(), visible.box()))
                inner = scrollers.lastIndex
            }
        }
        val before = pieces.size
        for (index in 0 until node.childCount) {
            node.getChild(index)?.let { visit(it, inner, across, depth + 1) }
        }
        val text = node.text
        if (pieces.size == before && !text.isNullOrBlank()) pieces += Piece(text.toString(), node, bounds, scroller)
    }

    /** The part of [node] the app shows above the keyboard. */
    private fun visiblePart(node: AccessibilityNodeInfo): Rect {
        val part = ScreenRules.visiblePart(Rect().also(node::getBoundsInScreen).box(), visible.box())
        return Rect(part.left, part.top, part.right, part.bottom)
    }

    private fun Rect.box() = Box(left, top, right, bottom)

    private companion object {
        const val NONE = ScreenRules.NONE
        const val WEB_VIEW = "android.webkit.WebView"

        /** A long web page is a few thousand nodes; this keeps a runaway tree from taking seconds. */
        const val MAX_NODES = 20_000
        const val MAX_DEPTH = 250

        val UP_DOWN = setOf(
            AccessibilityAction.ACTION_SCROLL_UP.id, AccessibilityAction.ACTION_SCROLL_DOWN.id,
            AccessibilityAction.ACTION_PAGE_UP.id, AccessibilityAction.ACTION_PAGE_DOWN.id,
        )
        val ACROSS = setOf(
            AccessibilityAction.ACTION_SCROLL_LEFT.id, AccessibilityAction.ACTION_SCROLL_RIGHT.id,
            AccessibilityAction.ACTION_PAGE_LEFT.id, AccessibilityAction.ACTION_PAGE_RIGHT.id,
        )

        fun isSideways(node: AccessibilityNodeInfo): Boolean {
            val collection = node.collectionInfo
            val actions = node.actionList.map { it.id }
            return ScreenRules.isSideways(
                rows = collection?.rowCount ?: 0,
                columns = collection?.columnCount ?: 0,
                upDown = actions.any { it in UP_DOWN },
                across = actions.any { it in ACROSS },
                className = node.className?.toString().orEmpty(),
            )
        }
    }
}
