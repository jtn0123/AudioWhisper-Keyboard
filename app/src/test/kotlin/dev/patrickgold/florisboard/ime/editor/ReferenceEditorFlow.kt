package dev.patrickgold.florisboard.ime.editor

import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.lib.FlorisLocale
import kotlinx.coroutines.runBlocking

/** The replaced EditorInstance branches, connected to the unchanged reference editor operations. */
internal class ReferenceEditorFlow(val base: ReferenceAbstractEditorInstance) : SpacingEditor {
    val spacing = ReferenceSpacing(this)
    override var autoSpacePunctuation=false
    override val isRawInputEditor get()=base.activeInfo.isRawInputEditor
    override var keyVariation=KeyVariation.NORMAL
    override val inputType get()=base.activeInfo.inputAttributes.raw
    override val selection get()=base.activeContent.selection.let{EditorRange(it.start,it.end)}
    override val locale=FlorisLocale.fromTag("en")
    override val punctuationRule=PunctuationRule.Fallback
    override fun textBeforeCursor(n:Int):String=base.run{activeContent.getTextBeforeCursor(n)}
    override val currentWordText get()=base.activeContent.currentWordText
    var immediateWord=true
    private var undo:Triple<String,Int,String>?=null
    private fun sync() {
        base.pendingSpace=spacing.phantomSpace.isActive
        base.determine=!isRawInputEditor && !base.activeInfo.inputAttributes.flagTextNoSuggestions &&
            (spacing.phantomSpace.isInactive || spacing.phantomSpace.showComposingRegion)
    }
    private fun phantom(text:String,force:Boolean=false)=spacing.run{phantomSpace.determine(text,force)}
    fun run(op:String,vararg args:String):Boolean {
        sync()
        return when(op) {
            "typeChar" -> {
                val spaces=spacing.commitChar(args[0]);sync()
                base.commitChar(args[0],spaces.deletePreviousSpace,spaces.insertSpaceBeforeChar,spaces.insertSpaceAfterChar)
            }
            "typeText" -> {
                val space=phantom(args[0]);spacing.autoSpace.setInactive();spacing.phantomSpace.setInactive();sync()
                base.commitText(if(space)" "+args[0]else args[0])
            }
            "completion" -> {
                val text=args[0]
                if(text.isEmpty()||isRawInputEditor)false
                else if(base.activeContent.composing.isValid) {
                    spacing.phantomSpace.setActive(false);sync();base.finalizeComposingText(text)
                } else {
                    val space=phantom(text);spacing.phantomSpace.setActive(false);sync()
                    base.commitText(if(space)" $text"else text).also{base.updateLastCommitPosition()}
                }
            }
            "gesture" -> {
                val text=args[0]
                if(text.isEmpty()||isRawInputEditor)false else {
                    val space=phantom(text,true);spacing.phantomSpace.setActive(true);sync()
                    base.commitText(if(space)" $text"else text).also{base.updateLastCommitPosition()}
                }
            }
            "backward","forward" -> {
                var words=args[0]=="1"
                if(op=="backward"&&!words&&spacing.phantomSpace.isActive&&base.activeContent.currentWord.isValid&&immediateWord)words=true
                spacing.autoSpace.setInactive();spacing.phantomSpace.setInactive();sync()
                if(base.activeContent.selection.isSelectionMode) base.commitText("")
                else runBlocking{base.deleteAroundCursor(if(words)OperationUnit.WORDS else OperationUnit.CHARACTERS,if(op=="forward")OperationScope.AFTER_CURSOR else OperationScope.BEFORE_CURSOR,1)}
            }
            "revert" -> {
                spacing.autoSpace.setInactive()
                if(args[2]=="1")spacing.phantomSpace.setActive(false)else spacing.phantomSpace.setInactive()
                sync();base.replaceTextBeforeCursor(args[0].toInt(),args[1]).also{if(!it)spacing.phantomSpace.setInactive()}
            }
            "dictation" -> {
                val text=args[0];val length=args[1].toInt();val content=base.activeContent
                if(text.isEmpty())false else {
                    spacing.autoSpace.setInactive();spacing.phantomSpace.setActive(false);sync()
                    val replaced=length>0&&base.replaceTextBeforeCursor(length,text)
                    val committed=replaced||base.commitText(text)
                    if(!committed){spacing.phantomSpace.setInactive();false}else {
                        base.updateLastCommitPosition()
                        val restore=if(replaced)content.textBeforeSelection.takeLast(length)else ""
                        val start=content.selection.start-restore.length
                        undo=Triple(text,start,restore).takeIf{content.selection.isCursorMode&&start>=0};true
                    }
                }
            }
            "undoDictation" -> {
                val previous=undo;undo=null
                val c=base.activeContent
                if(previous==null||!c.selection.isCursorMode||c.selection.start!=previous.second+previous.first.length||!c.textBeforeSelection.endsWith(previous.first))false
                else{spacing.autoSpace.setInactive();spacing.phantomSpace.setInactive();sync();base.replaceTextBeforeCursor(previous.first.length,previous.third)}
            }
            "doubleSpace" -> {
                run("backward","0")
                run("typeText",". ")
            }
            "selectAround" -> {
                spacing.autoSpace.setInactive();spacing.phantomSpace.setInactive();sync()
                val n=args[0].toInt();val words=args[1]=="1";val after=args[2]=="1"
                val c=base.activeContent;val s=c.selection
                if(!s.isValid)false else {
                    val text=if(after)c.text.substring(c.localSelection.start)else c.text.substring(0,c.localSelection.end)
                    val length=if(n<=0)0 else runBlocking {
                        if(after) {if(words)base.breakIterators.measureUWords(text,n)else base.breakIterators.measureUChars(text,n)}
                        else {if(words)base.breakIterators.measureLastUWords(text,n)else base.breakIterators.measureLastUChars(text,n)}
                    }
                    val a=if(after)s.start else (s.end-length).coerceAtLeast(c.safeEditorBounds.start)
                    val b=if(after)(s.start+length).coerceAtMost(c.safeEditorBounds.end)else s.end
                    base.setSelection(ReferenceEditorRange.normalized(a,b))
                }
            }
            "enter" -> {
                spacing.autoSpace.setInactive();spacing.phantomSpace.setInactive();sync()
                if(isRawInputEditor)base.sendDownUpKeyEvent(66)else base.commitText("\n")
            }
            "selectAll" -> {
                spacing.autoSpace.setInactive();spacing.phantomSpace.setInactive();sync();base.ic.finishComposingText()
                if(isRawInputEditor)base.sendDownUpKeyEvent(29,base.meta(ctrl=true))else base.ic.performContextMenuAction(16908319)
            }
            else -> error("Unknown reference event")
        }
    }
    fun update() {
        spacing.autoSpace.setInactiveFromUpdate();spacing.phantomSpace.setInactiveFromUpdate();sync()
    }
}
