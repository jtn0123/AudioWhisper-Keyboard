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

package dev.patrickgold.florisboard.ime.clipboard

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Link
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material.icons.outlined.Email
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.lib.util.NetworkUtils
import org.florisboard.lib.compose.rippleClickable
import org.florisboard.lib.compose.stringRes
import org.florisboard.lib.snygg.SnyggQueryAttributes
import org.florisboard.lib.snygg.ui.SnyggIcon
import org.florisboard.lib.snygg.ui.SnyggRow
import org.florisboard.lib.snygg.ui.SnyggText

internal val ItemWidth = 200.dp
internal val DialogWidth = 240.dp

@Composable
internal fun ClipCategoryTitle(
    text: String,
    modifier: Modifier = Modifier,
) {
    SnyggText(elementName = FlorisImeUi.ClipboardSubheader.elementName,
        modifier = modifier.fillMaxWidth(),
        text = text.uppercase(),
    )
}

@Composable
internal fun ClipTextItemDescription(
    elementName: String,
    attributes: SnyggQueryAttributes,
    text: String,
    modifier: Modifier = Modifier,
): Unit = with(LocalDensity.current) {
    val icon: ImageVector?
    val description: String?
    when {
        NetworkUtils.isEmailAddress(text) -> {
            icon = Icons.Outlined.Email
            description = stringRes(R.string.clipboard__item_description_email)
        }
        NetworkUtils.isUrl(text) -> {
            icon = Icons.Default.Link
            description = stringRes(R.string.clipboard__item_description_url)
        }
        NetworkUtils.isPhoneNumber(text) -> {
            icon = Icons.Default.Phone
            description = stringRes(R.string.clipboard__item_description_phone)
        }
        else -> {
            icon = null
            description = null
        }
    }
    if (icon != null && description != null) {
        SnyggRow(
            elementName = elementName,
            attributes = attributes,
            modifier = modifier,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            SnyggIcon(
                imageVector = icon,
            )
            SnyggText(
                modifier = Modifier.weight(1f),
                text = description,
            )
        }
    }
}

@Composable
internal fun PopupAction(
    icon: ImageVector,
    text: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    SnyggRow(elementName = FlorisImeUi.ClipboardItemAction.elementName,
        modifier = modifier.rippleClickable(onClick = onClick),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SnyggIcon(elementName = FlorisImeUi.ClipboardItemActionIcon.elementName,
            imageVector = icon,
        )
        SnyggText(elementName = FlorisImeUi.ClipboardItemActionText.elementName,
            modifier = Modifier.weight(1f),
            text = text,
        )
    }
}
