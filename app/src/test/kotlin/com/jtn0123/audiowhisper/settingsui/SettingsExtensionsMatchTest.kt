package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.keyboard.KeyboardExtension
import dev.patrickgold.florisboard.ime.nlp.LanguagePackComponent
import dev.patrickgold.florisboard.ime.nlp.LanguagePackExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentImpl
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.ext.ExtensionMaintainer
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/** The extension routes Rust draws, in the order it lists them. */
internal val extensionRoutes = listOf("ext", "ext/list/EXT_THEME", "ext/list/EXT_KEYBOARD", "ext/list/EXT_LANGUAGEPACK",
    "ext/view", "ext/check-updates")

/**
 * The extension screens through the real native settings, their report built by SettingsExtensions' own
 * [extensionRows], against what the frozen Compose screens did with the same index ([ReferenceExtensionScreens]).
 * Every id, title and address is made up.
 */
class SettingsExtensionsMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) =
        Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun node(s: JsonObject, id: Int) = nodes(s).singleOrNull { it.text("id") == id.toString() }

    private val random = Random(1010)
    private val words = listOf("Day", "Night", "Ocean", "", " ", "Ölfarbe", "🌙", "a, b")
    private fun maybe(s: String) = s.takeIf { random.nextInt(3) > 0 }
    private fun meta(id: String) = ExtensionMeta(id = id, version = "${random.nextInt(9)}.${random.nextInt(20)}", title = "Title $id",
        description = maybe(words.random(random)), keywords = maybe("")?.let { List(random.nextInt(4)) { words.random(random) } },
        homepage = maybe(listOf("https://example.org/$id", " ", "").random(random)),
        issueTracker = maybe(listOf("https://bugs.example.net/x", "").random(random)),
        maintainers = List(random.nextInt(3)) { ExtensionMaintainer("M$it", maybe("m$it@example.org"), maybe("https://m$it.example")) },
        license = listOf("apache-2.0", "mit").random(random))
    private fun theme(id: String) = ThemeExtension(meta(id), themes = List(random.nextInt(3)) {
        ThemeExtensionComponentImpl("t$it", "Theme $it", List(random.nextInt(3)) { a -> "author$a" }, random.nextBoolean(),
            stylesheetPath = maybe("custom/t$it.json"))
    })
    private fun pack(id: String) = LanguagePackExtension(meta(id), items = List(random.nextInt(3)) {
        LanguagePackComponent(listOf("de_DE", "zh_CN", "en")[it], "Pack $it", listOf("florisboard"))
    })

    /** The index: keyboards, themes and language packs, each extension deletable or not. */
    private class Index(val keyboards: List<Extension>, val themes: List<Extension>, val packs: List<Extension>, val deletable: Set<String>) {
        val all get() = keyboards + themes + packs
    }
    private fun index() = Index(List(random.nextInt(3)) { KeyboardExtension(meta("k$it.ext")) }, List(random.nextInt(4)) { theme("t$it.ext") },
        List(random.nextInt(3)) { pack("p$it.ext") }, setOf("t0.ext", "t2.ext", "p1.ext", "k1.ext"))
    private fun report(index: Index) =
        index.all.flatMap { extensionRows(it, it.meta.id in index.deletable) { link -> "host of $link" } }.flatMap { it.asList() }.toTypedArray()

    /** A phone-wide window, and the delete question's message. */
    private fun configure(n: KeyboardRenderer) {
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        n.settings(SettingsUiOp.RESOURCES, arrayOf("action__delete_confirm_message", "Delete {name}?"))
    }
    /** Native settings at [route] with [index] reported. */
    private fun open(n: KeyboardRenderer, index: Index, route: String): JsonObject {
        configure(n)
        assertEquals(listOf(SettingsUiExtensionEffect.LOAD), effects(state(n, SettingsUiOp.NAVIGATE, route))[0])
        return state(n, SettingsUiOp.EXTENSIONS, *report(index))
    }

    @Test fun searchForUpdatesLinksWhatGenerateUpdateUrlLinkedForEachScreen() {
        val routes = listOf("ext", "ext/check-updates", "ext/list/EXT_THEME?showUpdate=true", "ext/list/EXT_KEYBOARD?showUpdate=true",
            "ext/list/EXT_LANGUAGEPACK?showUpdate=true")
        repeat(30) {
            val index = index()
            for (route in routes) KeyboardRenderer(arrayOf(font)).use { n ->
                open(n, index, route)
                val listed = ReferenceExtensionScreens.listed(route, index.keyboards, index.themes, index.packs)
                val link = effects(state(n, SettingsUiOp.ACTIVATE, "200002")).single()
                assertEquals(listOf("url", ReferenceExtensionScreens.updateUrl(listed)), link, route)
            }
        }
    }

    /** Fix: Compose dropped the comma after an extension `equals` to the list's last, so two alike keyboard
     * extensions (a data class) ran together. Rust separates every pair. */
    @Test fun twoAlikeKeyboardExtensionsStillGiveWellFormedJson() {
        val meta = ExtensionMeta(id = "same.ext", version = "1.0", title = "Same", maintainers = emptyList(), license = "mit")
        val index = Index(listOf(KeyboardExtension(meta), KeyboardExtension(meta)), emptyList(), emptyList(), emptySet())
        val compose = ReferenceExtensionScreens.updateUrl(index.keyboards)
        assertEquals("https://beta.addons.florisboard.org/check-updates#data={\"same.ext\":\"1.0\"\"same.ext\":\"1.0\"}", compose)
        KeyboardRenderer(arrayOf(font)).use { n ->
            open(n, index, "ext/list/EXT_KEYBOARD?showUpdate=true")
            val link = effects(state(n, SettingsUiOp.ACTIVATE, "200002")).single()[1]
            assertEquals(compose.replace("\"\"", "\",\""), link)
            Json.parseToJsonElement(link.substringAfter("#data="))
        }
    }

    @Test fun eachListShowsItsIndexWithTheBoxesItsRouteAsks() {
        val routes = extensionRoutes.filter { it.startsWith("ext/list/") }.flatMap { listOf(it, "$it?showUpdate=true", "$it?showUpdate=false") } +
            listOf("ext", "ext/check-updates")
        repeat(20) {
            val index = index()
            for (route in routes) KeyboardRenderer(arrayOf(font)).use { n ->
                val s = open(n, index, route)
                assertEquals(ReferenceExtensionScreens.showsImport(route), node(s, 200001) != null, route)
                assertEquals(ReferenceExtensionScreens.showsUpdate(route), node(s, 200002) != null, route)
                if (route.startsWith("ext/list/")) {
                    val listed = ReferenceExtensionScreens.listed(route, index.keyboards, index.themes, index.packs)
                    val titles = listed.indices.map { e -> node(s, 210_000 + 8 * e)?.text("label") }
                    assertEquals(listed.map { it.meta.title }, titles, route)
                    assertNull(node(s, 210_000 + 8 * listed.size), route)
                    // Compose disabled Edit on an extension it could not delete; Rust leaves it out.
                    assertEquals(listed.map { it.meta.id in index.deletable }, listed.indices.map { e -> node(s, 210_004 + 8 * e) != null })
                }
            }
        }
    }

    @Test fun thePageShowsWhatExtensionViewScreenShowed() {
        repeat(40) {
            val index = index()
            for (ext in index.all) KeyboardRenderer(arrayOf(font)).use { n ->
                val s = open(n, index, "ext/view/${ext.meta.id}")
                val m = ext.meta
                // Difference: an empty description took a blank line in Compose; here it takes none.
                assertEquals(!m.description.isNullOrEmpty(), node(s, 300_000) != null)
                assertEquals(m.description?.takeIf { it.isNotEmpty() }, node(s, 300_000)?.text("label"))
                for ((k, value) in listOf(1 to m.id, 2 to m.version, 6 to m.license)) assertEquals(value, node(s, 300_020 + k)!!.text("state"))
                assertEquals(ReferenceExtensionScreens.showsKeywords(m.keywords), node(s, 300_013) != null)
                for ((k, link) in listOf(4 to m.homepage, 5 to m.issueTracker)) {
                    assertEquals(ReferenceExtensionScreens.showsLink(link), node(s, 300_020 + k) != null, "$k $link")
                    assertEquals(ReferenceExtensionScreens.showsLink(link), node(s, 300_010 + k) != null, "$k $link")
                    if (ReferenceExtensionScreens.showsLink(link)) assertEquals(listOf("host of $link", link, "link"),
                        node(s, 300_020 + k)!!.let { l -> listOf(l.text("label"), l.text("state"), l.text("role")) })
                }
                for ((i, maintainer) in m.maintainers.withIndex()) {
                    val chip = node(s, 301_000 + i)!!
                    assertEquals(maintainer.name, chip.text("label"))
                    assertEquals(if (ReferenceExtensionScreens.maintainerEnabled(maintainer)) "button" else "text", chip.text("role"))
                }
                assertEquals(ext.meta.id in index.deletable, node(s, 300_030) != null)
                assertNotNull(node(s, 300_031))
            }
        }
    }

    @Test fun aMaintainersDialogOffersWhatExtensionMaintainerChipsDialogOffered() {
        repeat(20) {
            val index = index()
            for (ext in index.all) for ((i, maintainer) in ext.meta.maintainers.withIndex()) KeyboardRenderer(arrayOf(font)).use { n ->
                open(n, index, "ext/view/${ext.meta.id}")
                val s = state(n, SettingsUiOp.ACTIVATE, (301_000 + i).toString())
                val links = ReferenceExtensionScreens.maintainerLinks(maintainer)
                assertEquals(links.map { it.removePrefix("mailto:") }, links.indices.map { c -> node(s, 3000 + c)?.text("label") })
                if (links.isNotEmpty()) assertEquals(listOf(listOf("url", links.last())),
                    effects(state(n, SettingsUiOp.ACTIVATE, (3000 + links.lastIndex).toString())))
            }
        }
    }

    @Test fun eachComponentSaysWhatExtensionComponentViewSaid() {
        repeat(30) {
            val index = index()
            for (ext in index.themes + index.packs) KeyboardRenderer(arrayOf(font)).use { n ->
                val s = open(n, index, "ext/view/${ext.meta.id}")
                val components = (ext as? ThemeExtension)?.themes ?: (ext as LanguagePackExtension).items
                assertEquals(components.map { ReferenceExtensionScreens.componentText(it) },
                    components.indices.map { c -> node(s, 310_002 + 4 * c)?.text("label") })
                assertEquals(components.isEmpty(), node(s, 300_041) != null)
            }
        }
    }

    /** Improvement: FlorisConfirmDeleteDialog was held in a plain `remember`, so recreation closed it. */
    @Test fun theDeleteQuestionSurvivesRecreationAndDeletesWhatItNamed() {
        val index = Index(emptyList(), listOf(theme("t0.ext"), theme("t1.ext")), emptyList(), setOf("t0.ext", "t1.ext"))
        KeyboardRenderer(arrayOf(font)).use { n ->
            open(n, index, "ext/list/EXT_THEME")
            state(n, SettingsUiOp.NAVIGATE, "ext/view/t1.ext")
            state(n, SettingsUiOp.ACTIVATE, "300030")
            val saved = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            KeyboardRenderer(arrayOf(font)).use { recreated ->
                configure(recreated)
                assertEquals(listOf(listOf(SettingsUiExtensionEffect.LOAD)), effects(state(recreated, SettingsUiOp.RESTORE, *saved)))
                val s = state(recreated, SettingsUiOp.EXTENSIONS, *report(index))
                assertTrue(nodes(s).any { it.text("label") == "Delete Title t1.ext?" }, nodes(s).map { it.text("label") }.toString())
                assertEquals(listOf(listOf(SettingsUiExtensionEffect.DELETE, "t1.ext")), effects(state(recreated, SettingsUiOp.ACTIVATE, "7")))
                // ExtensionViewScreen popped itself once the delete worked, and stayed when it failed.
                val done = { ok: Boolean -> SettingsUiExtensionDone.row(SettingsUiExtensionDone.WORK to SettingsUiExtensionWork.DELETE,
                    SettingsUiExtensionDone.EXTENSION to "t1.ext", SettingsUiExtensionDone.DONE to WireFlag.encode(ok)) }
                assertEquals("ext/view/t1.ext", state(recreated, SettingsUiOp.EXTENSION_DONE, *done(false)).text("route"))
                assertEquals("ext/list/EXT_THEME", state(recreated, SettingsUiOp.EXTENSION_DONE, *done(true)).text("route"))
            }
        }
    }
}
