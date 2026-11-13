package com.jtn0123.renderprobe;

import android.app.Activity;
import android.content.Context;
import android.content.BroadcastReceiver;
import android.content.Intent;
import android.content.IntentFilter;
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
    static boolean emptyEditor;
    private EditText editor;
    private TextView result;
    private final BroadcastReceiver commands = new BroadcastReceiver() {
        @Override public void onReceive(Context context, Intent intent) {
            editor.setInputType(inputType(intent));
            editor.setText("");
            editor.requestFocus();
            ((InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE))
                .showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT);
            saveDigest(editor.getText());
        }
    };

    @Override public void onDestroy() {
        unregisterReceiver(commands);
        super.onDestroy();
    }

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
        editor.setInputType(inputType(getIntent()));
        editor.setTextSize(18);
        root.addView(editor, new LinearLayout.LayoutParams(-1, 350));
        result = new TextView(this);
        root.addView(result);
        LinearLayout fields = new LinearLayout(this);
        for (String field : new String[]{"text", "number", "phone", "password"}) {
            Button button = new Button(this);
            button.setText(field);
            button.setOnClickListener(view -> {
                Intent options = new Intent().putExtra("field", field).putExtra("noSuggestions", true);
                editor.setInputType(inputType(options));
                editor.setText("");
                editor.requestFocus();
                ((InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE))
                    .showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT);
            });
            fields.addView(button, new LinearLayout.LayoutParams(0, -2, 1));
        }
        root.addView(fields);
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
        registerReceiver(commands, new IntentFilter("com.jtn0123.renderprobe.RESET"), Context.RECEIVER_EXPORTED);
        saveDigest(editor.getText());
        editor.requestFocus();
        editor.postDelayed(() -> ((InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE))
            .showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT), 400);
    }

    private static int inputType(Intent intent) {
        String field = intent.getStringExtra("field");
        if ("number".equals(field)) return android.text.InputType.TYPE_CLASS_NUMBER;
        if ("decimal".equals(field)) return android.text.InputType.TYPE_CLASS_NUMBER
            | android.text.InputType.TYPE_NUMBER_FLAG_DECIMAL | android.text.InputType.TYPE_NUMBER_FLAG_SIGNED;
        if ("phone".equals(field)) return android.text.InputType.TYPE_CLASS_PHONE;
        if ("password".equals(field)) return android.text.InputType.TYPE_CLASS_TEXT
            | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD;
        return android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_FLAG_MULTI_LINE
            | (intent.getBooleanExtra("noSuggestions", false) ? android.text.InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS : 0);
    }

    private void saveDigest(Editable text) {
        emptyEditor = text.length() == 0;
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
