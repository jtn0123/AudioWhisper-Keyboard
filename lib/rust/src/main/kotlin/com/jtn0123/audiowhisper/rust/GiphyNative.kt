package com.jtn0123.audiowhisper.rust

/**
 * GIPHY's entry points in libawnative: lib/rust/crates/awnative/src/giphy.rs, which documents
 * each. [GiphyRules] wraps them.
 */
internal object GiphyNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun parse(body: String): Array<Any>

    @JvmStatic external fun url(apiKey: String, endpoint: String, names: Array<String>, values: Array<String>): String

    @JvmStatic external fun trendingUrl(apiKey: String): String

    @JvmStatic external fun searchUrl(apiKey: String, query: String): String
}
