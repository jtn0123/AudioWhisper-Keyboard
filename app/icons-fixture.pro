# Only scripts/icon-golden.sh's app (-PiconsFixture=true) keeps these: IconGoldenDump reads every
# icon the Rust crate holds through Compose, as the app drew them before Rust did.
-keep class androidx.compose.material.icons.** { *; }
-keep class org.florisboard.lib.compose.icons.** { *; }
-keep class androidx.compose.ui.graphics.** { *; }
-keep class androidx.compose.ui.res.** { *; }
-keep class kotlin.** { *; }
