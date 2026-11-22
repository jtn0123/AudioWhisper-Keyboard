package dev.patrickgold.florisboard.ime.theme

import android.content.res.Configuration
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import java.time.LocalTime
import org.florisboard.lib.kotlin.io.FsDir
import org.florisboard.lib.snygg.SnyggStylesheet

/**
 * Main's ThemeManager.updateActiveTheme and evaluateActiveThemeName (f4aa9fc30), frozen, with what
 * it read from Android injected: the settings, the clock, the configuration's night mode, the
 * extension lookup and the archive read. The theme infos it makes are main's, with main's base
 * style ([ReferenceFlorisImeThemeBaseStyle]).
 */
internal class ReferenceThemeManager(private val platform: Platform) {
    interface Platform {
        val mode: ThemeMode
        val dayThemeId: ExtensionComponentName
        val nightThemeId: ExtensionComponentName
        val sunriseTime: LocalTime
        val sunsetTime: LocalTime
        val now: LocalTime
        val uiMode: Int

        /** `ExtensionManager.getExtensionById`: a [ThemeExtension], or anything else it found. */
        fun getExtensionById(id: String): Any?

        /** `themeExt.sourceRef`. */
        fun sourceRefOf(extension: ThemeExtension): Any?

        /** The new loaded directory, as main made one for each load. */
        fun newLoadedDir(): FsDir

        /** Unzips [extension] into [dir] and reads the stylesheet at [path] in it, or throws. */
        fun read(extension: ThemeExtension, dir: FsDir, path: String): String
    }

    val previewThemeId = MutableValue<ExtensionComponentName?>(null)
    val previewThemeInfo = MutableValue<ThemeManager.ThemeInfo?>(null)

    private val cachedThemeInfos = mutableListOf<ThemeManager.ThemeInfo>()
    var activeThemeInfo: ThemeManager.ThemeInfo = DEFAULT
        private set

    class MutableValue<T>(var value: T)

    fun updateActiveTheme(action: () -> Unit = { }) {
        action()
        previewThemeInfo.value?.let { previewThemeInfo ->
            activeThemeInfo = previewThemeInfo
            return
        }
        val activeName = evaluateActiveThemeName()
        val cachedInfo = cachedThemeInfos.find { it.name == activeName }
        if (cachedInfo != null) {
            activeThemeInfo = cachedInfo
            return
        }
        val themeExt = platform.getExtensionById(activeName.extensionId) as? ThemeExtension
        val themeExtRef = themeExt?.let(platform::sourceRefOf)
        if (themeExtRef == null) {
            return
        }
        val themeConfig = themeExt.themes.find { it.id == activeName.componentId }
        if (themeConfig == null) {
            return
        }
        val loadedDir = platform.newLoadedDir()
        runCatching {
            val stylesheetJson = platform.read(themeExt, loadedDir, themeConfig.stylesheetPath())
            SnyggStylesheet.fromJson(stylesheetJson).getOrThrow()
        }.fold(
            onSuccess = { newStylesheet ->
                val newInfo = ThemeManager.ThemeInfo(activeName, themeConfig, newStylesheet, loadedDir, null)
                cachedThemeInfos.add(newInfo)
                activeThemeInfo = newInfo
            },
            onFailure = { cause ->
                activeThemeInfo = DEFAULT.copy(
                    loadFailure = ThemeManager.LoadFailure(themeExt.meta, themeConfig, cause)
                )
            },
        )
    }

    fun clearCache() = cachedThemeInfos.clear()

    private fun evaluateActiveThemeName(): ExtensionComponentName {
        previewThemeId.value?.let { return it }
        return when (platform.mode) {
            ThemeMode.ALWAYS_DAY -> {
                platform.dayThemeId
            }
            ThemeMode.ALWAYS_NIGHT -> {
                platform.nightThemeId
            }
            ThemeMode.FOLLOW_SYSTEM -> if (platform.uiMode and
                Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES
            ) {
                platform.nightThemeId
            } else {
                platform.dayThemeId
            }
            ThemeMode.FOLLOW_TIME -> {
                val current = platform.now
                val sunrise = platform.sunriseTime
                val sunset = platform.sunsetTime
                if (current in sunrise..sunset) {
                    platform.dayThemeId
                } else {
                    platform.nightThemeId
                }
            }
        }
    }

    companion object {
        val DEFAULT = ThemeManager.ThemeInfo(
            name = extCoreTheme("base"),
            config = ThemeExtensionComponentImpl(id = "base", label = "Base", authors = listOf()),
            stylesheet = ReferenceFlorisImeThemeBaseStyle,
            loadedDir = null,
            loadFailure = null,
        )
    }
}
