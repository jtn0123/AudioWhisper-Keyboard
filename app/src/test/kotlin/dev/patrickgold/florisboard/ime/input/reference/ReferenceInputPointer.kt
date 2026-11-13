package dev.patrickgold.florisboard.ime.input.reference

import dev.patrickgold.florisboard.ime.text.key.KeyCode

/** Original Compose popup suitability, rolling input, sliding bounds and hold decisions. */
internal object ReferenceInputPointer {
    fun suitable(code:Int)=(code>KeyCode.SPACE && code!=KeyCode.CJK_SPACE) || code in listOf(
        KeyCode.ENTER,KeyCode.LANGUAGE_SWITCH,KeyCode.IME_UI_MODE_TEXT,KeyCode.IME_UI_MODE_MEDIA,
        KeyCode.IME_UI_MODE_CLIPBOARD,KeyCode.CHAR_WIDTH_SWITCHER)
    fun slide(v:FloatArray):Boolean {
        val width=v[4]-v[2];val height=v[5]-v[3]
        return v[0]<v[2]-0.1f*width || v[0]>v[4]+0.1f*width || v[1]<v[3]-0.35f*height || v[1]>v[5]+0.35f*height
    }
    fun hold(code:Int,popup:Boolean,uninterrupted:Boolean,spaceAction:Boolean):Int = when(code) {
        KeyCode.SPACE,KeyCode.CJK_SPACE->32 or if(spaceAction)1 else 0
        KeyCode.SHIFT->if(uninterrupted)2 or 16 else 0
        KeyCode.LANGUAGE_SWITCH->4 or 32
        else->if(suitable(code)&&popup)8 or 16 or 32 else 0
    }
}
