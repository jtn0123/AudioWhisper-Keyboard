package dev.patrickgold.florisboard.ime.theme

import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeComponent
import com.jtn0123.audiowhisper.rust.ImeThemeExtension
import com.jtn0123.audiowhisper.rust.ImeThemeLoaded
import com.jtn0123.audiowhisper.rust.ImeThemeOp
import com.jtn0123.audiowhisper.rust.ImeThemeStep
import com.jtn0123.audiowhisper.rust.ImeThemeStepKind
import com.jtn0123.audiowhisper.rust.ImeThemeUpdate
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import org.florisboard.lib.kotlin.io.FsDir
import org.florisboard.lib.snygg.SnyggStylesheet

/** What Android reports for a theme update: the settings, the clock and the system's night mode. */
internal data class ThemeFacts(
    val mode: ThemeMode,
    val day: ExtensionComponentName,
    val night: ExtensionComponentName,
    /** Sunrise, sunset and now, as nanoseconds of the day. */
    val sunrise: Long,
    val sunset: Long,
    val now: Long,
    val systemNight: Boolean,
    val previewId: ExtensionComponentName?,
)

/** A theme extension as Android indexed it: whether it has an archive, and its themes. */
internal class ThemeSource(val meta: ExtensionMeta, val hasSource: Boolean, val themes: List<ThemeExtensionComponent>)

/** A stylesheet Android read and parsed, and the directory its archive was unpacked to. */
internal class LoadedTheme(val stylesheet: SnyggStylesheet, val dir: FsDir)

/**
 * The theme manager's updates as Rust decides them (crates/imetheme): which theme shows, which
 * to load, and which loaded theme to show again. Android holds the loaded themes Rust names by
 * slot; [update]'s `load` reads one: the archive of the theme extension at an index in the list
 * it was given, and the stylesheet at a path in it.
 */
internal class ThemeUpdates(private val native: ImeTheme = ImeTheme()) {
    private val loaded = mutableMapOf<Int, ThemeManager.ThemeInfo>()

    /**
     * The theme to show after an update of [facts] over [keyboards] (keyboard extension ids) and
     * [themes], or null to keep the one showing. [clear] drops every loaded theme first; a
     * [preview] the theme editor holds wins.
     */
    fun update(
        clear: Boolean, facts: ThemeFacts, preview: ThemeManager.ThemeInfo?, keyboards: List<String>,
        themes: List<ThemeSource>, load: (Int, String) -> Result<LoadedTheme>,
    ): ThemeManager.ThemeInfo? {
        val step = native.call(ImeThemeOp.UPDATE, *rows(clear, facts, preview != null, keyboards, themes))
        if (WireFlag.decode(step[ImeThemeStep.CLEARED])) loaded.clear()
        return when (step[ImeThemeStep.KIND].toInt()) {
            ImeThemeStepKind.PREVIEW -> checkNotNull(preview)
            ImeThemeStepKind.CACHED -> loaded.getValue(step[ImeThemeStep.SLOT].toInt())
            ImeThemeStepKind.LOAD -> {
                val index = step[ImeThemeStep.EXTENSION].toInt()
                val source = themes[index]
                loaded(source, source.themes[step[ImeThemeStep.COMPONENT].toInt()], load(index, step[ImeThemeStep.PATH]))
            }
            else -> null
        }
    }

    private fun loaded(source: ThemeSource, config: ThemeExtensionComponent, result: Result<LoadedTheme>): ThemeManager.ThemeInfo {
        val ok = WireFlag.encode(result.isSuccess)
        val done = native.call(ImeThemeOp.LOADED, *ImeThemeLoaded.row(ImeThemeLoaded.OK to ok))
        if (done[ImeThemeStep.KIND].toInt() == ImeThemeStepKind.FAILED) {
            return ThemeManager.ThemeInfo.DEFAULT.copy(
                loadFailure = ThemeManager.LoadFailure(source.meta, config, checkNotNull(result.exceptionOrNull())),
            )
        }
        val theme = result.getOrThrow()
        val name = ExtensionComponentName(source.meta.id, config.id)
        return ThemeManager.ThemeInfo(name, config, theme.stylesheet, theme.dir, null)
            .also { loaded[done[ImeThemeStep.SLOT].toInt()] = it }
    }

    private fun rows(
        clear: Boolean, facts: ThemeFacts, previewInfo: Boolean, keyboards: List<String>, themes: List<ThemeSource>,
    ): Array<String> = buildList {
        addAll(ImeThemeUpdate.row(
            ImeThemeUpdate.CLEAR to WireFlag.encode(clear),
            ImeThemeUpdate.MODE to facts.mode.name,
            ImeThemeUpdate.DAY_EXTENSION to facts.day.extensionId,
            ImeThemeUpdate.DAY_COMPONENT to facts.day.componentId,
            ImeThemeUpdate.NIGHT_EXTENSION to facts.night.extensionId,
            ImeThemeUpdate.NIGHT_COMPONENT to facts.night.componentId,
            ImeThemeUpdate.SUNRISE to facts.sunrise.toString(),
            ImeThemeUpdate.SUNSET to facts.sunset.toString(),
            ImeThemeUpdate.NOW to facts.now.toString(),
            ImeThemeUpdate.SYSTEM_NIGHT to WireFlag.encode(facts.systemNight),
            ImeThemeUpdate.PREVIEW to WireFlag.encode(facts.previewId != null),
            ImeThemeUpdate.PREVIEW_EXTENSION to facts.previewId?.extensionId.orEmpty(),
            ImeThemeUpdate.PREVIEW_COMPONENT to facts.previewId?.componentId.orEmpty(),
            ImeThemeUpdate.PREVIEW_INFO to WireFlag.encode(previewInfo),
            ImeThemeUpdate.KEYBOARDS to keyboards.size.toString(),
            ImeThemeUpdate.THEMES to themes.size.toString(),
        ))
        addAll(keyboards)
        for (theme in themes) {
            addAll(ImeThemeExtension.row(
                ImeThemeExtension.ID to theme.meta.id,
                ImeThemeExtension.SOURCE to WireFlag.encode(theme.hasSource),
                ImeThemeExtension.COMPONENTS to theme.themes.size.toString(),
            ))
            for (component in theme.themes) addAll(ImeThemeComponent.row(
                ImeThemeComponent.ID to component.id,
                ImeThemeComponent.STYLESHEET to component.stylesheetPath.orEmpty(),
            ))
        }
    }.toTypedArray()
}
