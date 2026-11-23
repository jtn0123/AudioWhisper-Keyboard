package dev.patrickgold.florisboard.ime.theme

import android.content.res.Configuration
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.isUnspecified
import androidx.compose.ui.graphics.toArgb
import com.jtn0123.audiowhisper.render.RendererTheme
import com.jtn0123.audiowhisper.render.ThemePalette
import com.jtn0123.audiowhisper.render.referenceSnyggPalette
import com.jtn0123.audiowhisper.render.rendererTheme
import com.jtn0123.audiowhisper.render.themePalette
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamiccolor.ColorSpec
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import java.io.File
import java.io.FileNotFoundException
import java.time.LocalTime
import kotlin.coroutines.cancellation.CancellationException
import kotlin.random.Random
import kotlin.test.assertEquals
import kotlinx.serialization.json.Json
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.kotlin.io.FsDir
import org.florisboard.lib.snygg.SnyggStylesheet

/**
 * One seeded trace of theme updates, run by main's frozen theme manager ([ReferenceThemeManager])
 * and by [ThemeUpdates] over Rust (crates/imetheme) side by side in one synthetic world: theme
 * extensions that come and go, keyboard extensions that shadow them, archives whose stylesheets
 * change, fail to read, fail to parse or are cancelled, the settings, the clock and the system's
 * night mode, previews, and the service being made again. After every event the theme each shows
 * must be the same, field by field, and so must what the renderer is handed for it.
 */
internal class ThemeTrace(seed: Long) {
    private val random = Random(seed)
    val seen = mutableMapOf<String, Int>()
    private fun see(what: String) { seen[what] = (seen[what] ?: 0) + 1 }

    private val extensionIds = BUNDLED.map { it.meta.id } + listOf("x.alpha", "x.beta", "x.gamma")
    private val componentIds = BUNDLED.flatMap { e -> e.themes.map { it.id } } + listOf("c1", "c2", "base")
    private val paths = listOf(null, "", "  ", "custom/a.json", "stylesheets/c1.json")

    private var themes = BUNDLED.toMutableList()
    private val sources = mutableMapOf<ThemeExtension, Boolean>()
    private var keyboards = listOf<String>()
    private val files = mutableMapOf<Pair<String, String>, String?>()
    private var mode = ThemeMode.FOLLOW_SYSTEM
    private var day = extCoreTheme("floris_day")
    private var night = extCoreTheme("floris_night")
    private var sunrise = LocalTime.of(6, 0)
    private var sunset = LocalTime.of(18, 0)
    private var now = LocalTime.NOON
    private var uiMode = Configuration.UI_MODE_NIGHT_NO
    private var previewId: ExtensionComponentName? = null
    private var previewInfo: ThemeManager.ThemeInfo? = null
    private var accent = Color.Unspecified
    private var system = Color(0xFF6750A4)

    private var referenceDirs = 0
    private var productionDirs = 0
    private var reference = ReferenceThemeManager(platform())
    private var updates = ThemeUpdates()
    private var active = ThemeManager.ThemeInfo.DEFAULT
    private val palettes = mutableMapOf<Pair<Color, MaterialYouFlags>, ThemePalette>()

    init {
        for (extension in BUNDLED) for (component in extension.themes) {
            files[extension.meta.id to component.stylesheetPath()] = SHEETS.getValue(component.id)
        }
        BUNDLED.forEach { sources[it] = true }
    }

    fun run(states: Int) = repeat(states) {
        event()
        compare()
    }

    private fun event() {
        when (random.nextInt(100)) {
            in 0..11 -> { changeIndex(); update(clear = true) }
            in 12..36 -> { changeSettings(); update() }
            in 37..45 -> { previewId = if (random.nextInt(4) == 0) name() else null; see("preview id"); update() }
            in 46..50 -> { previewInfo = if (random.nextInt(8) == 0) preview() else null; see("preview info"); update() }
            in 51..60 -> { uiMode = if (random.nextBoolean()) Configuration.UI_MODE_NIGHT_YES else Configuration.UI_MODE_NIGHT_NO; update() }
            in 61..70 -> { now = time(); see("clock") }
            in 71..75 -> { keyboards = List(random.nextInt(3)) { extensionIds.random(random) }; see("keyboards") }
            in 76..85 -> { changeFile(); see("file") }
            in 86..88 -> recreate()
            in 89..90 -> see("cancelled before the lock")
            else -> { accent = if (random.nextBoolean()) Color.Unspecified else Color(random.nextInt() or (0xFF shl 24)); update() }
        }
    }

