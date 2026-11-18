#!/usr/bin/env bash
set -euo pipefail
BIN="$(realpath "$1")"
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT
mkdir -p "$root/bin"
printf 'model' > "$root/model.bin"
cat > "$root/bin/yt-dlp" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
for arg in "$@"; do
  if [[ "$arg" == --dump-single-json ]]; then
    case "$MOCK_CAPTIONS" in
      auto) printf '{"language":"fr-FR","subtitles":{},"automatic_captions":{"fr-orig":[{"ext":"json3"}]}}\n' ;;
      manual) printf '{"language":"fr-FR","subtitles":{"fr":[{"ext":"json3"}]},"automatic_captions":{}}\n' ;;
      absent) printf '{"language":"fr-FR","subtitles":{},"automatic_captions":{}}\n' ;;
      *) exit 75 ;;
    esac
    exit 0
  fi
done
template=""
lang=""
captions=0
while (($#)); do
  case "$1" in
    --output) template="$2"; shift 2 ;;
    --sub-langs) lang="$2"; shift 2 ;;
    --write-subs|--write-auto-subs) captions=1; shift ;;
    *) shift ;;
  esac
done
[[ -n "$template" ]] || exit 74
if ((captions)); then
  file="$(printf '%s' "$template" | sed 's/%(ext)s$//')"
  printf '{"events":[{"tStartMs":0,"dDurationMs":1000,"segs":[{"utf8":"Bonjour des sous-titres"}]}]}\n' > "$file$lang.json3"
else
  file="$(printf '%s' "$template" | sed 's/%(ext)s/m4a/')"
  printf 'MEDIA' > "$file"
  printf '%s\n' "$file"
fi
MOCK
cat > "$root/bin/ffmpeg" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
printf 'WAV' > "$(printf '%s\n' "$@" | tail -1)"
MOCK
cat > "$root/bin/whisper" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
prefix=""
while (($#)); do
  case "$1" in
    --output-file) prefix="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'Fallback Whisper.\n' > "$prefix.txt"
MOCK
chmod +x "$root/bin/"*
run() { "$BIN" transcribe "$@" >"$root/stdout" 2>"$root/stderr"; }

MOCK_CAPTIONS=auto run 'https://www.youtube.com/shorts/auto' \
  --yt-dlp-bin "$root/bin/yt-dlp" --language fr --output "$root/captions.txt"
[[ "$(cat "$root/captions.txt")" == "Bonjour des sous-titres" ]] || exit 1
echo 'PASS: YouTube auto captions without model'

MOCK_CAPTIONS=manual run 'https://www.youtube.com/shorts/manual' \
  --yt-dlp-bin "$root/bin/yt-dlp" --language fr \
  --transcript-source youtube --exporter json --output "$root/manual.json"
python3 - "$root/manual.json" <<'PY'
import json, sys
j = json.load(open(sys.argv[1], encoding="utf-8"))
assert j["backend"] == "youtube-manual-captions"
assert j["text"] == "Bonjour des sous-titres"
PY
echo 'PASS: manual captions source provenance'

MOCK_CAPTIONS=absent run 'https://www.youtube.com/shorts/absent' \
  --yt-dlp-bin "$root/bin/yt-dlp" --ffmpeg-bin "$root/bin/ffmpeg" \
  --whisper-bin "$root/bin/whisper" --model "$root/model.bin" \
  --language fr --output "$root/fallback.txt"
[[ "$(cat "$root/fallback.txt")" == "Fallback Whisper." ]] || exit 1
echo 'PASS: missing captions fall back to Whisper'

if MOCK_CAPTIONS=error run 'https://www.youtube.com/shorts/error' \
  --yt-dlp-bin "$root/bin/yt-dlp" --model "$root/model.bin" \
  --output "$root/error.txt"; then
  echo 'FAIL: metadata error ignored' >&2; exit 1
fi
[[ ! -e "$root/error.txt" ]] || exit 1
echo 'PASS: metadata failure does not fall back'

if MOCK_CAPTIONS=absent run 'https://www.youtube.com/shorts/forced' \
  --yt-dlp-bin "$root/bin/yt-dlp" --transcript-source youtube \
  --output "$root/forced.txt"; then
  echo 'FAIL: forced YouTube mode ignored missing captions' >&2; exit 1
fi
echo 'PASS: forced YouTube refuses missing captions'
