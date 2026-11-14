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
    /** Use the kept JNI transport: R8 removes the production's unused low-level wrapper.
     * Find only this app's live service reference, then execute the returned batch effects
     * against Android's real InputConnection after releasing the native session lock.
     */
    private boolean invalidFinalize() throws ReflectiveOperationException {
        android.view.inputmethod.InputConnection connection = null;
        for (java.lang.reflect.Field field : Class.forName("dev.patrickgold.florisboard.FlorisImeServiceKt").getDeclaredFields()) {
            if (!java.lang.reflect.Modifier.isStatic(field.getModifiers())
                || !java.lang.ref.WeakReference.class.isAssignableFrom(field.getType())) continue;
            field.setAccessible(true);
            Object reference = field.get(null);
            Object service = reference instanceof java.lang.ref.WeakReference ? ((java.lang.ref.WeakReference<?>)reference).get() : null;
            if (service instanceof android.inputmethodservice.InputMethodService) {
                connection = ((android.inputmethodservice.InputMethodService)service).getCurrentInputConnection();
                break;
            }
        }
        if (connection == null) throw new IllegalStateException("No live fixture InputConnection");
        long handle = com.jtn0123.audiowhisper.rust.EditorNative.create();
        try {
            String[] plan = com.jtn0123.audiowhisper.rust.EditorNative.call(handle,"finalize",new String[]{""});
            if (!"ready".equals(plan[0])) throw new IllegalStateException("Unexpected fixture query");
            for (int i=2;i<plan.length;i+=4) {
                if ("begin".equals(plan[i])) connection.beginBatchEdit();
                else if ("end".equals(plan[i])) connection.endBatchEdit();
                else throw new IllegalStateException("Unexpected invalid-finalize effect");
            }
            return "1".equals(plan[1]);
        } finally {com.jtn0123.audiowhisper.rust.EditorNative.free(handle);}
    }

    /** Uses Java I/O so an unchanged old APK need not retain unused Kotlin file facades. */
    private void readerState(Context context) {
        try {
            org.json.JSONObject report = new org.json.JSONObject().put("status",
                com.jtn0123.audiowhisper.reader.ReadAloud.INSTANCE.getStatus().getValue().name());
            java.lang.reflect.Field serviceField = com.jtn0123.audiowhisper.reader.ReadAloud.class.getDeclaredField("service");
            serviceField.setAccessible(true);
            Object service = serviceField.get(null);
            Object voice = null;
            if (service != null) {
                java.lang.reflect.Field speaking = service.getClass().getDeclaredField("speaking");
                speaking.setAccessible(true);
                voice = speaking.get(service);
            }
            boolean neural = voice != null && voice.getClass().getName().endsWith(".NeuralVoice");
            report.put("neural", neural);
            if (neural) {
                java.lang.reflect.Field trackField = voice.getClass().getDeclaredField("track");
                trackField.setAccessible(true);
                android.media.AudioTrack track = (android.media.AudioTrack) trackField.get(voice);
                report.put("playback_head_frames", ((long) track.getPlaybackHeadPosition()) & 0xffff_ffffL);
            }
            File pending = new File(context.getFilesDir(), "reader-probe.json.tmp");
            Files.write(pending.toPath(), report.toString().getBytes(StandardCharsets.UTF_8));
            if (!pending.renameTo(new File(context.getFilesDir(), "reader-probe.json"))) throw new IllegalStateException("Reader probe publish failed");
        } catch (Exception failure) { throw new IllegalStateException("Reader metadata probe failed", failure); }
    }

    @Override public void onCreate(Bundle arguments) { super.onCreate(arguments); start(); }
    @Override public void onStart() {
        Context context = getTargetContext();
        CountDownLatch done = new CountDownLatch(1);
        BroadcastReceiver receiver = new BroadcastReceiver() {
            @Override public void onReceive(Context ignored, Intent intent) {
                GifSearch search = GifSearch.INSTANCE;
                FlorisApplication app = (FlorisApplication) context.getApplicationContext();
                String op = intent.getStringExtra("op");
                if ("acceptance-snapshot".equals(op)) {
                    AcceptanceProbe.snapshot(context);
                } else if ("clipboard-snapshot".equals(op)) {
                    AcceptanceProbe.clipboard(context);
                } else if ("reader-state".equals(op)) {
                    readerState(context);
                } else if ("learned-fixture".equals(op)) {
                    AcceptanceProbe.learnedFixture(context);
                } else if ("backup-change-settings".equals(op)) {
                    AcceptanceProbe.changeBackupSettings();
                } else if ("editor-state".equals(op)) {
                    try {
                        dev.patrickgold.florisboard.ime.editor.EditorInstance editor = app.getEditorInstance().getValue();
                        if (!"com.jtn0123.renderprobe".equals(editor.getActiveInfo().getPackageName()) || editor.getActiveContent().getText().length() != 0)
                            throw new IllegalStateException("Editor metadata requires an empty fixture");
                        String json = "{\"input_type\":" + editor.getActiveInfo().getInputAttributes().getRaw() +
                            ",\"initial_caps\":" + editor.getActiveInfo().getInitialCapsMode().toFlags() +
                            ",\"cursor_caps\":" + editor.getActiveCursorCapsMode().toFlags() +
                            ",\"shift\":" + app.getKeyboardManager().getValue().getActiveState().getInputShiftState().ordinal() + "}";
                        File pending = new File(context.getFilesDir(), "editor-state-fixture.json.tmp");
                        Files.write(pending.toPath(), json.getBytes(StandardCharsets.UTF_8));
                        if (!pending.renameTo(new File(context.getFilesDir(), "editor-state-fixture.json"))) throw new IllegalStateException("Editor fixture publish failed");
                    } catch (Exception failure) { throw new IllegalStateException("Editor state fixture failed", failure); }
                } else if ("nlp-unlearn".equals(op)) {
                    app.getNlpManager().getValue().unlearnAutoCorrection(0);
                } else if ("nlp-state".equals(op)) {
                    try {
                        dev.patrickgold.florisboard.ime.nlp.NlpManager manager = app.getNlpManager().getValue();
                        String json = "{\"candidates\":" + manager.getActiveCandidatesFlow().getValue().size() +
                            ",\"inline\":" + ((java.util.List<?>) ((kotlinx.coroutines.flow.StateFlow<?>) dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill.class.getMethod("getSuggestions").invoke(dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill.INSTANCE)).getValue()).size() + "}";
                        File pending = new File(context.getFilesDir(), "nlp-state-fixture.json.tmp");
                        Files.write(pending.toPath(), json.getBytes(StandardCharsets.UTF_8));
                        if (!pending.renameTo(new File(context.getFilesDir(), "nlp-state-fixture.json"))) throw new IllegalStateException("NLP fixture publish failed");
                    } catch (Exception failure) { throw new IllegalStateException("NLP state fixture failed", failure); }
                } else if ("invalid-finalize".equals(op)) {
                    try {
                        boolean accepted = invalidFinalize();
                        Files.write(new File(context.getFilesDir(), "render-batch-fixture.json").toPath(),
                            ("{\"accepted\":" + accepted + "}").getBytes(StandardCharsets.UTF_8));
                    } catch (Exception failure) {throw new IllegalStateException("Batch fixture failed", failure);}
                } else if ("search".equals(op)) {
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
