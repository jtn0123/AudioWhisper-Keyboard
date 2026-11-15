# Opt-in device suite only; retain the actual adapters exercised by the test APK.
-keep class com.jtn0123.audiowhisper.settingsui.** { *; }
-keep class com.jtn0123.audiowhisper.rust.NativeRuntime { *; }
-keep class dev.patrickgold.florisboard.app.FlorisAppActivity { *; }
-keep class dev.patrickgold.florisboard.app.FlorisPreferenceModel** { *; }
-keep class dev.patrickgold.florisboard.app.AppPrefsKt { *; }
# The harness waits for the in-process IME before editing a field.
-keep class dev.patrickgold.florisboard.FlorisImeServiceKt { *; }
# Test APK calls JVM facades which main R8 would otherwise inline or remove.
-keep class kotlin.** { *; }
-keep class kotlinx.coroutines.** { *; }
-keep class dev.patrickgold.jetpref.datastore.** { *; }
-keep class androidx.activity.** { *; }
