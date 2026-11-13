package com.jtn0123.renderprobe;

import android.accessibilityservice.AccessibilityService;
import android.accessibilityservice.AccessibilityServiceInfo;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.graphics.Rect;
import android.os.Handler;
import android.os.HandlerThread;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityWindowInfo;
import java.io.File;
import java.io.FileOutputStream;
import java.nio.charset.StandardCharsets;
import org.json.JSONArray;
import org.json.JSONObject;

/** Disposable IME-only observer. Neither editor trees nor arbitrary event text are recorded. */
public final class ProbeAccessibilityService extends AccessibilityService {
    private static final String KEYBOARD = "com.jtn0123.audiowhisper.keyboard";
    // InputConnection callbacks share the probe activity main thread. Query the IME on
    // a worker so an accessibility request cannot block those synchronous callbacks.
    private final HandlerThread worker = new HandlerThread("ime-probe");
    private Handler handler;
    private volatile long generation;
    private final BroadcastReceiver commands = new BroadcastReceiver() {
        @Override public void onReceive(Context context, Intent intent) {
            String request = intent.getStringExtra("request");
            if (request == null || !request.matches("[a-zA-Z0-9_-]+")) return;
            handler.postDelayed(() -> capture(intent, request), intent.getIntExtra("delay", 0));
        }
    };

    @Override protected void onServiceConnected() {
        worker.start();
        handler = new Handler(worker.getLooper());
        AccessibilityServiceInfo info = getServiceInfo();
        info.flags |= AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS
            | AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS
            | AccessibilityServiceInfo.FLAG_INCLUDE_NOT_IMPORTANT_VIEWS;
        setServiceInfo(info);
        registerReceiver(commands, new IntentFilter("com.jtn0123.renderprobe.IME"), Context.RECEIVER_EXPORTED);
    }
    @Override public void onDestroy() {
        handler.removeCallbacksAndMessages(null);
        unregisterReceiver(commands);
        worker.quitSafely();
        super.onDestroy();
    }
    @Override public void onInterrupt() { }
    @Override public void onAccessibilityEvent(AccessibilityEvent event) {
        if (KEYBOARD.contentEquals(event.getPackageName() == null ? "" : event.getPackageName())) generation++;
    }

    private void capture(Intent intent, String request) {
        try {
            JSONObject result = new JSONObject().put("request", request).put("generation", generation)
                .put("uptime_ms", android.os.SystemClock.uptimeMillis());
            // Android can retain a tree from the IME opening animation without new events.
            // Bypass that cache on each request, including virtual descendants.
            clearCache();
            String setting = intent.getStringExtra("setting");
            AccessibilityNodeInfo root = root(setting == null
                ? AccessibilityWindowInfo.TYPE_INPUT_METHOD : AccessibilityWindowInfo.TYPE_APPLICATION);
            if (root == null) {
                JSONArray windows = new JSONArray();
                for (AccessibilityWindowInfo window : getWindows()) {
                    AccessibilityNodeInfo candidate = window.getRoot();
                    windows.put(new JSONObject().put("type", window.getType())
                        .put("package", candidate == null ? JSONObject.NULL : candidate.getPackageName()));
                }
                result.put("error", "Requested keyboard window unavailable").put("windows", windows);
            }
            else if (setting != null) {
                // Settings UI actions use exact public labels, never return the application tree.
                AccessibilityNodeInfo node = find(root, setting);
                while (node != null && !node.isClickable()) node = node.getParent();
                result.put("found", node != null).put("rootClass", root.getClassName())
                    .put("rootChildren", root.getChildCount())
                    .put("ok", node != null && node.performAction(AccessibilityNodeInfo.ACTION_CLICK));
            } else if (!ProbeActivity.emptyEditor) {
                result.put("error", "Tree capture requires an empty disposable editor");
            } else {
                JSONArray nodes = new JSONArray();
                dump(root, -1, nodes);
                // Compose emoji popups use a separate application window owned by the IME.
                // Never inspect another package's application window or any editor contents.
                for (AccessibilityWindowInfo window : getWindows()) {
                    if (window.getType() != AccessibilityWindowInfo.TYPE_APPLICATION) continue;
                    AccessibilityNodeInfo popup = window.getRoot();
                    if (popup != null && KEYBOARD.contentEquals(popup.getPackageName())) dump(popup, -1, nodes);
                }
                result.put("nodes", nodes);
            }
            write(request + ".json", result.toString());
        } catch (Exception failure) { throw new IllegalStateException("Probe capture failed", failure); }
    }
    private AccessibilityNodeInfo root(int type) {
        for (AccessibilityWindowInfo window : getWindows()) {
            if (window.getType() != type) continue;
            AccessibilityNodeInfo root = window.getRoot();
            if (root != null && KEYBOARD.contentEquals(root.getPackageName())) return root;
        }
        return null;
    }
    private AccessibilityNodeInfo find(AccessibilityNodeInfo node, String label) {
        if ((node.getText() != null && node.getText().toString().contains(label))
            || label.contentEquals(node.getContentDescription() == null ? "" : node.getContentDescription())) return node;
        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo child = node.getChild(i);
            if (child != null) {
                AccessibilityNodeInfo match = find(child, label);
                if (match != null) return match;
            }
        }
        return null;
    }
    private void dump(AccessibilityNodeInfo node, int parent, JSONArray nodes) throws Exception {
        Rect bounds = new Rect();
        node.getBoundsInScreen(bounds);
        JSONObject item = new JSONObject();
        int index = nodes.length();
        item.put("parent", parent).put("class", nullable(node.getClassName()))
            .put("text", nullable(node.getText())).put("description", nullable(node.getContentDescription()))
            .put("viewId", nullable(node.getViewIdResourceName())).put("stateDescription", nullable(node.getStateDescription()))
            .put("bounds", new JSONArray(new int[]{bounds.left, bounds.top, bounds.right, bounds.bottom}))
            .put("checked", node.isChecked()).put("checkable", node.isCheckable())
            .put("selected", node.isSelected()).put("enabled", node.isEnabled())
            .put("focusable", node.isFocusable()).put("clickable", node.isClickable())
            .put("longClickable", node.isLongClickable()).put("visible", node.isVisibleToUser())
            .put("textEntryKey", node.isTextEntryKey());
        JSONArray actions = new JSONArray();
        for (AccessibilityNodeInfo.AccessibilityAction action : node.getActionList()) {
            actions.put(new JSONObject().put("id", action.getId()).put("label", nullable(action.getLabel())));
        }
        item.put("actions", actions);
        nodes.put(item);
        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo child = node.getChild(i);
            if (child != null) dump(child, index, nodes);
        }
    }
    private Object nullable(CharSequence value) { return value == null ? JSONObject.NULL : value.toString(); }
    private void write(String name, String data) throws Exception {
        File pending = new File(getFilesDir(), name + ".tmp");
        try (FileOutputStream out = new FileOutputStream(pending)) {
            out.write(data.getBytes(StandardCharsets.UTF_8));
        }
        if (!pending.renameTo(new File(getFilesDir(), name))) throw new IllegalStateException("Atomic publish failed");
    }
}
