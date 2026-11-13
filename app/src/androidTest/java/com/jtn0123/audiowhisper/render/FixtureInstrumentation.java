package com.jtn0123.audiowhisper.render;

import android.app.Instrumentation;
import android.app.Activity;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.os.Bundle;
import com.jtn0123.audiowhisper.gif.GifSearch;
import dev.patrickgold.florisboard.FlorisApplication;
import dev.patrickgold.florisboard.ime.ImeUiMode;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.security.MessageDigest;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

/** Test-APK-only fixture transport, using the Android instrumentation API directly. */
public final class FixtureInstrumentation extends Instrumentation {
    @Override public void onCreate(Bundle arguments) { super.onCreate(arguments); start(); }
    @Override public void onStart() {
        Context context = getTargetContext();
        CountDownLatch done = new CountDownLatch(1);
        BroadcastReceiver receiver = new BroadcastReceiver() {
            @Override public void onReceive(Context ignored, Intent intent) {
                GifSearch search = GifSearch.INSTANCE;
                FlorisApplication app = (FlorisApplication) context.getApplicationContext();
                String op = intent.getStringExtra("op");
                if ("search".equals(op)) {
                    search.startTyping();
                    app.getKeyboardManager().getValue().getActiveState().setImeUiMode(ImeUiMode.TEXT);
                } else if ("stop".equals(op)) {
                    search.stopTyping();
                    app.getKeyboardManager().getValue().getActiveState().setImeUiMode(ImeUiMode.MEDIA);
                } else if ("finish".equals(op)) done.countDown();
                else if ("query".equals(op)) {
                    try {
                        String query = search.getQuery().getValue();
                        byte[] digest = MessageDigest.getInstance("SHA-256").digest(query.getBytes(StandardCharsets.UTF_8));
                        StringBuilder hex = new StringBuilder();
                        for (byte b : digest) hex.append(String.format(java.util.Locale.ROOT, "%02x", b & 255));
                        String json = "{\"utf16_units\":" + query.length() + ",\"sha256\":\"" + hex + "\",\"typing\":" + search.getTyping().getValue() + ",\"key_available\":" + search.getAvailable() + "}";
                        Files.write(new File(context.getFilesDir(), "render-query-fixture.json").toPath(), json.getBytes(StandardCharsets.UTF_8));
                    } catch (Exception failure) { throw new IllegalStateException("Fixture digest failed", failure); }
                }
            }
        };
        runOnMainSync(() -> context.registerReceiver(receiver, new IntentFilter("com.jtn0123.audiowhisper.RENDER_FIXTURE"), Context.RECEIVER_EXPORTED));
        Bundle result = new Bundle();
        try { if (!done.await(30, TimeUnit.MINUTES)) throw new IllegalStateException("Fixture controller timed out"); result.putBoolean("fixture_complete", true); }
        catch (InterruptedException failure) { Thread.currentThread().interrupt(); throw new IllegalStateException(failure); }
        finally { runOnMainSync(() -> { context.unregisterReceiver(receiver); GifSearch.INSTANCE.reset(); }); }
        finish(Activity.RESULT_OK, result);
    }
}
