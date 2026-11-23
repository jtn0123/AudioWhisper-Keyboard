/*
 * Copyright (C) 2020-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.ime.theme

import android.annotation.SuppressLint
import android.content.Context
import android.content.res.Configuration
import android.graphics.drawable.Icon
import android.os.Bundle
import androidx.autofill.inline.UiVersions
import androidx.autofill.inline.common.ImageViewStyle
import androidx.autofill.inline.common.TextViewStyle
import androidx.autofill.inline.common.ViewStyle
import androidx.autofill.inline.v1.InlineSuggestionUi
import androidx.compose.ui.graphics.isSpecified
import androidx.compose.ui.graphics.toArgb
import androidx.core.graphics.ColorUtils
import com.jtn0123.audiowhisper.render.inlineChipColors
import com.jtn0123.audiowhisper.render.themePalette
import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeRule
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import dev.patrickgold.florisboard.lib.io.ZipUtils
import dev.patrickgold.florisboard.lib.util.TimeUtils.javaLocalTime
import java.time.LocalTime
import java.util.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.florisboard.lib.kotlin.collectIn
import org.florisboard.lib.kotlin.io.FsDir
import org.florisboard.lib.kotlin.io.deleteContentsRecursively
import org.florisboard.lib.kotlin.io.subDir
import org.florisboard.lib.kotlin.io.subFile
import org.florisboard.lib.snygg.SnyggStylesheet
import java.util.concurrent.atomic.AtomicInteger

/**
 * The keyboard's theme. Rust (crates/imetheme, through [ThemeUpdates]) chooses which theme is
 * active and what to load; this reports the settings, the clock and the system's night mode,
 * unpacks the archives and reads the stylesheets Rust names, and builds the inline autofill
 * styles from the colours Rust resolves.
 */
class ThemeManager(context: Context) {
    private val prefs by FlorisPreferenceStore
    private val appContext by context.appContext()
    private val extensionManager by context.extensionManager()
    private val keyboardManager by context.keyboardManager()

    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    private val _indexedThemeConfigs = MutableStateFlow(mapOf<ExtensionComponentName, ThemeExtensionComponent>() to 0)
    val indexedThemeConfigs get() = _indexedThemeConfigs.asStateFlow()
    private val indexedThemeConfigVersion = AtomicInteger(0)

    val previewThemeId = MutableStateFlow<ExtensionComponentName?>(null)
    val previewThemeInfo = MutableStateFlow<ThemeInfo?>(null)
    val configurationChangeCounter = MutableStateFlow(0)

    private val updates = ThemeUpdates()
    private val activeThemeGuard = Mutex(locked = false)
    private val _activeThemeInfo = MutableStateFlow(ThemeInfo.DEFAULT)
    val activeThemeInfo get() = _activeThemeInfo.asStateFlow()

    init {
        extensionManager.themes.observeForever { themeExtensions ->
            val version = indexedThemeConfigVersion.incrementAndGet()
            _indexedThemeConfigs.value = buildMap {
                for (themeExtension in themeExtensions) {
                    for (themeComponent in themeExtension.themes) {
                        put(ExtensionComponentName(themeExtension.meta.id, themeComponent.id), themeComponent)
                    }
                }
            } to version
        }
        indexedThemeConfigs.collectIn(scope) {
            updateActiveTheme(clear = true)
        }
        combine(
            prefs.theme.mode.asFlow(),
            prefs.theme.dayThemeId.asFlow(),
            prefs.theme.nightThemeId.asFlow(),
            previewThemeId,
            previewThemeInfo,
            configurationChangeCounter,
        ) {}.collectIn(scope) {
            updateActiveTheme()
        }
    }

    /** Shows the theme Rust chooses, loading it first if Rust asks; [clear] drops every loaded theme first. */
    private suspend fun updateActiveTheme(clear: Boolean = false) = activeThemeGuard.withLock {
        val themes = extensionManager.themes.value.orEmpty()
        val sources = themes.map { ThemeSource(it.meta, it.sourceRef != null, it.themes) }
        val keyboards = extensionManager.keyboardExtensions.value.orEmpty().map { it.meta.id }
        val next = updates.update(clear, facts(), previewThemeInfo.value, keyboards, sources) { index, path ->
            load(themes[index], path)
        }
        if (next != null) _activeThemeInfo.value = next
    }

    private fun facts(): ThemeFacts {
        val uiMode = appContext.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK
        return ThemeFacts(
            mode = prefs.theme.mode.get(),
            day = prefs.theme.dayThemeId.get(),
            night = prefs.theme.nightThemeId.get(),
            sunrise = prefs.theme.sunriseTime.get().javaLocalTime.toNanoOfDay(),
            sunset = prefs.theme.sunsetTime.get().javaLocalTime.toNanoOfDay(),
            now = LocalTime.now().toNanoOfDay(),
            systemNight = uiMode == Configuration.UI_MODE_NIGHT_YES,
            previewId = previewThemeId.value,
        )
    }

