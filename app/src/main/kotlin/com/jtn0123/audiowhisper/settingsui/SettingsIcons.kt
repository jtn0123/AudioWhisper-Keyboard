package com.jtn0123.audiowhisper.settingsui

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.Assignment
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.Build
import androidx.compose.material.icons.outlined.EmojiSymbols
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Keyboard
import androidx.compose.material.icons.outlined.Palette
import androidx.compose.material.icons.outlined.Schedule
import com.jtn0123.audiowhisper.render.iconPath

/** Existing Material resource vectors, copied as paths; Rust places and colours them. */
internal fun settingsIcons(): Array<String> {
    val home = listOf(Icons.Default.Language, Icons.Outlined.Palette, Icons.Outlined.Keyboard, Icons.Default.SmartButton,
        Icons.Default.Mic, Icons.Default.Spellcheck, Icons.Default.Gesture, Icons.AutoMirrored.Outlined.Assignment,
        Icons.Default.SentimentSatisfiedAlt, Icons.Default.Extension, Icons.Outlined.Build, Icons.Outlined.Info)
    return (home.mapIndexed { index, icon -> "@icon:home$index" to icon } + listOf(
        "@icon:back" to Icons.AutoMirrored.Filled.ArrowBack, "@icon:picker" to Icons.Default.Keyboard,
        // Row icons, named as the native screens' catalogue names them.
        "@icon:schedule" to Icons.Outlined.Schedule, "@icon:emoji_symbols" to Icons.Outlined.EmojiSymbols,
    )).flatMap { (name, icon) -> listOf(name, iconPath(icon)) }.toTypedArray()
}
