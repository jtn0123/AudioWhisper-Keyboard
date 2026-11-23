package com.jtn0123.audiowhisper.render

import android.content.Context
import dev.patrickgold.florisboard.ime.theme.FlorisAssetResolver
import dev.patrickgold.florisboard.ime.theme.ThemeManager
import org.florisboard.lib.snygg.SnyggAnnotationRule
import org.florisboard.lib.snygg.SnyggMultiplePropertySets
import org.florisboard.lib.snygg.SnyggStylesheet
import org.florisboard.lib.snygg.value.SnyggAssetResolver
import org.florisboard.lib.snygg.value.SnyggUriValue

/** The theme [themeFontFiles] last resolved and its answer; resolving canonicalises paths on disk. Main thread only. */
private var resolved: Pair<ThemeManager.ThemeInfo, List<String>>? = null

/** [info]'s `@font` files ([fontFiles]) through its asset resolver, kept while [info] stays the theme asked for. */
internal fun themeFontFiles(context: Context, info: ThemeManager.ThemeInfo): List<String> {
    resolved?.takeIf { it.first === info }?.let { return it.second }
    return fontFiles(info.stylesheet, FlorisAssetResolver(context, info)).also { resolved = info to it }
}

/**
 * [stylesheet]'s `@font` files as the renderer reads them (`KeyboardRenderer.theme`'s `fonts`): each `src` URI
 * followed by the absolute path [resolver] gives it. A file that does not resolve is left out, and the renderer
 * draws that font's elements in its default fonts, as Compose does.
 */
internal fun fontFiles(stylesheet: SnyggStylesheet, resolver: SnyggAssetResolver): List<String> =
    stylesheet.rules.filterKeys { it is SnyggAnnotationRule.Font }.values
        .filterIsInstance<SnyggMultiplePropertySets>().flatMap { it.sets }
        .mapNotNull { (it.src as? SnyggUriValue)?.uri }.distinct()
        .flatMap { uri -> resolver.resolveAbsolutePath(uri).getOrNull()?.let { listOf(uri, it) }.orEmpty() }
