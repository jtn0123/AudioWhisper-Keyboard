#!/bin/bash
# Regenerate TextUtils' UTF-16 Ps/Pe categories with the project's JDK (24 / Unicode 16).
set -euo pipefail
cd "$(dirname "$0")/.."
output=build/editor-punctuation
mkdir -p "$output"
cat > "$output/EditorPunctuation.java" <<'JAVA'
class EditorPunctuation {
    public static void main(String[] args) {
        for (int type : new int[]{Character.START_PUNCTUATION, Character.END_PUNCTUATION}) {
            System.out.print(type == Character.START_PUNCTUATION ?
                "pub const OPEN: &[(u16,u16)] = &[" : "pub const CLOSE: &[(u16,u16)] = &[");
            for (int i = 0; i < 65536; i++) if (Character.getType((char) i) == type) {
                int start = i;
                while (i + 1 < 65536 && Character.getType((char) (i + 1)) == type) i++;
                System.out.printf("(0x%x,0x%x),", start, i);
            }
            System.out.println("]; ");
        }
    }
}
JAVA
java "$output/EditorPunctuation.java" > crates/editor/src/punctuation.rs
cargo fmt --package editor
