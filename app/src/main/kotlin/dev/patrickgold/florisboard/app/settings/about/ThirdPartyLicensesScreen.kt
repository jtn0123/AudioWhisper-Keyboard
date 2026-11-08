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

package dev.patrickgold.florisboard.app.settings.about

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import com.mikepenz.aboutlibraries.ui.compose.LibraryDefaults
import com.mikepenz.aboutlibraries.ui.compose.android.produceLibraries
import com.mikepenz.aboutlibraries.ui.compose.m3.LibrariesContainer
import com.mikepenz.aboutlibraries.ui.compose.m3.libraryColors
import com.mikepenz.aboutlibraries.ui.compose.m3.style.m3VariantColors
import com.mikepenz.aboutlibraries.ui.compose.style.LicenseHueResolver
import com.mikepenz.aboutlibraries.ui.compose.variant.LibraryDetailMode
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.lib.compose.FlorisScreen
import org.florisboard.lib.compose.florisScrollbar
import org.florisboard.lib.compose.stringRes

@Composable
fun ThirdPartyLicensesScreen() = FlorisScreen {
    title = stringRes(R.string.about__third_party_licenses__title)
    scrollable = false
    iconSpaceReserved = false

    val lazyListState = rememberLazyListState()
    val libs by produceLibraries()

    content {
        LibrariesContainer(
            libraries = libs,
            modifier = Modifier
                .fillMaxSize()
                .florisScrollbar(lazyListState, isVertical = true),
            // Since AboutLibraries 15, colors only covers the list background and the license dialog.
            colors = LibraryDefaults.libraryColors(
                libraryBackgroundColor = MaterialTheme.colorScheme.background,
                libraryContentColor = MaterialTheme.colorScheme.onBackground,
                dialogConfirmButtonColor = MaterialTheme.colorScheme.primary,
            ),
            // The rows are styled here: plain rows on the background and every license badge in
            // primaryContainer, as before, rather than the new per-license hues.
            variantColors = LibraryDefaults.m3VariantColors(
                rowBackground = MaterialTheme.colorScheme.background,
                rowOnBackground = MaterialTheme.colorScheme.onBackground,
                licenseHueResolver = LicenseHueResolver.None,
                licenseBadgeContainer = MaterialTheme.colorScheme.primaryContainer,
                licenseBadgeContent = MaterialTheme.colorScheme.onPrimaryContainer,
            ),
            // A tap shows the license text bundled in the app, as before AboutLibraries 15, rather
            // than a row of links whose "View license" leaves for the browser.
            detailMode = LibraryDetailMode.Dialog,
            lazyListState = lazyListState,
        )
    }
}
