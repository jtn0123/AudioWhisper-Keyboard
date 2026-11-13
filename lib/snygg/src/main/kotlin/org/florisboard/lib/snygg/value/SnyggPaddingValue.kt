/*
 * Copyright (C) 2025 The FlorisBoard Contributors
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

package org.florisboard.lib.snygg.value

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.calculateEndPadding
import androidx.compose.foundation.layout.calculateStartPadding
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp

data class SnyggPaddingValue(val values: PaddingValues) : SnyggValue {
    companion object : SnyggNativeValueEncoder<SnyggPaddingValue>(
        nativeId = 14, valueClass = SnyggPaddingValue::class,
        construct = { SnyggPaddingValue(PaddingValues(start = it[0].float().dp, top = it[1].float().dp, end = it[2].float().dp, bottom = it[3].float().dp)) },
        destruct = { listOf(it.values.calculateStartPadding(LayoutDirection.Ltr).value.bits(), it.values.calculateTopPadding().value.bits(), it.values.calculateEndPadding(LayoutDirection.Ltr).value.bits(), it.values.calculateBottomPadding().value.bits()) },
    )

    override fun encoder() = Companion
}
