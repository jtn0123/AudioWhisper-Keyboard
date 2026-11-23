# Only scripts/palette-art.sh's app (-PpaletteFixture=true) keeps these: PaletteArtDiff calls
# materialkolor as main's keyboard did on ART, and Rust's palette through themePalette, by name.
-keep class com.materialkolor.** { *; }
-keep class org.florisboard.lib.color.** { *; }
-keep class org.florisboard.lib.snygg.ui.SnyggUiKt { *; }
-keep class androidx.compose.material3.ColorScheme { *; }
-keep class androidx.compose.ui.graphics.Color { *; }
-keep class androidx.compose.ui.graphics.ColorKt { *; }
-keep class com.jtn0123.audiowhisper.render.RendererThemeKt { *; }
-keep class com.jtn0123.audiowhisper.render.ThemePalette { *; }
-keep class kotlin.** { *; }
