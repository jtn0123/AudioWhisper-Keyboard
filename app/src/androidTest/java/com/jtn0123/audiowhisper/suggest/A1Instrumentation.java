package com.jtn0123.audiowhisper.suggest;

import android.app.Activity;
import android.app.Instrumentation;
import android.os.Bundle;
import java.io.File;
import java.io.InputStream;
import java.nio.file.Files;

/** Runs the identical JVM corpus in the target process, reporting digests only. */
public final class A1Instrumentation extends Instrumentation {
    @Override public void onCreate(Bundle arguments) { super.onCreate(arguments); start(); }
    @Override public void onStart() {
        Bundle result = new Bundle();
        try {
            File dictionary = new File(getTargetContext().getFilesDir(), "a1-fixture.dict");
            try (InputStream input = getTargetContext().getAssets().open("dicts/main_en-US.dict")) {
                Files.copy(input, dictionary.toPath(), java.nio.file.StandardCopyOption.REPLACE_EXISTING);
            }
            File output = new File(getTargetContext().getFilesDir(), "a1-replay");
            kotlin.Pair<String, String> hashes = A1Replay.INSTANCE.capture(dictionary, output, false);
            result.putString("decoder_sha256", hashes.getFirst());
            result.putString("pipeline_sha256", hashes.getSecond());
            result.putLong("extension_maintainer_checks", dev.patrickgold.florisboard.lib.ext.ExtensionDeviceReplay.INSTANCE.run());
            dictionary.delete();
            finish(Activity.RESULT_OK, result);
        } catch (Exception failure) {
            result.putString("failure", failure.getClass().getName() + ": " + failure.getMessage());
            java.io.StringWriter trace = new java.io.StringWriter();
            failure.printStackTrace(new java.io.PrintWriter(trace));
            result.putString("fixture_trace", trace.toString());
            finish(Activity.RESULT_CANCELED, result);
        }
    }
}
