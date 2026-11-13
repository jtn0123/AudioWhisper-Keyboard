package dev.patrickgold.florisboard.ime.input.reference

import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.keyboard.ReferenceKeyboardState
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.input.InputShiftState

/** Frozen ComputingEvaluator label/icon decisions with Android resources represented by tokens. */
internal object ReferenceInputVisual {
    fun label(type:Int,code:Int,mode:Int):Int {
        if(type==0 && code!=KeyCode.SPACE && code!=KeyCode.CJK_SPACE && code!=KeyCode.HALF_SPACE && code!=KeyCode.KESHIDA || type==7) return 1
        return when(code) {
            KeyCode.SPACE,KeyCode.CJK_SPACE->if(mode==0)2 else 0
            KeyCode.PHONE_PAUSE,KeyCode.PHONE_WAIT,KeyCode.IME_UI_MODE_TEXT,KeyCode.VIEW_CHARACTERS,
            KeyCode.VIEW_NUMERIC,KeyCode.VIEW_NUMERIC_ADVANCED,KeyCode.VIEW_PHONE,KeyCode.VIEW_PHONE2,
            KeyCode.VIEW_SYMBOLS,KeyCode.VIEW_SYMBOLS2,KeyCode.HALF_SPACE,KeyCode.KESHIDA->code
            else->0
        }
    }
    fun icon(code:Int,raw:Long,input:Int,options:Int,mode:Int):Int {
        val state=ReferenceKeyboardState.new(raw.toULong())
        return when(code) {
            KeyCode.ENTER->if(options and 0x40000000!=0 || input and 0x20000!=0)1000 else when(options and 255) {
                6->1001;2,5,7->1002;3->1003;4->1004;else->1000
            }
            KeyCode.SHIFT->if(state.inputShiftState!=InputShiftState.UNSHIFTED)1006 else KeyCode.SHIFT
            KeyCode.CAPS_LOCK->0
            KeyCode.SPACE,KeyCode.CJK_SPACE->when(KeyboardMode.fromInt(mode)) {
                KeyboardMode.NUMERIC,KeyboardMode.NUMERIC_ADVANCED,KeyboardMode.PHONE,KeyboardMode.PHONE2->KeyCode.SPACE
                else->0
            }
            KeyCode.TOGGLE_INCOGNITO_MODE->if(state.isIncognitoMode)KeyCode.TOGGLE_INCOGNITO_MODE else 1005
            KeyCode.CHAR_WIDTH_SWITCHER->if(state.isCharHalfWidth)KeyCode.CHAR_WIDTH_FULL else KeyCode.CHAR_WIDTH_HALF
            else->code
        }
    }
}
