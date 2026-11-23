package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import com.jtn0123.audiowhisper.rust.SettingsUiReply
import kotlinx.serialization.json.*
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * Every way into settings opens the Rust host, which picks the first screen as FlorisAppActivity
 * and SetupScreen did (stage4-setup.md): setup until it is finished, unless a settings link asks
 * for a screen. The intents are the ones the app sends or declares.
 */
class SettingsEntryTest {
    private val view = "android.intent.action.VIEW"
    private val send = "android.intent.action.SEND"
    private val main = "android.intent.action.MAIN"

    /** The screen a fresh activity opens on, and the preferences it writes, as FlorisAppActivity asks. */
    private fun start(setUp: Boolean, notifications: String, action: String?, data: String?): Pair<String, List<List<String>>> =
        KeyboardRenderer(emptyArray()).use { n ->
            n.settings(SettingsUiOp.PREFERENCES, arrayOf("internal__is_ime_set_up", setUp.toString(), "internal__notification_permission_state", notifications))
            val link = SettingsEntry.route(action, data, data, data) { "0" }
            val state = Json.parseToJsonElement(n.settings(SettingsUiOp.START, listOfNotNull(link).toTypedArray())[SettingsUiReply.STATE]).jsonObject
            val writes = state.getValue("effects").jsonArray.map { e -> e.jsonArray.map { it.jsonPrimitive.content } }.filter { it[0] == "write" }
            state.getValue("route").jsonPrimitive.content to writes
        }

    @Test fun theLauncherAndTheKeyboardsSettingsButtonOpenSetupUntilItIsFinished() {
        // SettingsLauncherAlias sends MAIN; FlorisImeService.launchSettings sends no action or data.
        for (action in listOf(main, null)) {
            assertEquals("settings/home" to emptyList(), start(setUp = true, "GRANTED", action, null))
            assertEquals("settings/home" to emptyList(), start(setUp = true, "DENIED", action, null))
            assertEquals("setup" to emptyList(), start(setUp = false, "GRANTED", action, null), "Setup still comes first")
        }
    }

    @Test fun anUnansweredNotificationQuestionSendsAFinishedUserThroughSetupAgain() {
        val reset = listOf(listOf("write", "internal__is_ime_set_up", "false"))
        assertEquals("setup" to reset, start(setUp = true, "NOT_SET", main, null))
        assertEquals("setup" to emptyList(), start(setUp = false, "NOT_SET", main, null))
    }

    @Test fun aSettingsLinkOpensItsScreenEvenDuringSetup() {
        // Dictation's "model settings" button, and any other ui://florisboard link.
        for (setUp in listOf(true, false)) {
            assertEquals("settings/dictation", start(setUp, "GRANTED", view, "ui://florisboard/settings/dictation").first)
        }
        assertEquals("settings/dictation", start(setUp = true, "NOT_SET", view, "ui://florisboard/settings/dictation").first)
        assertEquals("settings/dictation", SettingsEntry.linkRoute(view, "ui://florisboard/settings/dictation"))
        assertEquals("ext/view?id=a", SettingsEntry.linkRoute(view, "ui://florisboard/ext/view?id=a"))
    }

    @Test fun anExtensionFileIsNotASettingsLink() {
        assertNull(SettingsEntry.linkRoute(view, "content://downloads/1"))
        assertNull(SettingsEntry.linkRoute(send, null))
        assertNull(SettingsEntry.linkRoute(send, "ui://florisboard/settings/home"))
        assertNull(SettingsEntry.linkRoute(main, null))
    }

