package com.jtn0123.audiowhisper.settingsui

import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponent
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.ext.ExtensionJsonConfig
import dev.patrickgold.florisboard.lib.ext.validate
import kotlinx.serialization.encodeToString
import org.florisboard.lib.kotlin.mimeTypeFilterOf

/**
 * Test-only copies of what the frozen Compose extension editor decided, kept when ExtensionEditScreen,
 * ExtensionEditFilesScreen and ExtensionComponentView were deleted (compose-removal step 10, part 3). Each is the
 * deleted code with its Compose calls taken out; SettingsExtensionEditorMatchTest holds Rust to them.
 * ExtensionComponentView's text is [ReferenceExtensionScreens.componentText], kept since part 1.
 */
internal object ReferenceExtensionEditor {
    /** EditScreen's `handleSave`: null where it showed the invalid meta data dialog, a failure where `editor.build()`
     * threw, else the `extension.json` it wrote into the archive. */
    fun handleSave(editor: ThemeExtensionEditor): Result<String>? {
        if (!editor.meta.validate()) return null
        return runCatching { ExtensionJsonConfig.encodeToString<Extension>(editor.build()) }
    }

    /** CreateComponentScreen's id for a copy of component [componentId]: the id itself while no theme of [themes] has
     * it, else the id with the first `_n` suffix none has. */
    fun copyId(themes: List<ThemeExtensionComponent>, componentId: String): String =
        if (themes.any { it.id == componentId }) {
            var suffix = 1
            var tempId: String
            do {
                tempId = "${componentId}_${suffix++}"
            } while (themes.any { it.id == tempId })
            tempId
        } else {
            componentId
        }

    /** ExtensionEditFilesScreen's `MIME_TYPES`: what each folder's Add took. */
    val MIME_TYPES = mapOf(
        "fonts" to mimeTypeFilterOf(
            "font/*",
            "application/font-*",
            "application/x-font-*",
            "application/vnd.ms-fontobject",
        ),
        "images" to mimeTypeFilterOf(
            "image/*",
        ),
    )
}
