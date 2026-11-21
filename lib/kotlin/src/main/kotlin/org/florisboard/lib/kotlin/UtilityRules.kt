package org.florisboard.lib.kotlin

/** Plain copied values; Android Uri/Locale objects and JVM callbacks never cross JNI. */
object UtilityRules {
    fun call(operation: UtilityOp, vararg values: String): List<String> =
        UtilityNative.call(operation.code, arrayOf(*values)).asList()
}