    // TODO: this leaks the loaded dir, but at least the state is not kaputt from compose viewpoint
    private fun load(extension: ThemeExtension, path: String): Result<LoadedTheme> {
        val loadedDir = appContext.cacheDir.subDir("loaded").subDir(UUID.randomUUID().toString())
        return runCatching {
            loadedDir.mkdirs()
            loadedDir.deleteContentsRecursively()
            ZipUtils.unzip(appContext, checkNotNull(extension.sourceRef), loadedDir).getOrThrow()
            LoadedTheme(SnyggStylesheet.fromJson(loadedDir.subFile(path).readText()).getOrThrow(), loadedDir)
        }
    }

    /**
     * Creates a new inline suggestion UI bundle, styled with the chip colours Rust resolves in
     * the active theme for the keyboard's state.
     *
     * @param context The context of the parent view/controller.
     *
     * @return A bundle containing all necessary attributes for the inline suggestion views to properly display.
     */
    // A lint false positive, kept narrow: setBackground, setPadding and setLayoutMargin are the public style builders'
    // own methods, inherited from a base class the library marks restricted. They are the only way to style inline
    // autofill suggestions.
    @SuppressLint("RestrictedApi")
    fun createInlineSuggestionUiStyleBundle(context: Context): Bundle {
        val info = activeThemeInfo.value
        val state = keyboardManager.activeState
        val palette = themePalette(preferredAccent(), systemAccent(context), info.config.materialYouFlags)
        val (bgColor, fgColor) = inlineChipColors(info, state.keyboardMode.toString(), state.inputShiftState.toString(), palette)

        val bgDrawableId = R.drawable.inline_suggestion_chip_background
        val bgDrawable = Icon.createWithResource(context, bgDrawableId).apply {
            setTint(bgColor)
        }
        val chipStyle = ViewStyle.Builder().run {
            setBackground(bgDrawable)
            setPadding(
                context.resources.getDimension(R.dimen.suggestion_chip_bg_padding_start).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_bg_padding_top).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_bg_padding_end).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_bg_padding_bottom).toInt(),
            )
            build()
        }
        val iconStyle = ImageViewStyle.Builder().run {
            setLayoutMargin(0, 0, 0, 0)
            build()
        }
        val titleStyle = TextViewStyle.Builder().run {
            setLayoutMargin(
                context.resources.getDimension(R.dimen.suggestion_chip_fg_title_margin_start).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_fg_title_margin_top).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_fg_title_margin_end).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_fg_title_margin_bottom).toInt(),
            )
            setTextColor(fgColor)
            setTextSize(16f)
            build()
        }
        val subtitleStyle = TextViewStyle.Builder().run {
            setLayoutMargin(
                context.resources.getDimension(R.dimen.suggestion_chip_fg_subtitle_margin_start).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_fg_subtitle_margin_top).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_fg_subtitle_margin_end).toInt(),
                context.resources.getDimension(R.dimen.suggestion_chip_fg_subtitle_margin_bottom).toInt(),
            )
            setTextColor(ColorUtils.setAlphaComponent(fgColor, 150))
            setTextSize(14f)
            build()
        }
        val suggestionStyle = InlineSuggestionUi.newStyleBuilder().run {
            setSingleIconChipStyle(chipStyle)
            setChipStyle(chipStyle)
            setStartIconStyle(iconStyle)
            setEndIconStyle(iconStyle)
            setTitleStyle(titleStyle)
            setSubtitleStyle(subtitleStyle)
            build()
        }
        return UiVersions.newStylesBuilder().run {
            addStyle(suggestionStyle)
            build()
        }
    }

    /** The accent the settings name for Material You themes, as an ARGB colour; null when they name none. */
    fun preferredAccent(): Int? = prefs.theme.accentColor.get().takeIf { it.isSpecified }?.toArgb()

    /** The system's accent colour, which a theme's palette is made from unless the settings name one. */
    private fun systemAccent(context: Context) = context.resources.getColor(android.R.color.system_accent1_500, context.theme)

    data class ThemeInfo(
        val name: ExtensionComponentName,
        val config: ThemeExtensionComponent,
        val stylesheet: SnyggStylesheet,
        val loadedDir: FsDir?,
        val loadFailure: LoadFailure?,
    ) {
        // Derived per instance: data-class copy(stylesheet = draft) must serialize that draft.
        val stylesheetJson: String by lazy { stylesheet.toJson().getOrThrow() }
        override fun toString(): String {
            return "ThemeInfo(name=$name, config=$config, loadedDir=$loadedDir)"
        }

        companion object {
            /** The base theme: the stylesheet Rust holds (crates/imetheme), shown when no other loads. */
            val DEFAULT = ThemeInfo(
                name = extCoreTheme("base"),
                config = ThemeExtensionComponentImpl(id = "base", label = "Base", authors = listOf()),
                stylesheet = SnyggStylesheet.fromJson(ImeTheme.rules(ImeThemeRule.BASE).single()).getOrThrow(),
                loadedDir = null,
                loadFailure = null,
            )
        }
    }

    data class LoadFailure(
        val extension: ExtensionMeta,
        val component: ThemeExtensionComponent,
        val cause: Throwable,
    )
}
