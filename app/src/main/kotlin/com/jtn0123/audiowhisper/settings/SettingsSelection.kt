package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.*

object SettingsSelection {
    fun <T> index(value: T, choices: List<T>): Int = SettingsRules.call(SettingsOp.CHOICE_INDEX,value.toString(),
        *choices.map { it.toString() }.toTypedArray())[0].toInt()
}
