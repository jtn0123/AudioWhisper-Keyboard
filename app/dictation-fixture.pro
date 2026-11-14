# Only opt-in comparison APKs retain managed owners accessed from the disposable test APK.
-keep class com.jtn0123.audiowhisper.dictation.** { *; }
-keep class com.jtn0123.audiowhisper.rust.SherpaAsr { *; }
-keep class com.jtn0123.audiowhisper.rust.SherpaVad** { *; }
-keep class com.jtn0123.audiowhisper.rust.MicKeyState** { *; }
-keep class com.jtn0123.audiowhisper.rust.DictationRules** { *; }
# The old JNI ASR/VAD oracle exists only in comparison builds.
-keep class com.k2fsa.sherpa.onnx.** { *; }
# The test APK treats main's Kotlin runtime as provided. Main R8 cannot see external fixture
# calls into its facade methods, so keep that runtime only for these opt-in comparison APKs.
-keep class kotlin.** { *; }
