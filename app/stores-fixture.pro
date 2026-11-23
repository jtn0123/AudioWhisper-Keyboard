# Keeps what StoresMigrationDeviceTest and StoresBackupDeviceTest drive in the shrunk debug build, so
# their instrumentation finds it (-PstoresFixture=true; docs/phase5-stores.md).
-keep class kotlin.** { *; }
-keep class kotlinx.** { *; }
-keep class androidx.core.net.** { *; }
-keep class com.jtn0123.audiowhisper.rust.** { *; }
-keep class com.jtn0123.audiowhisper.stores.** { *; }
-keep class dev.patrickgold.florisboard.ime.clipboard.provider.** { *; }
-keep class dev.patrickgold.florisboard.ime.dictionary.** { *; }
-keep class org.florisboard.lib.kotlin.io.** { *; }
