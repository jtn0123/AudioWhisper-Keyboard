# The JNI entry points are found by class and method name.
-keepclasseswithmembernames class com.jtn0123.audiowhisper.rust.** {
    native <methods>;
}

# The Rust calls DictationNames.isName by name (crates/awnative/src/dictation.rs), on whatever
# implements it.
-keep interface com.jtn0123.audiowhisper.rust.DictationNames {
    boolean isName(java.lang.String);
}
-keepclassmembers class * implements com.jtn0123.audiowhisper.rust.DictationNames {
    boolean isName(java.lang.String);
}

# The Rust calls the dictionary back by name (crates/awnative/src/suggest/suggester.rs).
-keep interface com.jtn0123.audiowhisper.rust.WordProbabilities {
    int probability(java.lang.String);
}
-keepclassmembers class * implements com.jtn0123.audiowhisper.rust.WordProbabilities {
    int probability(java.lang.String);
}
-keep interface com.jtn0123.audiowhisper.rust.SuggestDictionary {
    int ngramProbability(java.lang.String);
    int[] decode(java.lang.String, boolean, boolean);
}
-keepclassmembers class * implements com.jtn0123.audiowhisper.rust.SuggestDictionary {
    int ngramProbability(java.lang.String);
    int[] decode(java.lang.String, boolean, boolean);
}

# Synchronous PCM consumer called by awnative, before the C buffer can be freed.
-keep interface com.jtn0123.audiowhisper.rust.TtsAudio { boolean audio(java.nio.ByteBuffer); }
-keepclassmembers class * implements com.jtn0123.audiowhisper.rust.TtsAudio {
    boolean audio(java.nio.ByteBuffer);
}

# Rust owns the screen walk; this adapter fetches Android node data synchronously.
-keep interface com.jtn0123.audiowhisper.rust.ReaderTree { *; }
-keepclassmembers class * implements com.jtn0123.audiowhisper.rust.ReaderTree { *; }

# Utility rules live in the JVM-only Kotlin module, with JNI in awnative.
-keepclasseswithmembernames class org.florisboard.lib.kotlin.UtilityNative {
    native <methods>;
}
