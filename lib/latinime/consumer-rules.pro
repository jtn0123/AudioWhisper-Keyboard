# libawnative.so's LatinIME entry points are found by their JNI names, which
# spell out these exact class and method names (Java_com_android_inputmethod_...).
# A native whose class or method R8 renames can no longer be found.
-keep class com.android.inputmethod.** { *; }
