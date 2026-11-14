package dev.patrickgold.florisboard.app

import dev.patrickgold.florisboard.ime.core.Subtype

/** Saved bundled input methods removed from the Latin-only keyboard. Add-ons keep their IDs. */
internal fun Subtype.referenceSupportedInput(): Subtype {
    val deleted = (layoutMap.characters.extensionId == "org.florisboard.layouts" &&
        layoutMap.characters.componentId in removedCharacterLayouts) ||
        (composer.extensionId == "org.florisboard.composers" &&
            composer.componentId in setOf("hangul-unicode", "kana-unicode")) ||
        (popupMapping.extensionId == "org.florisboard.localization" &&
            popupMapping.componentId in removedPopupMappings) ||
        nlpProviders.spelling == REMOVED_HAN_PROVIDER || nlpProviders.suggestion == REMOVED_HAN_PROVIDER
    return if (deleted) Subtype.DEFAULT.copy(id = id) else this
}

private const val REMOVED_HAN_PROVIDER = "org.florisboard.nlp.providers.han.shape"

private val removedCharacterLayouts = setOf(
    "arabic",
    "armenian_alt_phonetic",
    "bengali_bd",
    "bulgarian_bds",
    "bulgarian_phonetic",
    "diktor",
    "eastern_armenian",
    "georgian_standard",
    "greek",
    "hebrew",
    "hindi_in",
    "jcuken_interslavic",
    "jcuken_russian",
    "jcuken_ukrainian",
    "jis",
    "korean",
    "korean_phonetic",
    "kurdish",
    "kurdish_standard",
    "persian",
    "persian2",
    "persian3",
    "rusyn",
    "rusyn_us",
    "serbian_cyrillic",
    "tamil",
    "thai_kedmanee",
    "thai_manoonchai",
    "udmurt_compact",
    "udmurt_extended",
    "urdu_phonetic",
    "warang_citi",
    "western_armenian"
)

private val removedPopupMappings = setOf(
    "ar",
    "bg",
    "bn-BD",
    "cjk",
    "ckb",
    "el",
    "fa",
    "fa2",
    "fa3",
    "hi-IN",
    "hy",
    "hy-am",
    "iw",
    "ja-JP-jis",
    "ka-std",
    "ko",
    "ko-KR",
    "ru",
    "rue",
    "udm",
    "uk",
    "uk-cyr-ext",
    "ur-PK"
)
