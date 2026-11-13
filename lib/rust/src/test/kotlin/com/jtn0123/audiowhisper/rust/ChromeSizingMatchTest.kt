package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals

/** Frozen AppPrefs.fontSizeMultiplier and ProvideKeyboardRowBaseHeight arithmetic. */
private object ReferenceChromeSizing {
    fun font(base: Int, compact: Int, enabled: Boolean, portrait: Boolean): Float =
        base/100f * if(enabled && portrait) compact/100f else 1f
    fun rows(base: Float, portrait: Int, landscape: Int, compact: Int, enabled: Boolean, isLandscape: Boolean): FloatArray {
        val row=base * if(isLandscape) landscape/100f else portrait/100f * if(enabled) compact/100f else 1f
        return floatArrayOf(row,row*.753f)
    }
}
class ChromeSizingMatchTest {
    @Test fun fontScaleMatchesEveryConfiguredCompactPercentageAndBothOrientations() {
        for(base in 40..160) for(compact in 50..100) for(enabled in 0..1) for(portrait in 0..1) {
            assertEquals(ReferenceChromeSizing.font(base,compact,enabled!=0,portrait!=0),
                ImeChromeRules.fontMultiplier(intArrayOf(base,compact,enabled,portrait)))
        }
    }
    @Test fun compactRowsAndSmartbarPreserveMultiplicationOrderAndLandscapeExclusion() {
        for(base in listOf(40f,101.3f,180.7f)) for(portrait in listOf(80,100,130))
            for(landscape in listOf(80,100,130)) for(compact in listOf(50,80,100))
                for(enabled in 0..1) for(isLandscape in 0..1) {
                    assertContentEquals(ReferenceChromeSizing.rows(base,portrait,landscape,compact,enabled!=0,isLandscape!=0),
                        ImeChromeRules.rowHeights(base,intArrayOf(portrait,landscape,compact,enabled,isLandscape)))
                }
    }
}
