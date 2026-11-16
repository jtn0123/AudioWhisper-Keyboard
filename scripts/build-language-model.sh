#!/bin/bash
# Builds the keyboard's word statistics from a pinned public corpus, and scores them.
#
# A run works in its own folder, build/language-model/run-<time>/ (never committed: build/ is
# ignored; --run reuses one for later steps). The split step downloads the corpus into a new
# download/ folder there, checks each file against
# the SHA-256 pinned below and stops on any mismatch. Downloads are data only: nothing in them is
# run, and they are decompressed with a size cap.
#
# The corpus (see docs/decisions.md, "Word statistics from a public corpus"):
#   tatoeba       OPUS Tatoeba v2023-04-12, English sentences. CC BY 2.0 FR.
#   globalvoices  OPUS GlobalVoices v2018q4, English sentences. CC BY 3.0.
#
# Steps, in order (each needs the ones before it; with none given, split, dictionary, model and
# score run):
#   split       held-out split: corpus/train-*.txt and corpus/test-*.txt (lexicon prepare)
#   dictionary  the AOSP dictionary (the test fixture) rebuilt from the training sentences, to
#               dictionary/next-32.dict, and whether it is the app's dictionary byte for byte
#   model       the n-gram model over that dictionary's words, to model/model-b3-t3.lm, and
#               whether it is the app's model byte for byte
#   score       the scoreboard on the AOSP dictionary, the one built here, and that with the
#               model, to score-*.md
#   install     copies the dictionary and model built here over the app's (only when named)
#
# Usage: scripts/build-language-model.sh [--run DIR] [--limit N] [step ...]
#   --run DIR   continue in an earlier run's folder instead of a new one
#   --limit N   score the first N held-out sentences of each source (default 5000)
set -euo pipefail
cd "$(dirname "$0")/.."

LIMIT=5000
RUN=
STEPS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --limit) LIMIT="${2:?--limit needs a number}"; shift 2 ;;
    --run) RUN="${2:?--run needs a folder}"; shift 2 ;;
    split|dictionary|model|score|install) STEPS+=("$1"); shift ;;
    *) echo "unknown step or option: $1" >&2; exit 2 ;;
  esac
