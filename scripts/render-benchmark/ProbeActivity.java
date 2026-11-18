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
    static volatile boolean emptyEditor;
    static volatile boolean nlpCaptureAllowed;
    private EditText editor;
    private TextView result;
    private int batchDepth;
    private int connectionGeneration;
    private int editorActions;
    private int lastEditorAction;
    private final BroadcastReceiver commands = new BroadcastReceiver() {
        @Override public void onReceive(Context context, Intent intent) {
            if ("com.jtn0123.renderprobe.CLIP_STATE".equals(intent.getAction())) {
                android.content.ClipboardManager clipboard = getSystemService(android.content.ClipboardManager.class);
                File backup = new File(getFilesDir(), "original-clipboard.parcel");
                android.os.Parcel parcel = android.os.Parcel.obtain();
                try {
                    String operation = intent.getStringExtra("operation");
                    if ("snapshot".equals(operation)) {
                        android.content.ClipData clip = clipboard.getPrimaryClip();
                        parcel.writeInt(clip == null ? 0 : 1);
                        if (clip != null) clip.writeToParcel(parcel, 0);
                        java.nio.file.Files.write(backup.toPath(), parcel.marshall());
                    } else if ("restore".equals(operation)) {
                        byte[] bytes = java.nio.file.Files.readAllBytes(backup.toPath());
                        parcel.unmarshall(bytes, 0, bytes.length); parcel.setDataPosition(0);
                        if (parcel.readInt() == 0) clipboard.clearPrimaryClip();
                        else clipboard.setPrimaryClip(android.content.ClipData.CREATOR.createFromParcel(parcel));
                    } else throw new IllegalArgumentException("Unknown clipboard state operation");
                } catch (java.io.IOException failure) { throw new IllegalStateException("Clipboard fixture state failed", failure); }
                finally { parcel.recycle(); }
                return;
            }
            if ("com.jtn0123.renderprobe.CLIP".equals(intent.getAction())) {
                String value = intent.getStringExtra("fixture");
                android.content.ClipboardManager clipboard = getSystemService(android.content.ClipboardManager.class);
                if (value == null) clipboard.clearPrimaryClip();
                else if (value.matches("aw-test-[1-4]")) clipboard.setPrimaryClip(android.content.ClipData.newPlainText("aw-fixture", value));
                else throw new IllegalArgumentException("Only fixed harmless clipboard fixtures are allowed");
                return;
            }
            if ("com.jtn0123.renderprobe.EDITOR".equals(intent.getAction())) {
                String fixture = intent.getStringExtra("fixture");
                if ("emoji".equals(fixture)) {
                    editor.setText("alpha 👨‍👩‍👧‍👦 😀 e\u0301 omega");
                    editor.setSelection(editor.length());
                } else if ("external".equals(fixture)) {
                    editor.getText().replace(0, Math.min(2, editor.length()), "XY");
                    editor.setSelection(Math.min(1, editor.length()));
                } else if ("repeat".equals(fixture)) {
                    editor.setText("alpha beta gamma");
                    editor.setSelection(editor.length());
                } else if ("repeat-long".equals(fixture)) {
                    editor.setText("alpha beta gamma ".repeat(64));
                    editor.setSelection(editor.length());
                } else if ("cursor-start".equals(fixture)) {
                    editor.setSelection(0);
                } else if ("clear".equals(fixture)) {
                    editor.setText("");
                } else throw new IllegalArgumentException("Unknown fixed editor fixture");
                saveDigest(editor.getText());
                return;
            }
            editor.setInputType(inputType(intent));
            configureAction(intent);
            editorActions = 0;
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
            @Override public android.view.inputmethod.InputConnection onCreateInputConnection(android.view.inputmethod.EditorInfo info) {
                android.view.inputmethod.InputConnection connection = super.onCreateInputConnection(info);
                if (connection == null || !"batch".equals(getIntent().getStringExtra("field"))) return connection;
                final int generation = ++connectionGeneration;
                batchDepth = 0;
                return new android.view.inputmethod.InputConnectionWrapper(connection, false) {
                    private int depth;
                    @Override public boolean beginBatchEdit() {
                        boolean result = super.beginBatchEdit();
                        depth++;
                        publish();
                        return result;
                    }
                    @Override public boolean endBatchEdit() {
                        boolean result = super.endBatchEdit();
                        depth--;
                        publish();
                        return result;
                    }
                    private void publish() {
                        if (generation == connectionGeneration) {
                            batchDepth = depth;
                            if (ProbeActivity.this.result != null) saveDigest(getText());
                        }
                    }
                };
            }
            @Override protected void onSelectionChanged(int start, int end) {
                super.onSelectionChanged(start, end);
                if (result != null) saveDigest(getText());
            }
        };
        editor.setContentDescription("Benchmark editor");
        editor.setInputType(inputType(getIntent()));
        configureAction(getIntent());
        editor.setOnEditorActionListener((view, action, event) -> {
            editorActions++;
            lastEditorAction = action;
            saveDigest(editor.getText());
            return true;
        });
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
        // Opt-in, as many apps do: a tap outside the editor hides the keyboard. Taps on the
        // editor show it again; reshow.py uses this as its "tap outside" path.
        root.setOnTouchListener((view, event) -> {
            if (!getIntent().getBooleanExtra("outsideHides", false)
                || event.getActionMasked() != android.view.MotionEvent.ACTION_DOWN) return false;
            ((InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE))
                .hideSoftInputFromWindow(editor.getWindowToken(), 0);
            return true;
        });
        editor.addTextChangedListener(new TextWatcher() {
            public void beforeTextChanged(CharSequence s, int start, int count, int after) { }
            public void onTextChanged(CharSequence s, int start, int before, int count) { }
            public void afterTextChanged(Editable text) { editor.post(() -> saveDigest(editor.getText())); }
        });
        reset.setOnClickListener(view -> {
            editor.setText("");
            editor.requestFocus();
            ((InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE)).showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT);
        });
        setContentView(root);
        // Read-aloud screenshots retain the real stop/highlight UI without saving its source.
        // The accessibility text is unchanged; only this disposable app's glyphs are hidden.
        if ("reader".equals(getIntent().getStringExtra("field"))) maskText(root);
        IntentFilter filter = new IntentFilter("com.jtn0123.renderprobe.RESET");
        filter.addAction("com.jtn0123.renderprobe.CLIP");
        filter.addAction("com.jtn0123.renderprobe.EDITOR");
        filter.addAction("com.jtn0123.renderprobe.CLIP_STATE");
        registerReceiver(commands, filter, Context.RECEIVER_EXPORTED);
        saveDigest(editor.getText());
        editor.requestFocus();
        editor.postDelayed(() -> ((InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE))
            .showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT), 400);
    }

    private void maskText(android.view.View view) {
        if (view instanceof TextView) {
            ((TextView) view).setTextColor(android.graphics.Color.TRANSPARENT);
            ((TextView) view).setHintTextColor(android.graphics.Color.TRANSPARENT);
        }
        if (view instanceof android.view.ViewGroup) {
            android.view.ViewGroup group = (android.view.ViewGroup) view;
            for (int i = 0; i < group.getChildCount(); i++) maskText(group.getChildAt(i));
        }
    }

    private void configureAction(Intent intent) {
        editor.setImeOptions("search".equals(intent.getStringExtra("field"))
            ? android.view.inputmethod.EditorInfo.IME_ACTION_SEARCH : android.view.inputmethod.EditorInfo.IME_ACTION_NONE);
    }

    private static int inputType(Intent intent) {
        String field = intent.getStringExtra("field");
        if ("number".equals(field)) return android.text.InputType.TYPE_CLASS_NUMBER;
        if ("decimal".equals(field)) return android.text.InputType.TYPE_CLASS_NUMBER
            | android.text.InputType.TYPE_NUMBER_FLAG_DECIMAL | android.text.InputType.TYPE_NUMBER_FLAG_SIGNED;
        if ("phone".equals(field)) return android.text.InputType.TYPE_CLASS_PHONE;
        if ("password".equals(field)) return android.text.InputType.TYPE_CLASS_TEXT
            | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD;
        if ("url".equals(field)) return android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_URI;
        if ("email".equals(field)) return android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS;
        if ("search".equals(field)) return android.text.InputType.TYPE_CLASS_TEXT;
        if ("caps".equals(field)) return android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_FLAG_CAP_SENTENCES;
        return android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_FLAG_MULTI_LINE
            | (intent.getBooleanExtra("noSuggestions", false) ? android.text.InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS : 0);
    }

    private void saveDigest(Editable text) {
        emptyEditor = text.length() == 0;
        nlpCaptureAllowed = "helo".contentEquals(text) || "the ".contentEquals(text);
        try {
            byte[] hash = MessageDigest.getInstance("SHA-256").digest(text.toString().getBytes(StandardCharsets.UTF_8));
            StringBuilder hex = new StringBuilder();
            for (byte b : hash) hex.append(String.format(java.util.Locale.ROOT, "%02x", b & 255));
            String json = "{\"utf16_units\":" + text.length() + ",\"sha256\":\"" + hex + "\",\"selection_start\":" + editor.getSelectionStart() + ",\"selection_end\":" + editor.getSelectionEnd() + ",\"composing_start\":" + android.view.inputmethod.BaseInputConnection.getComposingSpanStart(text) + ",\"composing_end\":" + android.view.inputmethod.BaseInputConnection.getComposingSpanEnd(text) + ",\"batch_depth\":" + batchDepth + ",\"editor_actions\":" + editorActions + ",\"last_editor_action\":" + lastEditorAction + "}";
            File pending = new File(getFilesDir(), "digest.json.tmp");
            try (FileOutputStream out = new FileOutputStream(pending)) {
                out.write(json.getBytes(StandardCharsets.UTF_8));
            }
            if (!pending.renameTo(new File(getFilesDir(), "digest.json"))) throw new IllegalStateException("Digest fixture publish failed");
            result.setText("UTF-16 units: " + text.length() + "\nSHA-256: " + hex);
        } catch (Exception exception) {
            throw new IllegalStateException("Could not save benchmark digest", exception);
        }
    }
}
