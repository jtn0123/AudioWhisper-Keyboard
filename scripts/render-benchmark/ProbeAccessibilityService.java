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
import java.util.ArrayList;
import java.util.List;
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
            if ("rolling".equals(intent.getStringExtra("gesture"))) {
                android.graphics.Path first = new android.graphics.Path();
                first.moveTo(110, 1900);
                android.graphics.Path second = new android.graphics.Path();
                second.moveTo(215, 1900);
                android.accessibilityservice.GestureDescription gesture = new android.accessibilityservice.GestureDescription.Builder()
                    .addStroke(new android.accessibilityservice.GestureDescription.StrokeDescription(first, 0, 160))
                    .addStroke(new android.accessibilityservice.GestureDescription.StrokeDescription(second, 80, 160)).build();
                dispatchGesture(gesture, new GestureResultCallback() {
                    @Override public void onCompleted(android.accessibilityservice.GestureDescription description) {reply(true);}
                    @Override public void onCancelled(android.accessibilityservice.GestureDescription description) {reply(false);}
                    private void reply(boolean ok) {
                        try {write(request + ".json", new JSONObject().put("request",request).put("ok",ok).toString());}
                        catch (Exception failure) {throw new IllegalStateException(failure);}
                    }
                }, null);
            } else handler.postDelayed(() -> capture(intent, request), intent.getIntExtra("delay", 0));
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
            boolean redact = intent.getBooleanExtra("redact", false);
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
                boolean geometryOnly = intent.getBooleanExtra("geometryOnly", false);
                if (node != null && geometryOnly) {
                    Rect bounds = new Rect(); node.getBoundsInScreen(bounds);
                    result.put("bounds", new JSONArray(new int[]{bounds.left, bounds.top, bounds.right, bounds.bottom}));
                }
                result.put("found", node != null).put("rootClass", root.getClassName())
                    .put("rootChildren", root.getChildCount())
                    .put("ok", node != null && (geometryOnly || node.performAction(AccessibilityNodeInfo.ACTION_CLICK)));
            } else if (intent.hasExtra("panelTarget")) {
                // Fixed action geometry only, even during the disposable editing fixture.
                // Never dump a tree, candidate label or application/editor node in this mode.
                String label = intent.getStringExtra("panelTarget");
                if (!java.util.Arrays.asList("Arrow left", "Arrow right", "Select all", "Copy", "Paste").contains(label)) {
                    result.put("error", "Unknown panel target");
                } else {
                    Rect bounds = panelTarget(root, label);
                    result.put("buttonBounds", bounds == null ? JSONObject.NULL :
                        new JSONArray(new int[]{bounds.left, bounds.top, bounds.right, bounds.bottom}));
                }
            } else if (intent.getBooleanExtra("nlpStrip", false) && ProbeActivity.nlpCaptureAllowed) {
                JSONArray nodes = new JSONArray();
                dumpStrip(root, nodes);
                result.put("strip", nodes);
            } else if (!ProbeActivity.emptyEditor && !redact) {
                // A redacted tree writes no text at all, so it may follow typing; others may not.
                result.put("error", "Tree capture requires an empty disposable editor");
            } else {
                JSONArray nodes = new JSONArray();
                List<AccessibilityNodeInfo> infos = new ArrayList<>();
                dump(root, -1, nodes, infos, redact);
                // Compose emoji popups use a separate application window owned by the IME.
                // Never inspect another package's application window or any editor contents.
                for (AccessibilityWindowInfo window : getWindows()) {
                    if (window.getType() != AccessibilityWindowInfo.TYPE_APPLICATION) continue;
                    AccessibilityNodeInfo popup = window.getRoot();
                    if (popup != null && KEYBOARD.contentEquals(popup.getPackageName())) dump(popup, -1, nodes, infos, redact);
                }
                traversal(nodes, infos);
                result.put("nodes", nodes);
            }
            write(request + ".json", result.toString());
        } catch (Exception failure) { throw new IllegalStateException("Probe capture failed", failure); }
    }
    private Rect panelTarget(AccessibilityNodeInfo node, String label) {
        Rect bounds = new Rect(); node.getBoundsInScreen(bounds);
        // Native virtual tiles retain their full bounds while partly clipped by the grid.
        // Tap inside their visible body intersection, below the fixed smartbar header.
        if (node.isVisibleToUser() && bounds.intersect(0, 1675, 1080, 2274) &&
            (label.contentEquals(node.getText() == null ? "" : node.getText()) ||
             label.contentEquals(node.getContentDescription() == null ? "" : node.getContentDescription()))) return bounds;
        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo child = node.getChild(i);
            if (child != null) {
                Rect found = panelTarget(child, label);
                if (found != null) return found;
            }
        }
        return null;
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
    private void dump(AccessibilityNodeInfo node, int parent, JSONArray nodes, List<AccessibilityNodeInfo> infos,
                      boolean redact) throws Exception {
        Rect bounds = new Rect();
        node.getBoundsInScreen(bounds);
        JSONObject item = new JSONObject();
        int index = nodes.length();
        infos.add(node);
        item.put("parent", parent).put("class", nullable(node.getClassName()))
            .put("text", redact ? JSONObject.NULL : nullable(node.getText()))
            .put("description", redact ? JSONObject.NULL : nullable(node.getContentDescription()))
            .put("viewId", redact ? JSONObject.NULL : nullable(node.getViewIdResourceName()))
            .put("stateDescription", redact ? JSONObject.NULL : nullable(node.getStateDescription()))
            .put("bounds", new JSONArray(new int[]{bounds.left, bounds.top, bounds.right, bounds.bottom}))
            .put("checked", node.isChecked()).put("checkable", node.isCheckable())
            .put("selected", node.isSelected()).put("enabled", node.isEnabled())
            .put("focusable", node.isFocusable()).put("clickable", node.isClickable())
            .put("longClickable", node.isLongClickable()).put("visible", node.isVisibleToUser())
            .put("textEntryKey", node.isTextEntryKey());
        JSONArray actions = new JSONArray();
        for (AccessibilityNodeInfo.AccessibilityAction action : node.getActionList()) {
            actions.put(new JSONObject().put("id", action.getId()).put("label", redact ? JSONObject.NULL : nullable(action.getLabel())));
        }
        item.put("actions", actions);
        // Fixture history is targeted by digest; never write clipboard payloads to a dump.
        for (String field : new String[]{"text", "description"}) {
            String label = item.optString(field, "");
            if (label.contains("aw-test-")) {
                byte[] digest = java.security.MessageDigest.getInstance("SHA-256")
                    .digest(label.getBytes(StandardCharsets.UTF_8));
                StringBuilder hash = new StringBuilder();
                for (byte value : digest) hash.append(String.format(java.util.Locale.ROOT, "%02x", value & 255));
                item.put(field + "Sha256", hash.toString()).put(field, JSONObject.NULL);
            }
        }
        nodes.put(item);
        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo child = node.getChild(i);
            if (child != null) dump(child, index, nodes, infos, redact);
        }
    }
    /** Each node's traversal hints as dump indices: -1 for none, -2 for a node outside the dump. */
    private static void traversal(JSONArray nodes, List<AccessibilityNodeInfo> infos) throws Exception {
        for (int i = 0; i < infos.size(); i++) {
            nodes.getJSONObject(i).put("traversalAfter", index(infos, infos.get(i).getTraversalAfter()))
                .put("traversalBefore", index(infos, infos.get(i).getTraversalBefore()));
        }
    }
    private static int index(List<AccessibilityNodeInfo> infos, AccessibilityNodeInfo node) {
        if (node == null) return -1;
        int found = infos.indexOf(node);
        return found < 0 ? -2 : found;
    }
    /** Hash only IME candidate labels from two allowlisted public fixtures, never editor nodes. */
    private void dumpStrip(AccessibilityNodeInfo node, JSONArray nodes) throws Exception {
        Rect bounds = new Rect(); node.getBoundsInScreen(bounds);
        CharSequence label = node.getText();
        if (label == null || label.length() == 0) label = node.getContentDescription();
        if (label != null && label.length() > 0 && bounds.centerY() >= 1540 && bounds.centerY() <= 1690
            && bounds.centerX() >= 240 && bounds.centerX() <= 800 && node.isVisibleToUser()) {
            AccessibilityNodeInfo target = node;
            while (target != null && !target.isClickable()) target = target.getParent();
            Rect actionBounds = new Rect();
            if (target != null) target.getBoundsInScreen(actionBounds);
            byte[] digest = java.security.MessageDigest.getInstance("SHA-256").digest(label.toString().getBytes(StandardCharsets.UTF_8));
            StringBuilder hash = new StringBuilder();
            for (byte value : digest) hash.append(String.format(java.util.Locale.ROOT, "%02x", value & 255));
            boolean clickAction = target != null && target.getActionList().stream().anyMatch(action -> action.getId() == AccessibilityNodeInfo.ACTION_CLICK);
            nodes.put(new JSONObject().put("labelSha256", hash.toString()).put("enabled", node.isEnabled())
                .put("clickable", target != null).put("clickAction", clickAction)
                .put("labelBounds", new JSONArray(new int[]{bounds.left, bounds.top, bounds.right, bounds.bottom}))
                .put("bounds", new JSONArray(new int[]{actionBounds.left, actionBounds.top, actionBounds.right, actionBounds.bottom})));
        }
        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo child = node.getChild(i);
            if (child != null) dumpStrip(child, nodes);
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