done
[ ${#STEPS[@]} -gt 0 ] || STEPS=(split dictionary model score)
wants() { local step; for step in "${STEPS[@]}"; do [ "$step" = "$1" ] && return 0; done; return 1; }

NAMES=(tatoeba globalvoices)
URLS=(
  https://object.pouta.csc.fi/OPUS-Tatoeba/v2023-04-12/mono/en.txt.gz
  https://object.pouta.csc.fi/OPUS-GlobalVoices/v2018q4/mono/en.txt.gz
)
SHA256=(
  a32c5500cd76b9479859764fb78537a4b9b53fab8fa3bdc0fc04dd70f28bf29b
  79c2a1522c64a345d7def0b36c3c044daee81d819af6f59211b5bb9f068458d5
)
# The dictionary build: AOSP's dictionary supplies the words and flags (see docs/decisions.md).
AOSP_DICT=lib/rust/crates/latin/tests/fixtures/aosp_main_en-US.dict
APP_DICT=app/src/main/assets/dicts/main_en-US.dict
NEXT_WORDS=32
MIN_PAIR=2
# The model keeps the pairs and triples the training sentences write at least this often.
MIN_BIGRAM=3
MIN_TRIGRAM=3
APP_MODEL=app/src/main/assets/dicts/main_en-US.lm
# The header's date, pinned so a rebuild writes the same bytes: 2026-11-16 00:00 UTC.
DICT_DATE=1794787200
# Neither file is near these; they stop a wrong or hostile file filling the disk.
MAX_DOWNLOAD=100000000
MAX_TEXT=1000000000

if [ -z "$RUN" ]; then
  mkdir -p build/language-model
  RUN="build/language-model/run-$(date +%Y%m%d-%H%M%S)"
  mkdir "$RUN"
elif [ ! -d "$RUN" ]; then
  echo "no run folder $RUN" >&2
  exit 2
fi
echo "Run folder: $RUN"
RUN="$(cd "$RUN" && pwd)"
# From lib/rust, so cargo uses its config (the build/cargo target folder).
cargo_run() { (cd lib/rust && cargo run --quiet --release --locked "$@"); }

if wants split; then
  mkdir "$RUN/download"
  SOURCES=()
  for i in "${!NAMES[@]}"; do
    name="${NAMES[$i]}"
    gz="$RUN/download/$name-en.txt.gz"
    echo "Downloading $name"
    curl -fsSL --proto '=https' --tlsv1.2 --max-filesize "$MAX_DOWNLOAD" -o "$gz" "${URLS[$i]}"
    actual=$(shasum -a 256 "$gz" | cut -d' ' -f1)
    if [ "$actual" != "${SHA256[$i]}" ]; then
      echo "$name: SHA-256 is $actual, expected ${SHA256[$i]}: stopping" >&2
      exit 1
    fi
    gunzip -c "$gz" | head -c "$MAX_TEXT" > "$RUN/download/$name-en.txt"
    SOURCES+=("$name=$RUN/download/$name-en.txt")
  done
  cargo_run -p lexicon -- prepare "$RUN/corpus" "${SOURCES[@]}"
fi

BUILT="$RUN/dictionary/next-$NEXT_WORDS.dict"

if wants dictionary; then
  [ -d "$RUN/corpus" ] || { echo "$RUN has no corpus: run the split step first" >&2; exit 2; }
  TRAIN=()
  for name in "${NAMES[@]}"; do TRAIN+=("$RUN/corpus/train-$name.txt"); done
  cargo_run -p lexicon -- dictionary "$PWD/$AOSP_DICT" "$RUN/dictionary" "${TRAIN[@]}" \
    --next "$NEXT_WORDS" --min-pair "$MIN_PAIR" --date "$DICT_DATE"
  echo "SHA-256 $(shasum -a 256 "$BUILT" | cut -d' ' -f1)"
  if cmp -s "$BUILT" "$APP_DICT"; then
    echo "The app's dictionary is this build, byte for byte."
  else
    echo "The app's dictionary differs from this build (the install step copies it)."
  fi
fi

MODEL="$RUN/model/model-b$MIN_BIGRAM-t$MIN_TRIGRAM.lm"

if wants model; then
  [ -f "$BUILT" ] || { echo "$RUN has no dictionary: run the dictionary step first" >&2; exit 2; }
  TRAIN=()
  for name in "${NAMES[@]}"; do TRAIN+=("$RUN/corpus/train-$name.txt"); done
  cargo_run -p lexicon -- model "$BUILT" "$RUN/model" "${TRAIN[@]}" \
    --min-bigram "$MIN_BIGRAM" --min-trigram "$MIN_TRIGRAM"
  echo "SHA-256 $(shasum -a 256 "$MODEL" | cut -d' ' -f1)"
  if cmp -s "$MODEL" "$APP_MODEL"; then
    echo "The app's model is this build, byte for byte."
  else
    echo "The app's model differs from this build (the install step copies it)."
  fi
fi

if wants score; then
  [ -d "$RUN/corpus" ] || { echo "$RUN has no corpus: run the split step first" >&2; exit 2; }
  TESTS=()
  for name in "${NAMES[@]}"; do TESTS+=(--test "$name=$RUN/corpus/test-$name.txt"); done
  echo "Scoring the AOSP dictionary"
  cargo_run -p scoreboard -- --dict "$PWD/$AOSP_DICT" --limit "$LIMIT" "${TESTS[@]}" \
    | tee "$RUN/score-aosp.md"
  if [ -f "$BUILT" ]; then
    echo "Scoring the rebuilt dictionary"
    cargo_run -p scoreboard -- --dict "$BUILT" --limit "$LIMIT" "${TESTS[@]}" \
      | tee "$RUN/score-rebuilt.md"
  fi
  if [ -f "$BUILT" ] && [ -f "$MODEL" ]; then
    echo "Scoring the rebuilt dictionary with the model"
    cargo_run -p scoreboard -- --dict "$BUILT" --model "$MODEL" --limit "$LIMIT" "${TESTS[@]}" \
      | tee "$RUN/score-model.md"
  fi
fi

if wants install; then
  [ -f "$BUILT" ] || { echo "$RUN has no dictionary: run the dictionary step first" >&2; exit 2; }
  [ -f "$MODEL" ] || { echo "$RUN has no model: run the model step first" >&2; exit 2; }
  cp "$BUILT" "$APP_DICT"
  echo "Copied $BUILT to $APP_DICT"
  cp "$MODEL" "$APP_MODEL"
  echo "Copied $MODEL to $APP_MODEL"
fi
