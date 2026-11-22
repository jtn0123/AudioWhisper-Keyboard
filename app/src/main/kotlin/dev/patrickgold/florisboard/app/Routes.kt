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

package dev.patrickgold.florisboard.app

import androidx.compose.animation.AnimatedContentScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideIn
import androidx.compose.animation.slideOut
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.unit.IntOffset
import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.navDeepLink
import androidx.navigation.toRoute
import dev.patrickgold.florisboard.app.ext.CheckUpdatesScreen
import dev.patrickgold.florisboard.app.ext.ExtensionEditScreen
import dev.patrickgold.florisboard.app.ext.ExtensionExportScreen
import dev.patrickgold.florisboard.app.ext.ExtensionHomeScreen
import dev.patrickgold.florisboard.app.ext.ExtensionImportScreen
import dev.patrickgold.florisboard.app.ext.ExtensionImportScreenType
import dev.patrickgold.florisboard.app.ext.ExtensionListScreen
import dev.patrickgold.florisboard.app.ext.ExtensionListScreenType
import dev.patrickgold.florisboard.app.ext.ExtensionViewScreen
import dev.patrickgold.florisboard.app.settings.dictionary.UserDictionaryType
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlin.reflect.KClass

@Target(AnnotationTarget.CLASS)
@Retention(AnnotationRetention.RUNTIME)
annotation class Deeplink(val path: String)

inline fun <reified T : Any> NavGraphBuilder.composableWithDeepLink(
    kClass: KClass<T>,
    noinline content: @Composable (AnimatedContentScope.(NavBackStackEntry) -> Unit),
) {
    val deeplink = requireNotNull(kClass.annotations.firstOrNull { it is Deeplink } as? Deeplink) {
        "faulty class: $kClass with annotations ${kClass.annotations}"
    }
    composable<T>(
        deepLinks = listOf(navDeepLink<T>(basePath = "ui://florisboard/${deeplink.path}")),
        content = content,
    )
}

object Routes {
    object Setup {
        @Serializable
        @Deeplink("setup")
        object Screen
    }

    object Settings {
        @Serializable
        @Deeplink("settings/home")
        object Home

        @Serializable
        @Deeplink("settings/localization")
        object Localization

        @Serializable
        @Deeplink("settings/localization/select-locale")
        object SelectLocale

        @Serializable
        @Deeplink("settings/localization/language-pack-manage")
        object LanguagePackManager

        @Serializable
        @Deeplink("settings/localization/subtype/add")
        object SubtypeAdd

        @Serializable
        @Deeplink("settings/localization/subtype/edit")
        object SubtypeEdit

        @Serializable
        @Deeplink("settings/theme")
        object Theme

        @Serializable
        @Deeplink("settings/theme/manage")
        data class ThemeManager(val action: String)

        @Serializable
        @Deeplink("settings/keyboard")
        object Keyboard

        @Serializable
        @Deeplink("settings/keyboard/input-feedback")
        object InputFeedback

        @Serializable
        @Deeplink("settings/smartbar")
        object Smartbar

        // AudioWhisper: filler words and vocabulary for dictation.
        @Serializable
        @Deeplink("settings/dictation")
        object Dictation

        @Serializable
        @Deeplink("settings/typing")
        object Typing

        @Serializable
        @Deeplink("settings/dictionary")
        object Dictionary

        @Serializable
        @Deeplink("settings/dictionary/user-dictionary")
        data class UserDictionary(val type: UserDictionaryType)

        @Serializable
        @Deeplink("settings/gestures")
        object Gestures

        @Serializable
        @Deeplink("settings/clipboard")
        object Clipboard

        @Serializable
        @Deeplink("settings/media")
        object Media

        @Serializable
        @Deeplink("settings/other")
        object Other

        @Serializable
        @Deeplink("settings/other/physical-keyboard")
        object PhysicalKeyboard

        @Serializable
        @Deeplink("settings/other/backup")
        object Backup

        @Serializable
        @Deeplink("settings/other/restore")
        object Restore

        @Serializable
        @Deeplink("settings/about")
        object About

        @Serializable
        @Deeplink("settings/about/project-license")
        object ProjectLicense

        @Serializable
        @Deeplink("settings/about/third-party-licenses")
        object ThirdPartyLicenses
    }

    object Ext {
        @Serializable
        @Deeplink("ext")
        object Home

        @Serializable
        @Deeplink("ext/list")
        data class List(val type: ExtensionListScreenType, val showUpdate: Boolean? = null)

        @Serializable
        @Deeplink("ext/edit")
        data class Edit(val id: String, @SerialName("create") val serialType: String? = null)

