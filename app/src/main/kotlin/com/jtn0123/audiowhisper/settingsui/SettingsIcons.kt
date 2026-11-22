package com.jtn0123.audiowhisper.settingsui

import android.content.res.Resources
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.HelpOutline
import androidx.compose.material.icons.automirrored.filled.Input
import androidx.compose.material.icons.automirrored.filled.LibraryBooks
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.automirrored.outlined.Assignment
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.Build
import androidx.compose.material.icons.outlined.Description
import androidx.compose.material.icons.outlined.EmojiSymbols
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Keyboard
import androidx.compose.material.icons.outlined.Palette
import androidx.compose.material.icons.outlined.Policy
import androidx.compose.material.icons.outlined.Schedule
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import dev.patrickgold.florisboard.R
import com.jtn0123.audiowhisper.render.iconPath

/** Existing Material resource vectors, copied as paths; Rust places and colours them. */
internal fun settingsIcons(resources: Resources): Array<String> {
    val home = listOf(Icons.Default.Language, Icons.Outlined.Palette, Icons.Outlined.Keyboard, Icons.Default.SmartButton,
        Icons.Default.Mic, Icons.Default.Spellcheck, Icons.Default.Gesture, Icons.AutoMirrored.Outlined.Assignment,
        Icons.Default.SentimentSatisfiedAlt, Icons.Default.Extension, Icons.Outlined.Build, Icons.Outlined.Info)
    return (home.mapIndexed { index, icon -> "@icon:home$index" to icon } + listOf(
        "@icon:back" to Icons.AutoMirrored.Filled.ArrowBack, "@icon:picker" to Icons.Default.Keyboard,
        // Row icons, named as the native screens' catalogue names them.
        "@icon:schedule" to Icons.Outlined.Schedule, "@icon:emoji_symbols" to Icons.Outlined.EmojiSymbols,
        // Group 2: Typing (with its spell checker card) and About.
        "@icon:incognito" to ImageVector.vectorResource(null, resources, R.drawable.ic_incognito),
        "@icon:space_bar" to Icons.Default.SpaceBar, "@icon:language" to Icons.Default.Language,
        "@icon:contacts" to Icons.Default.Contacts, "@icon:library_books" to Icons.AutoMirrored.Filled.LibraryBooks,
        "@icon:info" to Icons.Outlined.Info, "@icon:history" to Icons.Default.History, "@icon:code" to Icons.Default.Code,
        "@icon:policy" to Icons.Outlined.Policy, "@icon:description" to Icons.Outlined.Description,
        "@icon:error_outline" to Icons.Default.ErrorOutline, "@icon:warning" to Icons.Outlined.Warning,
        "@icon:help_outline" to Icons.AutoMirrored.Filled.HelpOutline,
        // Other, and the colour picker's text field.
        "@icon:palette" to Icons.Default.Palette, "@icon:format_color_fill" to Icons.Default.FormatColorFill,
        "@icon:keyboard_keys" to ImageVector.vectorResource(null, resources, R.drawable.ic_keyboard_keys),
        "@icon:archive" to Icons.Default.Archive, "@icon:settings_backup_restore" to Icons.Default.SettingsBackupRestore,
        "@icon:edit" to Icons.Default.Edit, "@icon:done" to Icons.Default.Done, "@icon:tune" to Icons.Default.Tune,
        "@icon:content_copy" to Icons.Default.ContentCopy, "@icon:clear" to Icons.Default.Clear,
        // The time picker's switch back from the keyboard to the dial.
        "@icon:clock" to Icons.Default.Schedule,
        // Theme, and its addon box's button.
        "@icon:brightness_auto" to Icons.Default.BrightnessAuto, "@icon:light_mode" to Icons.Default.LightMode,
        "@icon:dark_mode" to Icons.Default.DarkMode, "@icon:wb_twilight" to Icons.Default.WbTwilight,
        "@icon:brightness_2" to Icons.Default.Brightness2, "@icon:color_lens" to Icons.Default.ColorLens,
        "@icon:shop" to Icons.Default.Shop,
        // The language screens and their editable list.
        "@icon:add" to Icons.Default.Add, "@icon:delete" to Icons.Default.Delete, "@icon:arrow_upward" to Icons.Default.ArrowUpward,
        "@icon:arrow_downward" to Icons.Default.ArrowDownward, "@icon:arrow_drop_down" to Icons.Default.ArrowDropDown,
        "@icon:search" to Icons.Default.Search, "@icon:input" to Icons.AutoMirrored.Filled.Input,
        "@icon:open_in_new" to Icons.AutoMirrored.Filled.OpenInNew,
    )).flatMap { (name, icon) -> listOf(name, iconPath(icon)) }.toTypedArray()
}
