package com.jtn0123.audiowhisper.suggest;

import android.app.Activity;
import android.app.Instrumentation;
import android.os.Bundle;
import java.io.File;
import java.io.InputStream;
import java.nio.file.Files;

/** Runs the identical JVM corpus in the target process, reporting digests only. */
public final class A1Instrumentation extends Instrumentation {
    private boolean boundaries;
    private String only;
    @Override public void onCreate(Bundle arguments) {
        super.onCreate(arguments);
        boundaries = arguments != null && "true".equals(arguments.getString("boundaries"));
        only = arguments == null ? null : arguments.getString("only");
        start();
    }
    @Override public void onStart() { replay(this, boundaries, only); }

    /** Shared by the combined renderer/A1 test APK; assertions and accepted digests are unchanged. */
    public static void replay(Instrumentation instrumentation, boolean boundaries) { replay(instrumentation, boundaries, null); }

    /**
     * As above; [only] ("a1" or "extension") runs one of the two replays, so a run that writes one
     * family's golden files (scripts/device-goldens.py) meets no other family's tapes.
     */
    public static void replay(Instrumentation instrumentation, boolean boundaries, String only) {
        Bundle result = new Bundle();
        try {
            if (boundaries) {
                result.putString("boundaries", BoundaryReplay.run(instrumentation.getTargetContext()));
                instrumentation.finish(Activity.RESULT_OK, result);
                return;
            }
            if (only != null && !only.equals("a1") && !only.equals("extension")) throw new IllegalArgumentException("only a1 or extension");
            if ("extension".equals(only)) {
                result.putLong("extension_maintainer_checks", dev.patrickgold.florisboard.lib.ext.ExtensionDeviceReplay.INSTANCE.run());
                instrumentation.finish(Activity.RESULT_OK, result);
                return;
            }
            File dictionary = new File(instrumentation.getTargetContext().getFilesDir(), "a1-fixture.dict");
            try (InputStream input = instrumentation.getTargetContext().getAssets().open("dicts/main_en-US.dict")) {
                Files.copy(input, dictionary.toPath(), java.nio.file.StandardCopyOption.REPLACE_EXISTING);
            }
            File output = new File(instrumentation.getTargetContext().getFilesDir(), "a1-replay");
            kotlin.Pair<String, String> hashes = A1Replay.INSTANCE.capture(dictionary, output, false);
            result.putString("decoder_sha256", hashes.getFirst());
            result.putString("pipeline_sha256", hashes.getSecond());
            if (only == null) result.putLong("extension_maintainer_checks", dev.patrickgold.florisboard.lib.ext.ExtensionDeviceReplay.INSTANCE.run());
            dictionary.delete();
            instrumentation.finish(Activity.RESULT_OK, result);
        } catch (Exception | AssertionError failure) { // A golden tape fails with an AssertionError.
            result.putString("failure", failure.getClass().getName() + ": " + failure.getMessage());
            java.io.StringWriter trace = new java.io.StringWriter();
            failure.printStackTrace(new java.io.PrintWriter(trace));
            result.putString("fixture_trace", trace.toString());
            instrumentation.finish(Activity.RESULT_CANCELED, result);
        }
    }
}
