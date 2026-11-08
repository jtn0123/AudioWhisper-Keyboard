/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.lib.util

import java.util.Locale

object UnitUtils {
    private const val KiB = (1024).toFloat()
    private const val MiB = (1024 * 1024).toFloat()
    private const val GiB = (1024 * 1024 * 1024).toFloat()

    // AudioWhisper: formatted in Locale.ROOT, as these go to the debug log; and the bytes case
    // passes its number, which it didn't (a size under 1 KiB threw MissingFormatArgumentException).
    fun formatMemorySize(sizeBytes: Long): String {
        return when {
            sizeBytes >= GiB -> "%.2f GiB".format(Locale.ROOT, sizeBytes / GiB)
            sizeBytes >= MiB -> "%.2f MiB".format(Locale.ROOT, sizeBytes / MiB)
            sizeBytes >= KiB -> "%.2f KiB".format(Locale.ROOT, sizeBytes / KiB)
            else -> "$sizeBytes bytes"
        }
    }
}
