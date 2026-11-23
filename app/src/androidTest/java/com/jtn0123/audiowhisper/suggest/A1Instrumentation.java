package com.jtn0123.audiowhisper.suggest;

import android.app.Activity;
import android.app.Instrumentation;
import android.os.Bundle;
import com.jtn0123.audiowhisper.golden.DeviceGoldens;
import com.jtn0123.audiowhisper.golden.GoldenTape;
import java.io.File;
import java.io.InputStream;
import java.nio.file.Files;

/** Runs the identical JVM corpus in the target process, reporting digests only. */
public final class A1Instrumentation extends Instrumentation {
    private boolean boundaries;
    private String only;
    private Bundle arguments;
    @Override public void onCreate(Bundle arguments) {
        super.onCreate(arguments);
        this.arguments = arguments;
        boundaries = arguments != null && "true".equals(arguments.getString("boundaries"));
        only = arguments == null ? null : arguments.getString("only");
        start();
    }
    @Override public void onStart() { replay(this, boundaries, only, new DeviceGoldens(this, arguments)); }

    /**
     * Shared by the combined renderer/A1 test APK; assertions and accepted digests are unchanged.
     * [only] ("a1" or "extension") runs one of the two replays, so a run that writes one family's
     * golden files (scripts/device-goldens.py) meets no other family's tapes; [goldens] holds them.
     */
    public static void replay(Instrumentation instrumentation, boolean boundaries, String only, GoldenTape.Store goldens) {
        Bundle result = new Bundle();
        try {
            if (boundaries) {
                result.putString("boundaries", BoundaryReplay.run(instrumentation.getTargetContext(), goldens));
                instrumentation.finish(Activity.RESULT_OK, result);
                return;
            }
            if (only != null && !only.equals("a1") && !only.equals("extension")) throw new IllegalArgumentException("only a1 or extension");
            if ("extension".equals(only)) {
                result.putLong("extension_maintainer_checks", dev.patrickgold.florisboard.lib.ext.ExtensionDeviceReplay.INSTANCE.run(goldens));
                instrumentation.finish(Activity.RESULT_OK, result);
                return;
            }
            File dictionary = new File(instrumentation.getTargetContext().getFilesDir(), "a1-fixture.dict");
            try (InputStream input = instrumentation.getTargetContext().getAssets().open("dicts/main_en-US.dict")) {
                Files.copy(input, dictionary.toPath(), java.nio.file.StandardCopyOption.REPLACE_EXISTING);
            }
            File output = new File(instrumentation.getTargetContext().getFilesDir(), "a1-replay");
            kotlin.Pair<String, String> hashes = A1Replay.INSTANCE.capture(dictionary, output);
            result.putString("decoder_sha256", hashes.getFirst());
            result.putString("pipeline_sha256", hashes.getSecond());
            // The host wrote these goldens, from the AOSP engine glue, which is not built for Android.
            A1Replay.INSTANCE.record(output, "main_en-US", name -> GoldenTape.Companion.invoke(name, goldens));
            if (only == null) result.putLong("extension_maintainer_checks", dev.patrickgold.florisboard.lib.ext.ExtensionDeviceReplay.INSTANCE.run(goldens));
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
