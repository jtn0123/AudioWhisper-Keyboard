package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.dictation.ModelState

/**
 * Frozen TypingScreen (with ForgetLearnedWordsDialog and SpellCheckerServiceSelector),
 * DictionaryScreen, PhysicalKeyboardScreen, AboutScreen and DictationScreen (with ModelStatus)
 * decisions, transcribed from their
 * Compose sources at d752cb96: the row order, each enabledIf/visibleIf lambda and `if` branch,
 * each list's entries, and what each onClick does. No production code.
 */
internal object ReferenceGroupTwoScreens {
    /** What the screens read: stored preferences, Settings.Secure, the package manager, the build. */
    class Facts(
        val p: Map<String, String>,
        /** `LocalConfiguration.current.keyboard != KEYBOARD_NOKEYS`. */
        val keyboardAttached: Boolean,
        /** Settings.Secure `spell_checker_enabled` and `selected_spell_checker`; null when unset. */
        val spellEnabled: String?,
        val spellId: String?,
        /** `getApplicationLabel` of the spell checker's package; null when the lookup throws. */
        val spellLabel: String?,
        val packageName: String = "com.jtn0123.audiowhisper.keyboard",
        val versionName: String = "0.5.0-alpha01",
        val versionCode: Int = 101,
        /** ModelStatus's `speech` and `voice`, and `checked` once the SHA-256 check finished. */
        val speech: ModelState = ModelState.Ready,
        val voice: ModelState = ModelState.Ready,
        val checked: Boolean = true,
    ) {
        fun on(key: String) = p[key] == "true"
        /** `runCatching { ComponentName.unflattenFromString(id!!)!!.packageName }.getOrDefault("null")`. */
        val spellPackage: String get() {
            val id = spellId ?: return "null"
            val sep = id.indexOf('/')
            return if (sep < 0 || sep + 1 >= id.length) "null" else id.substring(0, sep)
        }
        val florisSpellChecker get() = spellEnabled == "1" && spellPackage == packageName
        /** AboutScreen's `appVersion`. */
        val version get() = "$versionName ($versionCode)"
    }

    /**
     * A row in source order. `kind` is toggle, choice, heading, route, confirm, info, effect, card
     * (the auto-space Card), spell (the selector's card) or header (AboutScreen's icon and name).
     * `click` is what Compose does on a click (an effect, its plan given resolved strings); for a
     * confirm it is what confirming does; for a card it is the hyperlink's.
     */
    data class Row(
        val key: String,
        val kind: String,
        val choices: List<String> = emptyList(),
        val route: String = "",
        val click: (Facts, (String) -> String) -> List<String> = { _, _ -> emptyList() },
        val enabled: (Facts) -> Boolean = { true },
        val visible: (Facts) -> Boolean = { true },
    )
    data class Screen(val route: String, val preview: Boolean, val rows: List<Row>, val inputs: Map<String, List<String>>)

    private val bool = listOf("false", "true")
    private fun suggestions(f: Facts) = f.on("suggestion__enabled")
    private fun url(key: String) = { _: Facts, s: (String) -> String -> listOf("url", s(key)) }

    val typing = Screen("settings/typing", true, listOf(
        Row("pref__suggestion__title", "heading"),
        Row("suggestion__enabled", "toggle"),
        Row("suggestion__block_possibly_offensive", "toggle", enabled = ::suggestions),
        Row("suggestion__api30_inline_suggestions_enabled", "toggle"),
        Row("suggestion__incognito_mode", "choice", listOf("FORCE_OFF", "DYNAMIC_ON_OFF", "FORCE_ON")),
        Row("suggestion__learn_from_typing", "toggle", enabled = ::suggestions),
        Row("suggestion__learn_tap_aim", "toggle", enabled = ::suggestions),
        // ForgetLearnedWordsDialog's onConfirm: forget, then the short toast.
        Row("pref__suggestion__forget_learned__label", "confirm",
            click = { _, s -> listOf("forgetLearnedWords", s("pref__suggestion__forget_learned__done")) }),
        Row("pref__correction__title", "heading"),
        Row("correction__auto_capitalization", "toggle"),
        Row("correction__auto_space_punctuation", "toggle"),
        Row("pref__correction__auto_space_punctuation__feedback", "card", click = url("florisboard__auto_space_feedback_url"),
            visible = { it.on("correction__auto_space_punctuation") }),
        Row("correction__remember_caps_lock_state", "toggle"),
        Row("correction__double_space_period", "toggle"),
        Row("pref__spelling__title", "heading"),
        // openSystemSpellCheckerSettings, from whichever card shows.
        Row("spell", "spell", click = { _, _ -> listOf("intent", "spellCheckerSettings") }),
        Row("spelling__language_mode", "choice", listOf("USE_SYSTEM_LANGUAGES", "USE_KEYBOARD_SUBTYPES"),
            enabled = { it.florisSpellChecker }),
        Row("spelling__use_contacts", "toggle", enabled = { it.florisSpellChecker }, visible = { false }),
        Row("spelling__use_udm_entries", "toggle", enabled = { it.florisSpellChecker }, visible = { false }),
        Row("settings__dictionary__title", "heading"),
        Row("settings__dictionary__title#route", "route", route = "settings/dictionary"),
    ), mapOf("suggestion__enabled" to bool, "correction__auto_space_punctuation" to bool))

