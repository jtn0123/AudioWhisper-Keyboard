# libjni_latinime.so registers its methods on these exact class, method and
# field names (JNI_OnLoad -> RegisterNatives). Registration fails, and the
# library refuses to load, if R8 renames or strips any of them, including the
# natives the app never calls.
-keep class com.android.inputmethod.** { *; }
