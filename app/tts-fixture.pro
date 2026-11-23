# Test APK calls these managed/native owners; absent from normal app retention rules.
-keep class com.jtn0123.audiowhisper.reader.** { *; }
-keep class com.jtn0123.audiowhisper.rust.SherpaTts** { *; }
-keep class kotlin.** { *; }
