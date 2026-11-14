package dev.patrickgold.florisboard.app

/** Existing settings screens in a separate Activity retain the suspended native navigation state. */
class ComposeSettingsFallbackActivity : FlorisAppActivity() {
    override val allowRustSettings = false
}
