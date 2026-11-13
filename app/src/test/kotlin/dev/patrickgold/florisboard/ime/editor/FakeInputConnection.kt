package dev.patrickgold.florisboard.ime.editor

/** EditText's UTF-16 replace/selection/composing rules, independent of the native expected model. */
class FakeInputConnection {
    data class Operation(val name: String, val a: Int = 0, val b: Int = 0, val text: String = "")
    var text = ""
    var start = 0
    var end = 0
    var composingStart = -1
    var composingEnd = -1
    var batches = 0
    val operations = mutableListOf<Operation>()
    val selection get() = ReferenceEditorRange.normalized(start,end)
    val composing get() = ReferenceEditorRange(composingStart,composingEnd)
    fun state() = listOf(text,start,end,composingStart,composingEnd,batches)
    fun external(value: String, cursor: Int, anchor: Int = cursor, composing: ReferenceEditorRange = ReferenceEditorRange.Unspecified) {
        text=value;start=cursor.coerceIn(0,text.length);end=anchor.coerceIn(0,text.length)
        composingStart=composing.start;composingEnd=composing.end
    }
    fun getTextBeforeCursor(n: Int, flags: Int): String? = text.substring(0,minOf(start,end)).takeLast(n)
    fun getTextAfterCursor(n: Int, flags: Int): String? = text.substring(maxOf(start,end)).take(n)
    fun getSelectedText(flags: Int): String? = text.substring(minOf(start,end),maxOf(start,end))
    fun requestCursorUpdates(flags: Int) = true
    fun beginBatchEdit() = record(Operation("begin")).also {batches++}
    fun endBatchEdit() = record(Operation("end")).also {batches--}
    fun finishComposingText() = record(Operation("finish")).also {composingStart=-1;composingEnd=-1}
    fun setComposingRegion(a: Int, b: Int): Boolean {
        record(Operation("composing",a,b))
        composingStart=minOf(a,b).coerceIn(0,text.length);composingEnd=maxOf(a,b).coerceIn(0,text.length)
        if (composingStart==composingEnd) {composingStart=-1;composingEnd=-1}
        return true
    }
    fun setComposingText(value: String, cursor: Int): Boolean {
        record(Operation("composingText",cursor,0,value));replace(value,cursor,true);return true
    }
    fun commitText(value: String, cursor: Int): Boolean {
        record(Operation("commit",cursor,0,value));replace(value,cursor,false);return true
    }
    private fun replace(value: String, cursor: Int, composing: Boolean) {
        val from=if (composingStart>=0) composingStart else minOf(start,end)
        val to=if (composingEnd>=0) composingEnd else maxOf(start,end)
        text=text.replaceRange(from,to,value)
        start=(if(cursor>0) from+value.length+cursor-1 else from+cursor).coerceIn(0,text.length);end=start
        composingStart=if(composing && value.isNotEmpty())from else -1
        composingEnd=if(composing && value.isNotEmpty())from+value.length else -1
    }
    fun deleteSurroundingText(before: Int, after: Int): Boolean {
        record(Operation("delete",before,after))
        val a=minOf(start,end);val b=maxOf(start,end)
        val from=(a-before).coerceAtLeast(0);val to=(b+after).coerceAtMost(text.length)
        text=text.removeRange(b,to).removeRange(from,a)
        start-=a-from;end-=a-from
        return true
    }
    fun deleteSurroundingTextInCodePoints(before: Int, after: Int): Boolean {
        val a=minOf(start,end);val b=maxOf(start,end)
        val left=text.offsetByCodePoints(a,-before.coerceAtMost(text.codePointCount(0,a)))
        val right=text.offsetByCodePoints(b,after.coerceAtMost(text.codePointCount(b,text.length)))
        val result=deleteSurroundingText(a-left,right-b)
        operations[operations.lastIndex]=Operation("deleteCodePoints",before,after)
        return result
    }
    fun setSelection(a: Int, b: Int): Boolean {
        record(Operation("selection",a,b));start=a.coerceIn(0,text.length);end=b.coerceIn(0,text.length);return true
    }
    fun performEditorAction(action: Int) = record(Operation("action",action))
    fun performContextMenuAction(action: Int): Boolean {
        record(Operation("contextMenu",action));start=0;end=text.length;return true
    }
    fun down(code: Int, meta: Int, repeat: Int): Boolean = record(Operation("down",code,meta,repeat.toString()))
    fun up(code: Int, meta: Int) = record(Operation("up",code,meta))
    fun apply(op: Operation): Boolean = when(op.name) {
        "begin" -> beginBatchEdit();"end" -> endBatchEdit();"finish" -> finishComposingText()
        "composing" -> setComposingRegion(op.a,op.b);"composingText" -> setComposingText(op.text,op.a)
        "commit" -> commitText(op.text,op.a);"delete" -> deleteSurroundingText(op.a,op.b)
        "selection" -> setSelection(op.a,op.b);"action" -> performEditorAction(op.a)
        "contextMenu" -> performContextMenuAction(op.a)
        "down" -> down(op.a,op.b,op.text.toIntOrNull()?:0);"up" -> up(op.a,op.b)
        else -> error("Unsupported plan operation: ${op.name}")
    }
    private fun record(op: Operation): Boolean { operations.add(op);return true }
    companion object {
        const val CURSOR_UPDATE_MONITOR=1
        const val CURSOR_UPDATE_IMMEDIATE=2
    }
}
