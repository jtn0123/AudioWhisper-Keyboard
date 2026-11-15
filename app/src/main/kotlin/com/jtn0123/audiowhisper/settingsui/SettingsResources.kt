package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import dev.patrickgold.florisboard.R
import org.florisboard.lib.kotlin.curlyFormat

/**
 * The native screens' strings as key/value pairs, formatted as Compose's stringRes and pluralsRes
 * do. Each plural request is `key|min|max` from the native screens; it arrives as one `key#n`
 * string per value, so the quantity rules stay Android's.
 */
internal fun settingsResources(context: Context, pluralRequests: List<String>): Array<String> {
    val appName = context.getString(R.string.floris_app_name)
    val strings = settingsStringIds.flatMap { (key, id) -> listOf(key, context.getString(id).curlyFormat("app_name" to appName)) }
    val plurals = pluralRequests.filter { it.isNotEmpty() }.flatMap { request ->
        val (key, min, max) = request.split("|")
        val id = requireNotNull(settingsPluralIds[key]) { "No plurals resource for $key" }
        (min.toInt()..max.toInt()).flatMap { n ->
            listOf("$key#$n", context.resources.getQuantityString(id, n).curlyFormat("app_name" to appName, "v" to n))
        }
    }
    return (strings + plurals).toTypedArray()
}
