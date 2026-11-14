package dev.patrickgold.florisboard.lib.reference

/** Extracted Android-free branches from FlorisRef and UserDictionaryValidation.
 * Uri.Builder/parse are platform effects; these fixtures compare the plans before that boundary. */
object ReferenceUtilityRules {
    fun normalize(str: String): List<String> = when {
        str.startsWith("assets:") -> listOf("assets", str.substring(7).let { if (it.isNotEmpty() && !it.startsWith('/')) "/$it" else it })
        str.startsWith("internal:") -> listOf("internal", str.substring(9).let { if (it.isNotEmpty() && !it.startsWith('/')) "/$it" else it })
        else -> listOf("", str)
    }
    fun url(url: String): String = when {
        url.startsWith("http://") || url.startsWith("https://") || url.startsWith("mailto:") -> url
        else -> "https://$url"
    }
    fun facts(scheme: String, authority: String, path: String) = listOf(
        if (scheme == "florisboard") when (authority) { "app-ui" -> "1"; "assets" -> "2"; "cache" -> "3"; "internal" -> "4"; else -> "0" } else "0",
        if (scheme != "florisboard") "1" else "0", if (scheme.isNotBlank() && authority.isNotBlank()) "1" else "0", path.removePrefix("/"))
    fun validate(rule: String, input: String, min: Int, max: Int): String {
        val str = input.trim()
        val regex = """^[^\s;,]+$""".toRegex()
        return when (rule) {
            "freq" -> {
                val freq = str.toIntOrNull(10)
                when { input.isBlank() || freq == null -> "empty"; freq < min || freq > max -> "invalid"; else -> "" }
            }
            "word" -> when { input.isBlank() -> "empty"; !str.matches(regex) -> "invalid"; else -> "" }
            else -> when { input.isBlank() -> ""; !str.matches(regex) -> "invalid"; else -> "" }
        }
    }
}
