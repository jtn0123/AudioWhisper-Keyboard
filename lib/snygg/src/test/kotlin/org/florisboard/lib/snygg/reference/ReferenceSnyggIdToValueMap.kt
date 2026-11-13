/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.florisboard.lib.snygg.reference

import org.florisboard.lib.kotlin.toStringWithoutDotZero

typealias ReferenceSnyggIdToValueMap = MutableMap<String, String>

fun referenceSnyggIdToValueMapOf(vararg pairs: Pair<String, Any>): ReferenceSnyggIdToValueMap {
    val map = mutableMapOf<String, String>()
    map.add(*pairs)
    return map
}

fun ReferenceSnyggIdToValueMap.getInt(id: String): Int {
    return getValue(id).toInt()
}

fun ReferenceSnyggIdToValueMap.getFloat(id: String): Float {
    return getValue(id).toFloat()
}

fun ReferenceSnyggIdToValueMap.getString(id: String): String {
    return getValue(id)
}

fun ReferenceSnyggIdToValueMap.add(vararg pairs: Pair<String, Any>) {
    pairs.forEach { (id, value) ->
        if (value is Number) {
            put(id, value.toStringWithoutDotZero())
        } else {
            put(id, value.toString())
        }
    }
}
