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
