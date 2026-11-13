# Only the optional device replay build uses these rules. The test APK calls Kotlin
# APIs which R8 otherwise merges or inlines across the target APK boundary.
-keep class kotlin.** { *; }
-keep class com.jtn0123.audiowhisper.suggest.** { *; }
-keep class com.jtn0123.audiowhisper.rust.** { *; }

# The extension replay compares Android ICU with the same test-only Kotlin parser.
-keep class dev.patrickgold.florisboard.lib.ext.** { *; }
