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

package dev.patrickgold.florisboard.lib.util

import android.content.Context
import android.content.Intent
import android.provider.Settings
import android.view.inputmethod.InputMethodManager
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import dev.patrickgold.florisboard.BuildConfig
import kotlinx.coroutines.delay
import org.florisboard.lib.android.systemServiceOrNull

private const val TIMED_QUERY_DELAY = 500L

object InputMethodUtils {
    fun isFlorisboardEnabled(context: Context): Boolean {
        return context.systemServiceOrNull(InputMethodManager::class)
            ?.enabledInputMethodList
            ?.any { it.packageName == BuildConfig.APPLICATION_ID } ?: false
    }

    fun isFlorisboardSelected(context: Context): Boolean {
        return context.systemServiceOrNull(InputMethodManager::class)
            ?.currentInputMethodInfo
            ?.packageName == BuildConfig.APPLICATION_ID
    }

    fun showImeEnablerActivity(context: Context) {
        val intent = Intent()
        intent.action = Settings.ACTION_INPUT_METHOD_SETTINGS
        intent.addCategory(Intent.CATEGORY_DEFAULT)
        context.startActivity(intent)
    }

    fun showImePicker(context: Context): Boolean {
        val imm = context.systemServiceOrNull(InputMethodManager::class)
        return if (imm != null) {
            imm.showInputMethodPicker()
            true
        } else {
            false
        }
    }

    @Composable
    fun observeIsFlorisboardEnabled(): State<Boolean> {
        val state = remember { mutableStateOf(false) }
        val context = LocalContext.current
        LaunchedEffect(Unit) {
            while (true) {
                state.value = isFlorisboardEnabled(context)
                delay(TIMED_QUERY_DELAY)
            }
        }
        return state
    }

    @Composable
    fun observeIsFlorisboardSelected(): State<Boolean> {
        val state = remember { mutableStateOf(false) }
        val context = LocalContext.current
        LaunchedEffect(Unit) {
            while (true) {
                state.value = isFlorisboardSelected(context)
                delay(TIMED_QUERY_DELAY)
            }
        }
        return state
    }
}