    /** An extension file opened in, or shared with, the app starts the Rust importer on it, set up or not, where
     * FlorisAppActivity handed it to the Compose fallback. Only the document it imports is kept. */
    @Test fun anOpenedOrSharedFileStartsTheImporterOnIt() {
        val kept = mutableListOf<String>()
        val keep = { d: String -> kept += d; "${kept.size - 1}" }
        assertEquals("ext/import/EXT_ANY?document=0", SettingsEntry.route(view, "content://downloads/1", "opened", "clip", keep))
        assertEquals("ext/import/EXT_ANY?document=1", SettingsEntry.route(send, null, "data", "shared", keep))
        assertNull(SettingsEntry.route(send, null, "data", null, keep))
        assertNull(SettingsEntry.route(view, null, null, "clip", keep))
        assertNull(SettingsEntry.route(main, "content://downloads/1", "data", "clip", keep))
        assertEquals("settings/dictation", SettingsEntry.route(view, "ui://florisboard/settings/dictation", "link", null, keep))
        assertEquals(listOf("opened", "shared"), kept)
        for (setUp in listOf(true, false)) {
            assertEquals("ext/import/EXT_ANY?document=0", start(setUp, "GRANTED", view, "content://downloads/1").first)
            assertEquals("ext/import/EXT_ANY?document=0", start(setUp, "GRANTED", send, "content://downloads/1").first)
        }
    }

    /** The screens open, bottom first, once [intents] reach the renderer as FlorisAppActivity hands them on: the
     * first as the intent a fresh activity opens with, the rest as later ones (onNewIntent). */
    private fun open(setUp: Boolean, vararg intents: Pair<String?, String?>): List<String> =
        KeyboardRenderer(emptyArray()).use { n ->
            n.settings(SettingsUiOp.PREFERENCES, arrayOf("internal__is_ime_set_up", setUp.toString(), "internal__notification_permission_state", "GRANTED"))
            intents.forEachIndexed { i, (action, data) ->
                val link = SettingsEntry.route(action, data, data, data) { "0" }
                SettingsEntry.request(fresh = i == 0, link)?.let { (operation, input) -> n.settings(operation, input) }
            }
            stack(n)
        }

    private fun stack(n: KeyboardRenderer) = n.settings(SettingsUiOp.SAVE)[SettingsUiReply.SIZE + 1].split("|")
    private fun link(route: String) = view to "ui://florisboard/$route"
    private val home = "settings/home"
    private val notFound = "settings/not-found"
    private val importer = "ext/import/EXT_ANY?document=0"
    private val file = "content://downloads/1"

    /** Each way into settings (stage4-compose-host.md's table) and the Rust screens it leaves open. */
    @Test fun everyWayIntoSettingsLandsOnItsRustScreen() {
        val dictation = "settings/dictation"
        // The launcher, and Android's keyboard and spell-checker settings (method.xml, spellchecker.xml), open
        // SettingsLauncherAlias with MAIN; the keyboard's settings button (FlorisImeService.launchSettings) sends none.
        for (action in listOf(main, null)) {
            assertEquals(listOf(home), open(true, action to null))
            assertEquals(listOf("setup"), open(false, action to null))
            // With settings open, neither moves them: singleTask hands the intent to the open activity.
            assertEquals(listOf(home, dictation), open(true, link(dictation), action to null))
        }
        // Setup's return (SettingsSetup's RETURN effect) carries no link and leaves setup where it was.
        assertEquals(listOf("setup"), open(false, main to null, null to null))
        assertEquals(listOf("setup"), open(true, link("setup")), "a setup link opens setup, set up or not")
        // A settings link, fresh or into open settings: dictation's model button sends this one.
        assertEquals(listOf(home, dictation), open(true, link(dictation)))
        assertEquals(listOf(home, dictation), open(false, link(dictation)))
        assertEquals(listOf(home, "settings/about", dictation), open(true, link("settings/about"), link(dictation)))
        // ThemeEditorActivity opens on its own link, as a root.
        assertEquals(listOf("settings/theme/edit/u/0"), open(true, link("settings/theme/edit/u/0")))
        // A .flex file opened (VIEW) or shared (SEND): the importer, fresh or over open settings.
        for (action in listOf(view, send)) {
            assertEquals(listOf(home, importer), open(true, action to file))
            assertEquals(listOf(home, importer), open(false, action to file), "over home, as any link during setup")
            assertEquals(listOf(home, dictation, importer), open(true, link(dictation), action to file))
        }
        // A link to a page Rust does not draw: its not-found page, over whatever was open, once.
        assertEquals(listOf(home, notFound), open(true, link("settings/gone")))
        assertEquals(listOf(home, notFound), open(false, link("settings/gone")))
        assertEquals(listOf(home, dictation, notFound), open(true, link(dictation), link("settings/gone"), link("x")))
    }

