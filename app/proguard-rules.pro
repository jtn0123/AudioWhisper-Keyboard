# Disable obfuscation (we use Proguard exclusively for optimization)
-dontobfuscate

# Keep `Companion` object fields of serializable classes.
# This avoids serializer lookup through `getDeclaredClasses` as done for named companion objects.
-if @kotlinx.serialization.Serializable class **
-keepclassmembers class <1> {
    static <1>$Companion Companion;
}

# Keep `serializer()` on companion objects (both default and named) of serializable classes.
-if @kotlinx.serialization.Serializable class ** {
    static **$* *;
}
-keepclassmembers class <2>$<3> {
    kotlinx.serialization.KSerializer serializer(...);
}

# Keep `INSTANCE.serializer()` of serializable objects.
-if @kotlinx.serialization.Serializable class ** {
    public static ** INSTANCE;
}
-keepclassmembers class <1> {
    public static <1> INSTANCE;
    kotlinx.serialization.KSerializer serializer(...);
}

# @Serializable and @Polymorphic are used at runtime for polymorphic serialization.
-keepattributes RuntimeVisibleAnnotations,AnnotationDefault

# Only Pocket TTS still uses sherpa JNI. Its native config/result field names must survive.
# ASR/VAD Kotlin classes are retained only in opt-in comparison APKs.
-keep class com.k2fsa.sherpa.onnx.OfflineTts** { *; }
-keep class com.k2fsa.sherpa.onnx.GenerationConfig { *; }
-keep class com.k2fsa.sherpa.onnx.GeneratedAudio { *; }
-keep class com.k2fsa.sherpa.onnx.WaveReader** { *; }
-keep class com.k2fsa.sherpa.onnx.WaveData { *; }

# The read-aloud voice's callback, which that native code finds by its method's signature.
-keep class com.jtn0123.audiowhisper.reader.VoiceEngine$AudioCallback { *; }
