package com.jtn0123.renderprobe;

import android.app.Activity;
import android.content.Context;
import android.os.Bundle;
import android.text.Editable;
import android.text.TextWatcher;
import android.view.inputmethod.InputMethodManager;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.TextView;
import java.io.File;
import java.io.FileOutputStream;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;

/** Temporary local measurement editor. It stores only count and SHA-256, never entered text. */
public final class ProbeActivity extends Activity {
    private EditText editor;
    private TextView result;

    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(32, 110, 32, 32);
        TextView title = new TextView(this);
        title.setText("Keyboard renderer comparison");
        title.setTextSize(22);
        root.addView(title);
        Button reset = new Button(this);
        reset.setText("Clear editor");
        root.addView(reset);
        editor = new EditText(this) {
            @Override protected void onSelectionChanged(int start, int end) {
                super.onSelectionChanged(start, end);
                if (result != null) saveDigest(getText());
            }
        };
        editor.setContentDescription("Benchmark editor");
        editor.setInputType(android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_FLAG_MULTI_LINE);
        editor.setTextSize(18);
        root.addView(editor, new LinearLayout.LayoutParams(-1, 350));
        result = new TextView(this);
        root.addView(result);
        editor.addTextChangedListener(new TextWatcher() {
            public void beforeTextChanged(CharSequence s, int start, int count, int after) { }
            public void onTextChanged(CharSequence s, int start, int before, int count) { }
            public void afterTextChanged(Editable text) { saveDigest(text); }
        });
        reset.setOnClickListener(view -> {
            editor.setText("");
            editor.requestFocus();
            ((InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE)).showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT);
        });
        setContentView(root);
        saveDigest(editor.getText());
    }

    private void saveDigest(Editable text) {
        try {
            byte[] hash = MessageDigest.getInstance("SHA-256").digest(text.toString().getBytes(StandardCharsets.UTF_8));
            StringBuilder hex = new StringBuilder();
            for (byte b : hash) hex.append(String.format(java.util.Locale.ROOT, "%02x", b & 255));
            String json = "{\"utf16_units\":" + text.length() + ",\"sha256\":\"" + hex + "\",\"selection_start\":" + editor.getSelectionStart() + ",\"selection_end\":" + editor.getSelectionEnd() + "}";
            try (FileOutputStream out = new FileOutputStream(new File(getFilesDir(), "digest.json"))) {
                out.write(json.getBytes(StandardCharsets.UTF_8));
            }
            result.setText("UTF-16 units: " + text.length() + "\nSHA-256: " + hex);
        } catch (Exception exception) {
            throw new IllegalStateException("Could not save benchmark digest", exception);
        }
    }
}
