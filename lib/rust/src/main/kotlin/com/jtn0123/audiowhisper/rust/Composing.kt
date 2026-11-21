package com.jtn0123.audiowhisper.rust

/**
 * FlorisBoard's text composers' table of [rules], each an ending and what it becomes, tried in the
 * order given (see [ComposingRules]).
 */
fun ComposingRules(rules: List<Pair<String, String>>): ComposingRules =
    ComposingRules(rules.map { it.first }, rules.map { it.second })
