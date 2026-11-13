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

import androidx.compose.ui.layout.ContentScale

data class SnyggUriValue(val uri: String) : SnyggValue {
    companion object : SnyggNativeValueEncoder<SnyggUriValue>(
        nativeId = 25, valueClass = SnyggUriValue::class,
        construct = { SnyggUriValue(it[0]) },
        destruct = { listOf(it.uri) },
    )

    override fun encoder() = Companion
}

data class SnyggContentScaleValue(val contentScale: ContentScale) : SnyggValue {
    companion object : SnyggEnumLikeValueEncoder<ContentScale>(
        serializationId = "textAlign",
        nativeId = 26,
        serializationMapping = NativeSnyggValues.keywords(26).zip(listOf(
            ContentScale.Crop,
            ContentScale.FillBounds,
            ContentScale.FillHeight,
            ContentScale.FillWidth,
            ContentScale.Fit,
            ContentScale.Inside,
            ContentScale.None,
        )).toMap(),
        default = ContentScale.FillBounds,
        construct = { SnyggContentScaleValue(it) },
        destruct = { (it as SnyggContentScaleValue).contentScale },
    )
    override fun encoder() = Companion
}

/**
 * The assert resolver for [SnyggUriValue].
 *
 * Implement this interface if you want to use [SnyggUriValue] (image and custom font-family support)
 * and pass the instance to [org.florisboard.lib.snygg.ui.ProvideSnyggTheme]
 *
 * @since 0.5.0-alpha01
 * @see [org.florisboard.lib.snygg.ui.ProvideSnyggTheme]
 */
interface SnyggAssetResolver {
    /**
     * Resolve a given flex URI to an absolute path.
     * If the asset cannot be found i.e. the uri is invalid, return [Result.Failure].
     *
     * @param uri The URI of the asset.
     * @return Result with the absolute path of the asset or a [Result.Failure].
     * @since 0.5.0-alpha01
     */
    fun resolveAbsolutePath(uri: String): Result<String>
}

internal object SnyggDefaultAssetResolver : SnyggAssetResolver {
    override fun resolveAbsolutePath(uri: String): Result<String> {
        return Result.failure(NotImplementedError("Default asset resolver does not implement path resolve ability"))
    }
}
