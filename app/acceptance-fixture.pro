# Test-APK-only platform acceptance calls; normal builds keep their existing shrinking.
-keep class org.florisboard.lib.kotlin.MimeTypeFilter { public *; }
-keep class org.florisboard.lib.kotlin.MimeTypeFilterKt { public *; }
-keep class dev.patrickgold.florisboard.lib.FlorisLocale { public *; }
-keep class dev.patrickgold.florisboard.lib.FlorisLocale$Companion { public *; }
-keep class dev.patrickgold.florisboard.lib.io.FlorisRef { public *; }
-keep class dev.patrickgold.florisboard.lib.io.FlorisRef$Companion { public *; }
-keep class dev.patrickgold.florisboard.app.AppPrefsKt { public *; }
# Kotlin test facades are provided by main; retain them for external probe calls.
-keep class kotlin.** { *; }

# Synthetic learned-store mutation uses this otherwise unused compatibility API.
-keep class com.jtn0123.audiowhisper.suggest.LearnedWords { *; }

# The GIF search check (RendererFixtureTest) sets the search source, which nothing in the app does,
# and compares with Rust's own typing rule, which the app reaches only through the search session.
-keep class com.jtn0123.audiowhisper.gif.GifSearch { *; }
-keep class com.jtn0123.audiowhisper.rust.GifRules { *; }
