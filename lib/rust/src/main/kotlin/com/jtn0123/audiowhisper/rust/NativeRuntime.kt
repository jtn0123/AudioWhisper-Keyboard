package com.jtn0123.audiowhisper.rust

/** Android/JVM library bootstrap, usable before any preference or Compose initialization. */
object NativeRuntime {
    @Volatile var failure: Throwable? = null
        private set
    private var loaded = false

    @Synchronized fun load(loader: () -> Unit = { System.loadLibrary("awnative") }): Boolean {
        if (failure != null) return false
        if (loaded) return true
        return try {
            loader()
            loaded = true
            true
        } catch (error: LinkageError) {
            failure = error
            false
        } catch (error: SecurityException) {
            failure = error
            false
        }
    }

    fun requireLoaded() {
        if (!load()) throw UnsatisfiedLinkError("AudioWhisper native engine unavailable")
    }
}