    val dictionary = Screen("settings/dictionary", true, listOf(
        Row("suggestion__enable_system_user_dictionary", "toggle"),
        Row("manage_system", "route", route = "settings/dictionary/user-dictionary/SYSTEM",
            enabled = { it.on("suggestion__enable_system_user_dictionary") }),
        Row("suggestion__enable_floris_user_dictionary", "toggle"),
        Row("manage_floris", "route", route = "settings/dictionary/user-dictionary/FLORIS",
            enabled = { it.on("suggestion__enable_floris_user_dictionary") }),
    ), mapOf("suggestion__enable_system_user_dictionary" to bool, "suggestion__enable_floris_user_dictionary" to bool))

    val physicalKeyboard = Screen("settings/other/physical-keyboard", false, listOf(
        Row("system_settings", "effect", click = { _, _ -> listOf("intent", "hardKeyboardSettings") }, visible = { it.keyboardAttached }),
        // The not-attached Preference has no onClick.
        Row("system_settings_not_attached", "info", visible = { !it.keyboardAttached }),
        Row("physical_keyboard__show_on_screen_keyboard", "toggle"),
    ), emptyMap())

    val about = Screen("settings/about", false, listOf(
        Row("header", "header"),
        Row("version", "effect", click = { f, s ->
            listOf("copy", f.version, s("about__version_copied__title"), s("about__version_copied__error"))
        }),
        Row("changelog", "effect", click = { f, s -> listOf("url", s("florisboard__changelog_url").replace("{version}", f.versionName)) }),
        Row("repository", "effect", click = url("florisboard__repo_url")),
        Row("privacy_policy", "effect", click = url("florisboard__privacy_policy_url")),
        Row("project_license", "route", route = "settings/about/project-license"),
        Row("third_party_licenses", "route", route = "settings/about/third-party-licenses"),
    ), emptyMap())

    val dictation = Screen("settings/dictation", true, listOf(
        Row("dictation__models__title", "heading"),
        Row("speech", "status"),
        Row("voice", "status"),
        Row("dictation__remove_fillers", "toggle"),
        Row("pref__dictation__vocabulary__label", "heading"),
        Row("note", "note"),
        // The OutlinedTextField: onValueChange saves each change to the preference.
        Row("dictation__vocabulary", "field"),
    ), emptyMap())

    val screens = listOf(typing, dictionary, physicalKeyboard, about, dictation)

    /** ModelStatus's `describe`. Android's getString without arguments leaves `%1$s` as written. */
    fun describe(state: ModelState, checked: Boolean, speechModel: Boolean, s: (String) -> String): String = when (state) {
        ModelState.Ready -> when {
            !checked -> s("dictation__models__checking")
            speechModel -> s("dictation__models__speech_ready")
            else -> s("dictation__models__voice_ready")
        }
        is ModelState.Missing -> if (speechModel) s("dictation__models__speech_missing") else s("dictation__models__voice_missing")
        is ModelState.Damaged -> {
            val files = state.files.joinToString(", ")
            (if (speechModel) s("dictation__models__speech_damaged") else s("dictation__models__voice_damaged")).replace("%1${'$'}s", files)
        }
    }
    val modelStates = listOf(ModelState.Ready, ModelState.Missing(listOf("encoder.onnx")),
        ModelState.Damaged(listOf("encoder.onnx", "tokens.txt")))

    /** SpellCheckerServiceSelector's branch: the card, its text and secondary text. */
    fun spellCard(f: Facts, s: (String) -> String): Triple<String, String, String> = when {
        f.spellEnabled != "1" -> Triple("error", s("pref__spelling__active_spellchecker__summary_disabled"), "")
        f.spellId == null -> Triple("warning", s("pref__spelling__active_spellchecker__summary_none"), "")
        else -> Triple("app", f.spellLabel ?: s("pref__spelling__active_spellchecker__unknown"), f.spellPackage)
    }
    /** The (enabled, id, label) states the selector distinguishes, as Settings.Secure can hold them. */
    val spellStates: List<Triple<String?, String?, String?>> = listOf(null, "0", "1").flatMap { enabled ->
        listOf(null, "com.jtn0123.audiowhisper.keyboard/dev.patrickgold.florisboard.FlorisSpellCheckerService",
            "com.google.android.inputmethod.latin/com.android.inputmethod.latin.spellcheck.AndroidSpellCheckerService",
            "no-slash", "trailing/").flatMap { id -> listOf(null, "Spell App").map { label -> Triple(enabled, id, label) } }
    }
    /** AboutScreen's summary for the version row, and its one templated summary. */
    fun summary(key: String, f: Facts, s: (String) -> String): String? = when (key) {
        "version" -> f.version
        "project_license" -> s("about__project_license__summary").replace("{license_name}", "Apache 2.0")
        "speech" -> describe(f.speech, f.checked, speechModel = true, s)
        "voice" -> describe(f.voice, f.checked, speechModel = false, s)
        else -> null
    }
}
