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
# Licence, as found (SPEC-079): dng_sdk_1_7_1.zip (sha256 above) ships no
# licence agreement document. Its own terms are the per-file notice in each
# dng_sdk/source file, which refers to "the Adobe license agreement
# accompanying it"; the only licence files in the zip are third-party ones
# (libjpeg, the XMP toolkit, libjxl). irradiance reads the SDK's source and
# runs dng_validate locally; it copies no SDK code and redistributes nothing
# from the SDK (DEC-069 §6).

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
# The output directory must not be inside the repository: SDK bytes are never
# committed (DEC-069 §6). Its absolute path is resolved WITHOUT creating
# anything, so a refused in-repo path leaves no directory behind (SPEC-079).
#
# "The repository" is every worktree of it, found through git (SPEC-080): run
# from one worktree, the main checkout and every other worktree are refused
# too. Each path from `git worktree list --porcelain` is resolved with pwd -P.
# Without git, or outside a git checkout, the fallback is the script-relative
# root alone, and the script says so on stderr.
script_root=$(cd "$(dirname "$0")/.." && pwd -P)
worktrees=""
if command -v git >/dev/null 2>&1 \
    && listing=$(git -C "$script_root" worktree list --porcelain 2>/dev/null); then
    # A worktree whose directory is gone (prunable) has nothing to write
    # into; its recorded path is kept as it is.
    worktrees=$(printf '%s\n' "$listing" | sed -n 's/^worktree //p' \
        | while IFS= read -r wt; do
            if [ -d "$wt" ]; then (cd "$wt" && pwd -P); else printf '%s\n' "$wt"; fi
        done)
fi
if [ -z "$worktrees" ]; then
    echo "sdk-stage2: git is unavailable or this is not a git checkout; refusing only the script-relative root ($script_root)" >&2
    worktrees=$script_root
fi
if [ -d "$out" ]; then
    out_abs=$(cd "$out" && pwd -P)
elif [ -e "$out" ]; then
    die "output path exists and is not a directory: $out"
else
    parent=$(dirname "$out")
    [ -d "$parent" ] || die "the output directory's parent does not exist: $parent"
    out_abs=$(cd "$parent" && pwd -P)/$(basename "$out")
fi
# A here-document, not a pipe, so die exits the script and not a subshell.
while IFS= read -r repo; do
    [ -n "$repo" ] || continue
    case "$out_abs/" in
        "$repo"/*) die "output directory is inside the repository ($repo): $out_abs" ;;
    esac
done <<EOF
$worktrees
EOF
mkdir -p "$out_abs" || die "cannot create output directory: $out_abs"

# Rewrite the single IFD entry for tag 51009 (OpcodeList2, type UNDEFINED) to
# tag 51010, found by walking the IFDs (SPEC-079): IFD0, every NextIFD chain,
# and every IFD reachable through SubIFDs (tag 330), recursively. Every offset
# and count is bounds-checked against the file length, an IFD reached twice is
# refused (no cycle can loop), and the copy is refused unless exactly one 51009
# entry exists and its type is 7. Only that entry's two tag bytes change, so a
# copy of the tag pattern in pixel or metadata bytes is never touched.
disable_opcode_list_2() {
    src=$1
    dst=$2
    perl -e '
        use strict;
        use warnings FATAL => "all";
        my ($src, $dst) = @ARGV;
        open(my $in, "<:raw", $src) or die "read $src: $!\n";
        local $/; my $b = <$in>; close($in);
        my $len = length($b);
        die "$src: $len bytes is too short for a TIFF header\n" if $len < 8;
        my $order = substr($b, 0, 2);
        my ($u16, $u32);
        if ($order eq "II") { ($u16, $u32) = ("v", "V"); }
        elsif ($order eq "MM") { ($u16, $u32) = ("n", "N"); }
        else { die "$src: not a TIFF byte-order mark\n"; }
        my $at16 = sub { my ($o) = @_;
            die "$src: read of 2 bytes at $o is past the end ($len)\n" if $o + 2 > $len;
            return unpack($u16, substr($b, $o, 2)); };
        my $at32 = sub { my ($o) = @_;
            die "$src: read of 4 bytes at $o is past the end ($len)\n" if $o + 4 > $len;
            return unpack($u32, substr($b, $o, 4)); };
        my $magic = $at16->(2);
        die "$src: TIFF magic is $magic, expected 42\n" unless $magic == 42;

        my @queue = ($at32->(4));
        my %seen;
        my @hits;
        while (@queue) {
            my $ifd = shift @queue;
            die "$src: IFD offset $ifd is inside the header\n" if $ifd < 8;
            die "$src: IFD offset $ifd is past the end ($len)\n" if $ifd + 2 > $len;
            die "$src: IFD at $ifd is reached twice (an IFD cycle)\n" if $seen{$ifd}++;
            my $n = $at16->($ifd);
            my $end = $ifd + 2 + 12 * $n + 4;
            die "$src: IFD at $ifd has $n entries, which run past the end ($len)\n"
                if $end > $len;
            for my $k (0 .. $n - 1) {
                my $e = $ifd + 2 + 12 * $k;
                my $tag = $at16->($e);
                my $type = $at16->($e + 2);
                my $count = $at32->($e + 4);
                if ($tag == 51009) {
                    push @hits, [$ifd, $e, $type];
                } elsif ($tag == 330) {
                    # SubIFDs: LONG (4) or IFD (13), four bytes each; inline when
                    # they fit the 4-byte value field, else at its offset.
                    die "$src: SubIFDs at $e has type $type, expected 4 or 13\n"
                        unless $type == 4 || $type == 13;
                    my $base = $count <= 1 ? $e + 8 : $at32->($e + 8);
                    die "$src: SubIFDs at $e: $count offsets at $base run past the end ($len)\n"
                        if $base + 4 * $count > $len;
                    push @queue, map { $at32->($base + 4 * $_) } 0 .. $count - 1;
                }
            }
            my $next = $at32->($ifd + 2 + 12 * $n);
            push @queue, $next if $next != 0;
        }
        my $found = scalar @hits;
        die "$src: $found OpcodeList2 (51009) entries found, expected exactly 1\n"
            unless $found == 1;
        my ($ifd, $e, $type) = @{$hits[0]};
        die "$src: OpcodeList2 entry at byte $e has type $type, expected 7 (UNDEFINED)\n"
            unless $type == 7;
        substr($b, $e, 2) = pack($u16, 51010);
        open(my $out, ">:raw", $dst) or die "write $dst: $!\n";
        print $out $b; close($out) or die "close $dst: $!\n";
        print "  OpcodeList2 entry at byte $e (IFD at byte $ifd) rewritten to tag 51010\n";
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
