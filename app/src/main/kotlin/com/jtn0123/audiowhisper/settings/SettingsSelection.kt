package com.jtn0123.audiowhisper.settings

import com.jtn0123.audiowhisper.rust.SettingsRules

object SettingsSelection {
    fun <T> index(value: T, choices: List<T>): Int = SettingsRules.call(30,value.toString(),
        *choices.map { it.toString() }.toTypedArray())[0].toInt()
}