    /** Rotation and process death: the activity saves the stack each of those intents left, and a new renderer opens
     * it again, the not-found page and the importer included. */
    @Test fun recreationReopensTheScreensEachEntryLeft() {
        for (intents in listOf(arrayOf(main to null), arrayOf(link("settings/gone")), arrayOf(link("settings/dictation"), link("x")),
            arrayOf(view to file), arrayOf(link("settings/theme/edit/u/0")))) {
            KeyboardRenderer(emptyArray()).use { n ->
                n.settings(SettingsUiOp.PREFERENCES, arrayOf("internal__is_ime_set_up", "true", "internal__notification_permission_state", "GRANTED"))
                for ((i, intent) in intents.withIndex()) {
                    val link = SettingsEntry.route(intent.first, intent.second, intent.second, intent.second) { "0" }
                    SettingsEntry.request(i == 0, link)?.let { (operation, input) -> n.settings(operation, input) }
                }
                val saved = n.settings(SettingsUiOp.SAVE).drop(SettingsUiReply.SIZE).toTypedArray()
                KeyboardRenderer(emptyArray()).use { recreated ->
                    recreated.settings(SettingsUiOp.RESTORE, saved)
                    assertEquals(stack(n), stack(recreated), intents.joinToString { "${it.first} ${it.second}" })
                }
            }
        }
    }

    /** The manifest sends every one of those intents to the Rust host: FlorisAppActivity, through the launcher alias
     * the system settings name too, or ThemeEditorActivity, which is one. No other activity takes a settings link. */
    @Test fun theManifestSendsEverySettingsIntentToTheRustHost() {
        fun xml(path: String) = DocumentBuilderFactory.newInstance().apply { isNamespaceAware = true }.newDocumentBuilder().parse(File(path)).documentElement
        val android = "http://schemas.android.com/apk/res/android"
        fun org.w3c.dom.Element.attr(name: String) = getAttributeNS(android, name)
        fun org.w3c.dom.Element.all(tag: String) = getElementsByTagName(tag).let { l -> (0 until l.length).map { l.item(it) as org.w3c.dom.Element } }
        val app = xml("src/main/AndroidManifest.xml")
        val host = "dev.patrickgold.florisboard.app.FlorisAppActivity"
        val activities = app.all("activity").associateBy { it.attr("name") }
        val filters = activities.getValue(host).all("intent-filter").map { f ->
            f.all("action").map { it.attr("name") } to f.all("data").map { it.attr("scheme") + it.attr("host") + it.attr("pathPrefix") + it.attr("mimeType") }
        }
        val flex = listOf("application/vnd.florisboard.extension+zip", "application/octet-stream")
        assertEquals(listOf(listOf(view) to listOf("uiflorisboard/"), listOf(view) to listOf("content") + flex, listOf(send) to flex), filters)
        assertEquals("singleTask", activities.getValue(host).attr("launchMode"))
        val alias = app.all("activity-alias").single()
        assertEquals(host, alias.attr("targetActivity"))
        assertEquals(listOf(main), alias.all("action").map { it.attr("name") })
        for (path in listOf("src/main/res/xml/method.xml", "src/main/res/xml/spellchecker.xml")) {
            assertEquals(alias.attr("name"), xml(path).attr("settingsActivity"), path)
        }
        assertEquals<Class<*>>(FlorisAppActivity::class.java, ThemeEditorActivity::class.java.superclass)
        assertEquals("false", activities.getValue(ThemeEditorActivity::class.java.name).attr("exported"))
        for ((name, activity) in activities - host) {
            val data = activity.all("data")
            assertTrue(data.none { it.attr("scheme") == "ui" || it.attr("mimeType") in flex }, "$name takes a settings intent")
        }
    }
}