    private fun update(clear: Boolean = false) {
        reference.previewThemeId.value = previewId
        reference.previewThemeInfo.value = previewInfo
        if (clear) reference.updateActiveTheme { reference.clearCache() } else reference.updateActiveTheme()
        val sourcesNow = themes.map { ThemeSource(it.meta, sources[it] == true, it.themes) }
        val facts = ThemeFacts(mode, day, night, sunrise.toNanoOfDay(), sunset.toNanoOfDay(), now.toNanoOfDay(),
            uiMode and Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES, previewId)
        var loaded = false
        val next = updates.update(clear, facts, previewInfo, keyboards, sourcesNow) { index, path ->
            see("load")
            loaded = true
            val dir = File("dir-${productionDirs++}")
            runCatching { LoadedTheme(SnyggStylesheet.fromJson(read(themes[index], path)).getOrThrow(), dir) }
        }
        if (next != null) active = next else see("kept")
        if (next?.loadedDir != null && !loaded) see("cached")
    }

    private fun platform() = object : ReferenceThemeManager.Platform {
        override val mode get() = this@ThemeTrace.mode
        override val dayThemeId get() = day
        override val nightThemeId get() = night
        override val sunriseTime get() = sunrise
        override val sunsetTime get() = sunset
        override val now get() = this@ThemeTrace.now
        override val uiMode get() = this@ThemeTrace.uiMode
        override fun getExtensionById(id: String): Any? =
            keyboards.find { it == id }?.let { KEYBOARD } ?: themes.find { it.meta.id == id }
        override fun sourceRefOf(extension: ThemeExtension): Any? = if (sources[extension] == true) extension else null
        override fun newLoadedDir(): FsDir = File("dir-${referenceDirs++}")
        override fun read(extension: ThemeExtension, dir: FsDir, path: String) = read(extension, path)
    }

    private fun read(extension: ThemeExtension, path: String): String {
        val text = files[extension.meta.id to path] ?: throw FileNotFoundException(path)
        if (text == CANCEL) throw CancellationException("cancelled")
        return text
    }

    private fun recreate() {
        see("recreated")
        reference = ReferenceThemeManager(platform())
        updates = ThemeUpdates()
        active = ThemeManager.ThemeInfo.DEFAULT
        update(clear = true)
    }

    private fun changeIndex() {
        when (random.nextInt(5)) {
            0 -> if (themes.isNotEmpty()) { themes.removeAt(random.nextInt(themes.size)); see("extension removed") }
            1 -> { themes.add(random.nextInt(themes.size + 1), synthetic()); see("extension added") }
            2 -> { themes.shuffle(random); see("extensions reordered") }
            3 -> if (themes.isNotEmpty()) {
                val at = random.nextInt(themes.size)
                sources[themes[at]] = sources[themes[at]] != true
                see("source toggled")
            }
            else -> { themes = BUNDLED.toMutableList(); see("extensions reset") }
        }
    }

    private fun changeSettings() {
        when (random.nextInt(3)) {
            0 -> mode = ThemeMode.entries.random(random)
            1 -> day = name()
            else -> night = name()
        }
        if (random.nextInt(4) == 0) { sunrise = time(); sunset = time() }
        see("mode $mode")
    }

    private fun changeFile() {
        val extension = if (themes.isEmpty()) return else themes.random(random)
        val component = if (extension.themes.isEmpty()) return else extension.themes.random(random)
        files[extension.meta.id to component.stylesheetPath()] = when (random.nextInt(6)) {
            0 -> null
            1 -> CANCEL
            2 -> SHEETS.values.random(random)
            else -> SYNTHETIC.random(random)
        }
    }

    private fun synthetic(): ThemeExtension {
        val id = extensionIds.random(random)
        val components = List(random.nextInt(4)) {
            ThemeExtensionComponentImpl(componentIds.random(random), "Label $it", listOf(), random.nextBoolean(),
                MaterialYouFlags(PaletteStyle.entries.random(random), Contrast.entries.random(random),
                    ColorSpec.SpecVersion.entries.random(random)), paths.random(random))
        }
        return ThemeExtension(ExtensionMeta(id = id, version = "1", title = id, maintainers = listOf(), license = "x"),
            null, components).also { extension ->
            sources[extension] = random.nextInt(5) != 0
            for (component in components) {
                if (random.nextInt(4) != 0) files[id to component.stylesheetPath()] = (SYNTHETIC + SHEETS.values).random(random)
            }
        }
    }

    private fun preview(): ThemeManager.ThemeInfo {
        val stylesheet = SnyggStylesheet.fromJson(SHEETS.values.random(random)).getOrThrow()
        val config = ThemeExtensionComponentImpl("p", "Preview", listOf(), random.nextBoolean())
        return ThemeManager.ThemeInfo.DEFAULT.copy(name = extPreviewTheme("p"), config = config, stylesheet = stylesheet)
    }

