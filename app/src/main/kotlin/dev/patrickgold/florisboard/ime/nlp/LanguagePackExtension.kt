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

package dev.patrickgold.florisboard.ime.nlp

import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.ext.ExtensionComponent
import dev.patrickgold.florisboard.lib.ext.ExtensionEditor
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient

@Serializable
class LanguagePackComponent(
    override val id: String,
    override val label: String,
    override val authors: List<String>,
    val locale: FlorisLocale = FlorisLocale.fromTag(id),
) : ExtensionComponent {
    @Transient var parent: LanguagePackExtension? = null

}

@SerialName(LanguagePackExtension.SERIAL_TYPE)
@Serializable
class LanguagePackExtension(
    override val meta: ExtensionMeta,
    override val dependencies: List<String>? = null,
    val items: List<LanguagePackComponent> = listOf(),
) : Extension() {

    override fun components(): List<ExtensionComponent> = items

    override fun edit(): ExtensionEditor {
        TODO("LOL LMAO")
    }

    companion object {
        const val SERIAL_TYPE = "ime.extension.languagepack"
    }

    override fun serialType() = SERIAL_TYPE

}
