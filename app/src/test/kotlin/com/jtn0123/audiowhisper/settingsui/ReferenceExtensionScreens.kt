package com.jtn0123.audiowhisper.settingsui

import dev.patrickgold.florisboard.ime.nlp.LanguagePackComponent
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponent
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.ext.ExtensionComponent
import dev.patrickgold.florisboard.lib.ext.ExtensionMaintainer

/**
 * Test-only copies of what the frozen Compose extension screens decided, kept when ExtensionHomeScreen,
 * ExtensionListScreen, ExtensionViewScreen, CheckUpdatesScreen, AddonBox and the chips were deleted (compose-removal
 * step 10). Each is the deleted code with its Compose calls taken out; the match tests hold Rust to them.
 */
internal object ReferenceExtensionScreens {
    /** Extension.kt's `generateUpdateUrl`: `Uri.Builder` with the https scheme, the store's authority, the
     * `check-updates` path and an encoded fragment, which `Uri` keeps as written. The pair is
     * `ExtensionMeta.getUpdateJsonPair`. */
    fun updateUrl(list: List<Extension>, host: String = "beta.addons.florisboard.org") = "https://$host/check-updates#" +
        buildString {
            append("data={")
            for (extension in list) {
                append("\"${extension.meta.id}\":\"${extension.meta.version}\"")
                if (extension != list.last()) {
                    append(",")
                }
            }
            append("}")
        }

    /** The extensions a screen listed or checked for updates: `ExtensionListScreenType.getExtensionIndex` for a list,
     * `combinedExtensionList()` (keyboards, themes, language packs) for the home and CheckUpdatesScreen. */
    fun listed(route: String, keyboards: List<Extension>, themes: List<Extension>, packs: List<Extension>) =
        when (route.substringBefore('?')) {
            "ext/list/EXT_THEME" -> themes
            "ext/list/EXT_KEYBOARD" -> keyboards
            "ext/list/EXT_LANGUAGEPACK" -> packs
            else -> keyboards + themes + packs
        }

    /** Whether a screen showed ImportExtensionBox: the home always, a list given `showUpdate = true`. UpdateBox showed
     * there and on CheckUpdatesScreen. The route's `showUpdate` was `payload.showUpdate != null && payload.showUpdate`. */
    fun showsImport(route: String) = route == "ext" || route.substringAfter('?', "").split('&').contains("showUpdate=true")
    fun showsUpdate(route: String) = route == "ext/check-updates" || showsImport(route)

    /** ExtensionViewScreen's homepage and issue tracker rows. */
    fun showsLink(link: String?) = !link.isNullOrBlank()

    /** ExtensionViewScreen's keywords row. */
    fun showsKeywords(keywords: List<String>?) = keywords != null && keywords.isNotEmpty()

    /** ExtensionMaintainerChip's `enabled`. */
    fun maintainerEnabled(maintainer: ExtensionMaintainer) = maintainer.email != null || maintainer.url != null

    /** ExtensionMaintainerChip's dialog: the email as a `mailto:` link, then the website. */
    fun maintainerLinks(maintainer: ExtensionMaintainer) =
        listOfNotNull(maintainer.email?.let { "mailto:$it" }, maintainer.url)

    /** ExtensionComponentView's text for a component. */
    fun componentText(component: ExtensionComponent): String = when (component) {
        is ThemeExtensionComponent -> buildString {
            appendLine("authors = ${component.authors}")
            appendLine("isNightTheme = ${component.isNightTheme}")
            append("stylesheetPath = ${component.stylesheetPath()}")
        }
        is LanguagePackComponent -> buildString {
            appendLine("authors = ${component.authors}")
            appendLine("locale = ${component.locale.localeTag()}")
        }
        else -> ""
    }
}
