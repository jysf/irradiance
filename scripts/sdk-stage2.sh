#!/bin/sh
# sdk-stage2.sh — generate Adobe dng_validate stage-2 references for the
# Pixel GainMap cross-check (SPEC-060, DEC-069 §6) and the black-normalisation
# cross-check (SPEC-078, DEC-070).
#
# Usage:
#   scripts/sdk-stage2.sh <dng_validate> <corpus_root> <out_dir>
#
# Then:
#   IRRADIANCE_SDK_STAGE2_DIR=<out_dir> just test
#
# For each held Pixel DNG it writes, into <out_dir> only:
#   <CAMERA>-post.tif  dng_validate -2 on the real file (stage 2 AFTER OpcodeList2)
#   <CAMERA>-pre.tif   dng_validate -2 on a scratch copy whose OpcodeList2 tag
#                      number (51009) is rewritten to the unused private tag
#                      51010, so the SDK skips the list (stage 2 BEFORE the gain)
#   <CAMERA>-nogain.dng  that scratch copy (about 24 MB; delete freely)
#
# and, for the held Canon EOS 5D Mark III (no opcode lists, blacks that differ
# by position):
#   CANON-EOS-5D-MARK-III-stage2.tif  dng_validate -2 on the real file
#
# Nothing here is ever committed, vendored, or run in CI. <out_dir> must be
# outside the repository.
#
# ── Building dng_validate (local reference tool, never a dependency) ─────────
# - Download dng_sdk_1_7_1.zip from Adobe (download.adobe.com, "DNG SDK 1.7.1"):
#   sha256 5366d9abd0e623858573bfaf0291b5e352d704f51b322753a22f82af3ae6e0c2
# - Compile every dng_sdk/source/*.cpp with clang++ -std=c++17 -O2 and
#     -DqMacOS=1 -DqDNGValidateTarget=1 -DqDNGUseXMP=0 -DqDNGXMPFiles=0
#     -DqDNGXMPDocOps=0 -DqDNGUseLibJPEG=0 -I/opt/homebrew/include
#   i.e. no XMP toolkit and the SDK's own JPEG code.
# - JPEG XL is stubbed out: replace dng_jxl.cpp with a file defining
#   ParseJXL (returning false), SupportsJXL (false) and the dng_jxl_decoder /
#   EncodeJXL_* entry points as ThrowNotYetImplemented. The bundled
#   dng_jxl.cpp does not compile against Homebrew's newer libjxl, and the
#   Pixel files are lossless-JPEG tiled DNGs, so no JXL path is reached.
# - Link the objects into dng_validate against Homebrew's libjxl
#   (brew install jpeg-xl) plus -lz and the CoreFoundation/CoreServices
#   frameworks.
# The SDK's licence is the Adobe DNG SDK licence agreement shipped in the zip.

set -eu

die() {
    echo "sdk-stage2: $*" >&2
    exit 1
}

[ "$#" -eq 3 ] || die "usage: $0 <dng_validate> <corpus_root> <out_dir>"
validate=$1
corpus=$2
out=$3

[ -x "$validate" ] || die "not an executable: $validate"
[ -d "$corpus" ] || die "corpus root is not a directory: $corpus"
mkdir -p "$out" || die "cannot create output directory: $out"

# The output directory must not be inside the repository: SDK bytes are never
# committed (DEC-069 §6).
repo=$(cd "$(dirname "$0")/.." && pwd -P)
out_abs=$(cd "$out" && pwd -P)
case "$out_abs/" in
    "$repo"/*) die "output directory is inside the repository ($repo): $out_abs" ;;
esac

# Rewrite the single IFD entry for tag 51009 (OpcodeList2, type UNDEFINED) to
# tag 51010. The 4-byte pattern is tag + type in the file's byte order; it must
# occur exactly once or the copy is refused (a second hit could be pixel data).
disable_opcode_list_2() {
    src=$1
    dst=$2
    perl -e '
        use strict;
        my ($src, $dst) = @ARGV;
        open(my $in, "<:raw", $src) or die "read $src: $!\n";
        local $/; my $b = <$in>; close($in);
        my $order = substr($b, 0, 2);
        my ($from, $to);
        if ($order eq "II") { ($from, $to) = ("\x41\xC7\x07\x00", "\x42\xC7\x07\x00"); }
        elsif ($order eq "MM") { ($from, $to) = ("\xC7\x41\x00\x07", "\xC7\x42\x00\x07"); }
        else { die "$src: not a TIFF byte-order mark\n"; }
        my $count = () = $b =~ /\Q$from\E/g;
        die "$src: OpcodeList2 entry pattern found $count times, expected exactly 1\n"
            unless $count == 1;
        my $at = index($b, $from);
        substr($b, $at, 4) = $to;
        open(my $out, ">:raw", $dst) or die "write $dst: $!\n";
        print $out $b; close($out) or die "close $dst: $!\n";
        print "  OpcodeList2 entry at byte $at rewritten to tag 51010\n";
    ' "$src" "$dst" || die "could not disable OpcodeList2 in $src"
}

run_stage2() {
    input=$1
    stem=$2
    rm -f "$stem.tif"
    "$validate" -2 "$stem" "$input" >"$stem.log" 2>&1 \
        || die "dng_validate failed on $input (see $stem.log)"
    [ -s "$stem.tif" ] || die "dng_validate wrote no $stem.tif (see $stem.log)"
}

count=0
for entry in \
    "GOOGLE-PIXEL-4A:GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng" \
    "GOOGLE-PIXEL-3A:GOOGLE-PIXEL-3A/IMG_20190918_164153.dng" \
    "GOOGLE-PIXEL-7-PRO:GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng"; do
    camera=${entry%%:*}
    path=$corpus/${entry#*:}
    [ -f "$path" ] || die "missing corpus file: $path"
    echo "$camera: $path"

    run_stage2 "$path" "$out_abs/$camera-post"
    disable_opcode_list_2 "$path" "$out_abs/$camera-nogain.dng"
    run_stage2 "$out_abs/$camera-nogain.dng" "$out_abs/$camera-pre"
    echo "  wrote $camera-post.tif and $camera-pre.tif"
    count=$((count + 1))
done

canon=CANON-EOS-5D-MARK-III
canon_path=$corpus/$canon/5G4A9394.DNG
[ -f "$canon_path" ] || die "missing corpus file: $canon_path"
echo "$canon: $canon_path"
run_stage2 "$canon_path" "$out_abs/$canon-stage2"
echo "  wrote $canon-stage2.tif"
count=$((count + 1))

[ "$count" -eq 4 ] || die "expected 4 files, processed $count"
echo "sdk-stage2: 4 files done; set IRRADIANCE_SDK_STAGE2_DIR=$out_abs"