    /** A theme's name: mostly one an extension has now, so it loads or is cached, else any. */
    private fun name(): ExtensionComponentName {
        val extension = themes.randomOrNull(random)
        val component = extension?.themes?.randomOrNull(random)
        if (component != null && random.nextInt(3) != 0) return ExtensionComponentName(extension.meta.id, component.id)
        return ExtensionComponentName(extensionIds.random(random), componentIds.random(random))
    }

    private fun time(): LocalTime = when (random.nextInt(4)) {
        0 -> sunrise
        1 -> sunset
        else -> LocalTime.ofSecondOfDay(random.nextLong(86_400))
    }

    private fun compare() {
        val expected = reference.activeThemeInfo
        assertEquals(expected.name, active.name)
        assertEquals(expected.config, active.config)
        assertEquals(expected.stylesheet, active.stylesheet)
        assertEquals(expected.loadedDir?.name, active.loadedDir?.name)
        assertEquals(expected.loadFailure?.extension, active.loadFailure?.extension)
        assertEquals(expected.loadFailure?.component, active.loadFailure?.component)
        assertEquals(expected.loadFailure?.cause?.javaClass, active.loadFailure?.cause?.javaClass)
        assertEquals(expected.loadFailure?.cause?.message, active.loadFailure?.cause?.message)
        see(kind(expected))
        val density = DENSITIES.random(random)
        val (fontScale, multiplier) = random.nextFloat() + 0.5f to random.nextFloat() + 0.5f
        val (mode, shift) = MODES.random(random) to SHIFTS.random(random)
        // Main handed the renderer the active theme's JSON and the palette Compose made from the
        // settings' accent, or the system's when unspecified, and the theme's Material You flags.
        val palette = palette(if (accent.isUnspecified) system else accent, expected.config.materialYouFlags)
        val main = RendererTheme(expected.stylesheetJson, palette.names, palette.colors, density, fontScale, multiplier, mode, shift)
        val made = themePalette(if (accent.isUnspecified) null else accent.toArgb(), system.toArgb(), active.config.materialYouFlags)
        val branch = rendererTheme(active, made, density, fontScale, multiplier, mode, shift)
        assertEquals(main, branch)
        if (accent.isUnspecified) see("system accent") else see("settings accent")
    }

    private fun palette(primary: Color, flags: MaterialYouFlags): ThemePalette =
        palettes.getOrPut(primary to flags) { referenceSnyggPalette(primary, flags) }

    private fun kind(info: ThemeManager.ThemeInfo) = when {
        info === previewInfo -> "showing preview"
        info.loadFailure?.cause is CancellationException -> "showing cancelled load"
        info.loadFailure?.cause is FileNotFoundException -> "showing unreadable stylesheet"
        info.loadFailure != null -> "showing unparsable stylesheet"
        info.loadedDir == null -> "showing base"
        else -> "showing loaded"
    }

    companion object {
        private const val CANCEL = "\u0000cancel"
        private val KEYBOARD = Any()
        private val DENSITIES = listOf(1f, 2.625f, 3f, 0.75f)
        private val MODES = listOf("characters", "symbols", "numeric", "phone")
        private val SHIFTS = listOf("unshifted", "shifted_manual", "caps_lock")
        private val json = Json { ignoreUnknownKeys = true }
        private val root = File("src/main/assets/ime/theme")
        val BUNDLED: List<ThemeExtension> = root.listFiles().orEmpty().sortedBy { it.name }.map {
            json.decodeFromString(ThemeExtension.serializer(), File(it, "extension.json").readText())
        }
        val SHEETS: Map<String, String> = BUNDLED.flatMap { e -> e.themes.map { e to it } }.associate { (e, c) ->
            c.id to File(File(root, e.meta.id), c.stylesheetPath()).readText()
        }
        private const val SCHEMA = "\"\$schema\":\"https://schemas.florisboard.org/snygg/v2/stylesheet\""
        val SYNTHETIC = listOf(
            "{$SCHEMA}", "{$SCHEMA,\"window\":{\"background\":\"#123456\"}}", "{", "", "[]", "{\"window\":{}}",
            "{\"\$schema\":\"wrong\"}", "{$SCHEMA,\"bad_name\":{}}", "{$SCHEMA,\"key\":{\"foreground\":\"nonsense\"}}",
            "{$SCHEMA,\"key\":{\"unknown\":\"1dp\"}}", "{$SCHEMA,\"key\":{},\"key\":{\"foreground\":\"transparent\"}}",
            "{$SCHEMA,\"key\":{\"font-size\":12}}", "{$SCHEMA,\"key[code=\\uD800]\":{}}",
            "{$SCHEMA,\"inline-autofill-chip\":{\"background\":\"dynamic-light-color(primary)\"}}",
        )
    }
}
