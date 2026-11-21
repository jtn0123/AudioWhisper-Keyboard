# The JNI entry points are found by class and method name.
-keepclasseswithmembernames class com.jtn0123.audiowhisper.rust.** {
    native <methods>;
}

# Host/opt-in reference callbacks use these names. Their native exports are excluded from
# normal Android builds by awnative's legacy-fixture feature; preserve fixture R8 compatibility.
-keep interface com.jtn0123.audiowhisper.rust.DictationNames {
    boolean isName(java.lang.String);
}
-keepclassmembers class * implements com.jtn0123.audiowhisper.rust.DictationNames {
    boolean isName(java.lang.String);
}

# Frozen callback dictionary adapter (suggest/legacy_suggester.rs), for reference fixtures only.
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

# The spacing rules ask the editor for text only when they need it (awnative/src/spacing.rs).
-keep interface com.jtn0123.audiowhisper.rust.SpacingRules$Editor { *; }
-keepclassmembers class * implements com.jtn0123.audiowhisper.rust.SpacingRules$Editor { *; }
-keepclassmembers class com.jtn0123.audiowhisper.rust.SpacingRules$Rule {
    java.lang.String get*();
}

# Utility rules live in the JVM-only Kotlin module, with JNI in awnative.
-keepclasseswithmembernames class org.florisboard.lib.kotlin.UtilityNative {
    native <methods>;
}
