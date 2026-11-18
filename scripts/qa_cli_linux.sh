#!/usr/bin/env bash
# Real Rust CLI + fake executables; Linux/WSL only.
# This does not demonstrate actual FFmpeg/Whisper inference quality.
set -euo pipefail
if (( $# != 1 )); then
  echo "Usage: bash scripts/qa_cli_linux.sh /absolute/path/to/gotoo-vid2txt" >&2
  exit 2
fi
BIN="$(realpath "$1")"
[[ -x "$BIN" ]] || { echo "Missing executable: $BIN" >&2; exit 2; }
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT
mkdir -p "$root/bin" "$root/temp"
printf 'video source\n' > "$root/video.mp4"
printf 'model source\n' > "$root/model.bin"

cat > "$root/bin/ffmpeg-mock" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
[[ "${MOCK_FAIL_FFMPEG:-0}" != 1 ]] || exit 71
out="${@: -1}"
if [[ -n "${MOCK_TEMP_PERM:-}" ]]; then
  stat -c '%a' "$(dirname "$out")" > "$MOCK_TEMP_PERM"
fi
printf 'WAV test\n' > "$out"
MOCK
cat > "$root/bin/whisper-mock" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
[[ "${MOCK_FAIL_WHISPER:-0}" != 1 ]] || exit 72
if [[ "${1:-}" == --help ]]; then echo 'whisper-mock help'; exit 0; fi
prefix=""
while (($#)); do
  case "$1" in
    --output-file) prefix="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[[ -n "$prefix" ]] || exit 73
printf 'Bonjour de la simulation.\n' > "${prefix}.txt"
MOCK
cat > "$root/bin/yt-dlp-mock" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "$MOCK_YT_ARGS"
template=""
while (($#)); do
  case "$1" in
    --output) template="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[[ -n "$template" ]] || exit 74
file="${template//\%\(ext\)s/m4a}"
printf 'URL source\n' > "$file"
printf '%s\n' "$file"
MOCK
chmod +x "$root/bin/"*
export TMPDIR="$root/temp"
export MOCK_YT_ARGS="$root/yt-args.log"
common=(--model "$root/model.bin" --ffmpeg-bin "$root/bin/ffmpeg-mock" --whisper-bin "$root/bin/whisper-mock" --yt-dlp-bin "$root/bin/yt-dlp-mock")
run() { "$BIN" transcribe "$@" > "$root/stdout.log" 2> "$root/stderr.log"; }
assert_equals() {
  local actual
  actual="$(cat "$1")" || { echo "FAIL: $3 (missing output file: $1)" >&2; exit 1; }
  if [[ "$actual" != "$2" ]]; then
    printf 'FAIL: %s (expected %q, got %q)\n' "$3" "$2" "$actual" >&2
    exit 1
  fi
  echo "PASS: $3"
}
run "$root/video.mp4" "${common[@]}" --output "$root/local.txt" --audio-out "$root/audio.wav"
assert_equals "$root/local.txt" 'Bonjour de la simulation.' 'local transcription'
assert_equals "$root/audio.wav" 'WAV test' 'local WAV'

run 'https://www.youtube.com/shorts/QoPPlTBY4ls' "${common[@]}" --transcript-source whisper --exporter json --output "$root/remote.json"
python3 - "$root/remote.json" <<'PY'
import json, sys
value = json.load(open(sys.argv[1], encoding="utf-8"))
assert value["schema_version"] == 1
assert value["text"].strip() == "Bonjour de la simulation."
assert value["backend"] == "whisper-cpp-cli"
PY
printf 'PASS: remote JSON contract\n'
grep -Fxq -- '--ignore-config' "$root/yt-args.log" || { echo 'FAIL: yt-dlp config isolation' >&2; exit 1; }
echo 'PASS: yt-dlp config isolation'

if run "$root/video.mp4" "${common[@]}" --output "$root/local.txt"; then
  echo 'FAIL: implicit overwrite' >&2; exit 1
fi
assert_equals "$root/local.txt" 'Bonjour de la simulation.' 'no implicit overwrite'

printf 'SENTINEL\n' > "$root/sentinel.txt"
ln -s "$root/sentinel.txt" "$root/symlink.txt"
if run "$root/video.mp4" "${common[@]}" --output "$root/symlink.txt" --force; then
  echo 'FAIL: symlink accepted' >&2; exit 1
fi
assert_equals "$root/sentinel.txt" SENTINEL 'symlink preserved'

printf 'HARDLINK\n' > "$root/hard-source.mp4"
ln "$root/hard-source.mp4" "$root/hard-output.txt"
run "$root/hard-source.mp4" "${common[@]}" --output "$root/hard-output.txt" --force
assert_equals "$root/hard-source.mp4" HARDLINK 'hardlink source preserved'

printf 'OLD\n' > "$root/old.txt"
if MOCK_FAIL_FFMPEG=1 run "$root/video.mp4" "${common[@]}" --output "$root/old.txt" --force; then
  echo 'FAIL: injected FFmpeg error ignored' >&2; exit 1
fi
assert_equals "$root/old.txt" OLD 'FFmpeg failure preserves output'
if MOCK_FAIL_WHISPER=1 run "$root/video.mp4" "${common[@]}" --output "$root/old.txt" --force; then
  echo 'FAIL: injected Whisper error ignored' >&2; exit 1
fi
assert_equals "$root/old.txt" OLD 'Whisper failure preserves output'

MOCK_TEMP_PERM="$root/workspace-perms.txt" run "$root/video.mp4" "${common[@]}" --output "$root/permissions.txt"
assert_equals "$root/workspace-perms.txt" 700 'temporary workspace private'
[[ -z "$(find "$root/temp" -mindepth 1 -maxdepth 1 -print -quit)" ]] || { echo 'FAIL: tempfile cleanup' >&2; exit 1; }
echo 'PASS: temporary workspace cleanup'

# Config selection: native WAV path, without ffmpeg or yt-dlp present.
cat > "$root/pipeline.toml" <<TOML
[profile.default]
source = "local"
audio = "wav_pcm16"
transcriber = "whisper_cpp"
exporter = "json"
[backends.whisper_cpp]
model = "$root/model.bin"
binary = "$root/bin/whisper-mock"
[backends.ffmpeg]
binary = "/not/installed/ffmpeg"
[backends.yt_dlp]
binary = "/not/installed/yt-dlp"
TOML

python3 - "$root/fixture.wav" <<'PYWAV'
import struct, sys
pcm = b"\x00\x00" * 160
length = len(pcm)
with open(sys.argv[1], "wb") as f:
    f.write(b"RIFF" + struct.pack("<I", 36+length) + b"WAVE")
    f.write(b"fmt " + struct.pack("<IHHIIHH", 16, 1, 1, 16000, 32000, 2, 16))
    f.write(b"data" + struct.pack("<I", length) + pcm)
PYWAV

run "$root/fixture.wav" --config "$root/pipeline.toml" --output "$root/native.json"
python3 - "$root/native.json" <<'PYJSON'
import json, sys
value = json.load(open(sys.argv[1], encoding="utf-8"))
assert value["text"].strip() == "Bonjour de la simulation."
PYJSON
echo 'PASS: TOML-selected native WAV + JSON pipeline, no ffmpeg'

run "$root/fixture.wav" --config "$root/pipeline.toml" --exporter txt --output "$root/override.txt"
assert_equals "$root/override.txt" 'Bonjour de la simulation.' 'CLI override exporter TXT'

"$BIN" transcribe "$root/fixture.wav" --config "$root/pipeline.toml" \
  --output "$root/no-created.txt" --dry-run > "$root/dry.log"
[[ ! -e "$root/no-created.txt" ]] || { echo 'FAIL: dry-run created output' >&2; exit 1; }
grep -Eq 'WavPcm16|wav-pcm16' "$root/dry.log" || { echo 'FAIL: dry-run wrong audio backend' >&2; exit 1; }
echo 'PASS: dry-run plan without processing'

if run "$root/fixture.wav" --config "$root/pipeline.toml" --source yt-dlp --output "$root/bad.txt"; then
  echo 'FAIL: mismatch source backend accepted' >&2; exit 1
fi
[[ ! -e "$root/bad.txt" ]] || { echo 'FAIL: mismatch created output' >&2; exit 1; }
echo 'PASS: explicit source mismatch rejected'

printf 'invalid' > "$root/bad.wav"
if run "$root/bad.wav" --config "$root/pipeline.toml" --output "$root/bad-wav.txt"; then
  echo 'FAIL: non-WAV accepted by strict native decoder' >&2; exit 1
fi
[[ ! -e "$root/bad-wav.txt" ]] || { echo 'FAIL: bad audio created output' >&2; exit 1; }
echo 'PASS: unsupported WAV rejected without fallback'

printf '[profile.default]\naudo = "ffmpeg"\n' > "$root/bad.toml"
if "$BIN" transcribe "$root/fixture.wav" --config "$root/bad.toml" --dry-run \
  >"$root/stdout.log" 2>"$root/stderr.log"; then
  echo 'FAIL: typo in config accepted' >&2; exit 1
fi
echo 'PASS: invalid config rejected'

"$BIN" doctor --config "$root/pipeline.toml" --input "$root/fixture.wav" >"$root/doctor.log"
echo 'PASS: doctor checks only selected requirements'
echo 'All mocked CLI checks PASS; real tools and performance remain untested.'