        @Serializable
        @Deeplink("ext/export")
        data class Export(val id: String)

        @Serializable
        @Deeplink("ext/import")
        data class Import(val type: ExtensionImportScreenType, val uuid: String? = null)

        @Serializable
        @Deeplink("ext/view")
        data class View(val id: String)

        @Serializable
        @Deeplink("ext/check-updates")
        object CheckUpdates
    }

    @Composable
    fun AppNavHost(
        modifier: Modifier,
        navController: NavHostController,
        startDestination: KClass<*>,
    ) {
        NavHost(
            modifier = modifier,
            navController = navController,
            startDestination = startDestination,
            enterTransition = {
                slideIn { IntOffset(it.width, 0) } + fadeIn()
            },
            exitTransition = {
                slideOut { IntOffset(-it.width, 0) } + fadeOut()
            },
            popEnterTransition = { EnterTransition.None },
            popExitTransition = {
                scaleOut(
                    targetScale = 0.85F,
                    transformOrigin = TransformOrigin(pivotFractionX = 0.8f, pivotFractionY = 0.5f)
                ) + fadeOut(spring(stiffness = Spring.StiffnessMedium))
            },
        ) {
            // An empty entry is a screen only Rust draws. It stays so that reaching it still
            // works: the fallback activity hands any of these routes back to Rust
            // (docs/compose-removal.md, step 2), Setup included while it is the start destination.
            composableWithDeepLink(Setup.Screen::class) {}
            composableWithDeepLink(Settings.Home::class) {}

            composableWithDeepLink(Settings.Localization::class) {}
            composableWithDeepLink(Settings.SelectLocale::class) {}
            composableWithDeepLink(Settings.LanguagePackManager::class) {}
            composableWithDeepLink(Settings.SubtypeAdd::class) {}
            composableWithDeepLink(Settings.SubtypeEdit::class) {}

            composableWithDeepLink(Settings.Theme::class) {}
            composableWithDeepLink(Settings.ThemeManager::class) {}

            composableWithDeepLink(Settings.Keyboard::class) {}
            composableWithDeepLink(Settings.InputFeedback::class) {}

            composableWithDeepLink(Settings.Smartbar::class) {}
            composableWithDeepLink(Settings.Dictation::class) {}

            composableWithDeepLink(Settings.Typing::class) {}

            composableWithDeepLink(Settings.Dictionary::class) {}
            composableWithDeepLink(Settings.UserDictionary::class) {}

            composableWithDeepLink(Settings.Gestures::class) {}

            composableWithDeepLink(Settings.Clipboard::class) {}

            composableWithDeepLink(Settings.Media::class) {}

            composableWithDeepLink(Settings.Other::class) {}
            composableWithDeepLink(Settings.PhysicalKeyboard::class) {}
            composableWithDeepLink(Settings.Backup::class) {}
            composableWithDeepLink(Settings.Restore::class) {}

            composableWithDeepLink(Settings.About::class) {}
            composableWithDeepLink(Settings.ProjectLicense::class) {}
            composableWithDeepLink(Settings.ThirdPartyLicenses::class) {}

            composableWithDeepLink(Ext.Home::class) { ExtensionHomeScreen() }
            composableWithDeepLink(Ext.List::class) { navBackStack ->
                val payload = navBackStack.toRoute<Ext.List>()
                val showUpdate = payload.showUpdate != null && payload.showUpdate
                ExtensionListScreen(payload.type, showUpdate)
            }
            composableWithDeepLink(Ext.Edit::class) { navBackStack ->
                val payload = navBackStack.toRoute<Ext.Edit>()
                val extensionId = payload.id
                val serialType = payload.serialType
                ExtensionEditScreen(
                    id = extensionId,
                    createSerialType = serialType.takeIf { !it.isNullOrBlank() },
                )
            }
            composableWithDeepLink(Ext.Export::class) { navBackStack ->
                val payload = navBackStack.toRoute<Ext.Export>()
                val extensionId = payload.id
                ExtensionExportScreen(id = extensionId)
            }
            composableWithDeepLink(Ext.Import::class) { navBackStack ->
                val payload = navBackStack.toRoute<Ext.Import>()
                val uuid = payload.uuid
                ExtensionImportScreen(payload.type, uuid)
            }
            composableWithDeepLink(Ext.View::class) { navBackStack ->
                val payload = navBackStack.toRoute<Ext.View>()
                val extensionId = payload.id
                ExtensionViewScreen(id = extensionId)
            }
            composableWithDeepLink(Ext.CheckUpdates::class) {
                CheckUpdatesScreen()
            }
        }
    }
}
