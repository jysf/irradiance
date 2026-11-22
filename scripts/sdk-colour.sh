#!/bin/sh
# sdk-colour.sh — generate Adobe DNG SDK camera->XYZ(D50) matrices for the
# colour cross-check (SPEC-085, DEC-076; the lane is tests/colour_sdk.rs).
#
# Usage:
#   scripts/sdk-colour.sh <sdk_build_dir> <sdk_source_dir> <corpus_root> <out_dir>
#
#   <sdk_build_dir>   holds obj/ (every SDK object, dng_validate.o included) and
#                     jxl_stubs.o, as built for scripts/sdk-stage2.sh
#   <sdk_source_dir>  dng_sdk_1_7_1/dng_sdk/source (the SDK headers)
#
# Then:
#   IRRADIANCE_SDK_COLOUR_DIR=<out_dir> just test
#
# It compiles scripts/sdk-camera-to-pcs.cpp (this repo's own harness) against
# the SDK headers, links it with the already-built SDK objects minus
# dng_validate.o (which carries its own main), and writes into <out_dir> only:
#   sdk-camera-to-pcs              the harness binary
#   <CAMERA>-camera-to-pcs.txt     its output for each of the five held colour
#                                  files: white_xy, neutral, three cam2pcs rows
#
# Nothing here is ever committed, vendored, or run in CI, and <out_dir> must be
# outside the repository (DEC-069 §6). Building the SDK objects is described in
# scripts/sdk-stage2.sh.

set -eu

die() {
    echo "sdk-colour: $*" >&2
    exit 1
}

[ "$#" -eq 4 ] || die "usage: $0 <sdk_build_dir> <sdk_source_dir> <corpus_root> <out_dir>"
build=$1
headers=$2
corpus=$3
out=$4

[ -d "$build/obj" ] || die "no obj/ directory in the SDK build dir: $build"
[ -f "$build/jxl_stubs.o" ] || die "no jxl_stubs.o in the SDK build dir: $build"
[ -f "$headers/dng_color_spec.h" ] || die "not the SDK source directory (no dng_color_spec.h): $headers"
[ -d "$corpus" ] || die "corpus root is not a directory: $corpus"

# The output directory must not be inside the repository, nor inside any of
# its worktrees: the same guard as scripts/sdk-stage2.sh (SPEC-079, SPEC-080,
# SPEC-083), resolved WITHOUT creating anything first.
script_root=$(cd "$(dirname "$0")/.." && pwd -P)
harness=$script_root/scripts/sdk-camera-to-pcs.cpp
[ -f "$harness" ] || die "missing harness source: $harness"
worktrees=""
if command -v git >/dev/null 2>&1 \
    && listing=$(git -C "$script_root" worktree list --porcelain 2>/dev/null); then
    worktrees=$(printf '%s\n' "$listing" | sed -n 's/^worktree //p' \
        | while IFS= read -r wt; do
            if [ -d "$wt" ]; then
                (cd "$wt" 2>/dev/null && pwd -P) || {
                    echo "sdk-colour: cannot enter worktree $wt; keeping its recorded path as a refused location" >&2
                    printf '%s\n' "$wt"
                }
            else
                printf '%s\n' "$wt"
            fi
        done)
fi
if [ -z "$worktrees" ]; then
    echo "sdk-colour: git is unavailable or this is not a git checkout; refusing only the script-relative root ($script_root)" >&2
    worktrees=$script_root
fi
if [ -d "$out" ]; then
    out_abs=$(cd "$out" 2>/dev/null && pwd -P) || die "cannot enter the output directory: $out"
elif [ -e "$out" ]; then
    die "output path exists and is not a directory: $out"
else
    parent=$(dirname "$out")
    [ -d "$parent" ] || die "the output directory's parent does not exist: $parent"
    parent_abs=$(cd "$parent" 2>/dev/null && pwd -P) || die "cannot enter the output directory's parent: $parent"
    out_abs=$parent_abs/$(basename "$out")
fi
while IFS= read -r repo; do
    [ -n "$repo" ] || continue
    case "$out_abs/" in
        "$repo"/*) die "output directory is inside the repository ($repo): $out_abs" ;;
    esac
done <<END
$worktrees
END
mkdir -p "$out_abs" || die "cannot create output directory: $out_abs"

# Compile and link the harness. The flags are the SDK build's own
# (no XMP toolkit, the SDK's own JPEG code, Homebrew's libjxl).
objects=""
object_count=0
for o in "$build"/obj/*.o; do
    [ -f "$o" ] || continue
    case "$(basename "$o")" in
        dng_validate.o) continue ;;
    esac
    objects="$objects $o"
    object_count=$((object_count + 1))
done
[ "$object_count" -gt 0 ] || die "no SDK objects in $build/obj"
bin=$out_abs/sdk-camera-to-pcs
# shellcheck disable=SC2086 # $objects is a deliberate word list of paths without spaces
clang++ -std=c++17 -O2 -w -DqMacOS=1 -DqDNGValidateTarget=1 -DqDNGUseXMP=0 \
    -DqDNGXMPFiles=0 -DqDNGXMPDocOps=0 -DqDNGUseLibJPEG=0 \
    -I"$headers" -I/opt/homebrew/include \
    "$harness" $objects "$build/jxl_stubs.o" \
    -L/opt/homebrew/lib -ljxl -ljxl_threads -lz \
    -framework CoreFoundation -framework CoreServices \
    -o "$bin" || die "could not build the harness"
echo "built $bin from $object_count SDK objects"

count=0
for entry in \
    "GOOGLE-PIXEL-7-PRO:GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng" \
    "GOOGLE-PIXEL-3A:GOOGLE-PIXEL-3A/IMG_20190918_164153.dng" \
    "GOOGLE-PIXEL-4A:GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng" \
    "CANON-EOS-5D-MARK-III:CANON-EOS-5D-MARK-III/5G4A9394.DNG" \
    "NIKON-D300S:NIKON-D300S/_DSC4615.DNG"; do
    camera=${entry%%:*}
    path=$corpus/${entry#*:}
    [ -f "$path" ] || die "missing corpus file: $path"
    txt=$out_abs/$camera-camera-to-pcs.txt
    "$bin" "$path" >"$txt" || die "the harness failed on $path"
    lines=$(wc -l <"$txt" | tr -d ' ')
    [ "$lines" -eq 5 ] || die "$txt has $lines lines, expected 5"
    echo "$camera: wrote $camera-camera-to-pcs.txt"
    count=$((count + 1))
done

[ "$count" -eq 5 ] || die "expected 5 files, processed $count"
echo "sdk-colour: 5 files done; set IRRADIANCE_SDK_COLOUR_DIR=$out_abs"
