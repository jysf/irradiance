#!/bin/sh
# sdk-stage3.sh — generate Adobe dng_validate stage-3 references for the
# per-plane WarpRectilinear cross-check at sensor geometry (SPEC-099, DEC-087).
#
# Usage:
#   scripts/sdk-stage3.sh <dng_validate> <corpus_root> <out_dir>
#
# Then:
#   IRRADIANCE_SDK_STAGE3_DIR=<out_dir> just test
#
# For each held Leica colour file with a three-plane WarpRectilinear (the Q2,
# Q3 and SL3) it writes, into <out_dir> only:
#   <tag>-full.tif    dng_validate -3 on the real file: camera RGB over the
#                     ActiveArea after OpcodeList3's per-plane warp
#   <tag>-green.dng   a scratch copy whose first WarpRectilinear has its red
#                     and blue coefficient sets (planes 0 and 2) overwritten by
#                     green's (plane 1), so the SDK warps every plane alike
#                     (several hundred MB in all; delete freely)
#   <tag>-green.tif   dng_validate -3 on that copy
# with <tag> one of q2, q3, sl3. Each stage-3 TIFF is 280 to 360 MB.
#
# Nothing here is ever committed, vendored, or run in CI, and the Q3 and SL3
# (LEICA-Q3-PB, LEICA-SL3-PB) have no stated licence: neither they nor anything
# derived from them may enter the repository. <out_dir> must be outside it.
#
# Building dng_validate: see scripts/sdk-stage2.sh's header (the same local
# reference tool, never a dependency).

set -eu

die() {
    echo "sdk-stage3: $*" >&2
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
    # into; its recorded path is still refused. So is one whose directory
    # exists but cannot be entered (SPEC-083): under set -e a failed cd here
    # used to end the script with no message of its own.
    worktrees=$(printf '%s\n' "$listing" | sed -n 's/^worktree //p' \
        | while IFS= read -r wt; do
            if [ -d "$wt" ]; then
                (cd "$wt" 2>/dev/null && pwd -P) || {
                    echo "sdk-stage3: cannot enter worktree $wt; keeping its recorded path as a refused location" >&2
                    printf '%s\n' "$wt"
                }
            else
                printf '%s\n' "$wt"
            fi
        done)
fi
if [ -z "$worktrees" ]; then
    echo "sdk-stage3: git is unavailable or this is not a git checkout; refusing only the script-relative root ($script_root)" >&2
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

# Overwrite planes 0 and 2 of the first WarpRectilinear (opcode ID 1) in the
# single OpcodeList3 entry (tag 51022, type UNDEFINED) with plane 1's 48 bytes,
# found by walking the IFDs as scripts/sdk-stage2.sh does: IFD0, every NextIFD
# chain, and every IFD reachable through SubIFDs (tag 330). Every offset and
# count is bounds-checked, an IFD reached twice is refused, and the copy is
# refused unless exactly one 51022 entry of type 7 exists and its first
# WarpRectilinear has N = 3. Only those 96 coefficient bytes change.
green_only_warp() {
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
        # Opcode lists are big-endian whatever the file order.
        my $be32 = sub { my ($o) = @_;
            die "$src: read of 4 bytes at $o is past the end ($len)\n" if $o + 4 > $len;
            return unpack("N", substr($b, $o, 4)); };
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
                if ($tag == 51022) {
                    push @hits, [$e, $type, $count, $count <= 4 ? $e + 8 : $at32->($e + 8)];
                } elsif ($tag == 330) {
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
        die "$src: $found OpcodeList3 (51022) entries found, expected exactly 1\n"
            unless $found == 1;
        my ($e, $type, $count, $list) = @{$hits[0]};
        die "$src: OpcodeList3 entry at byte $e has type $type, expected 7 (UNDEFINED)\n"
            unless $type == 7;
        die "$src: OpcodeList3 ($count bytes at $list) runs past the end ($len)\n"
            if $list + $count > $len;
        my $ops = $be32->($list);
        my $o = $list + 4;
        my $patched = 0;
        for my $i (1 .. $ops) {
            die "$src: opcode $i header runs past the list\n" if $o + 16 > $list + $count;
            my ($id, $size) = ($be32->($o), $be32->($o + 12));
            my $p = $o + 16;
            die "$src: opcode $i parameters run past the list\n" if $p + $size > $list + $count;
            if ($id == 1) {
                my $planes = $be32->($p);
                die "$src: the first WarpRectilinear has N = $planes, expected 3\n"
                    unless $planes == 3;
                die "$src: WarpRectilinear size $size, expected 164\n" unless $size == 164;
                my $green = substr($b, $p + 4 + 48, 48);
                substr($b, $p + 4, 48) = $green;
                substr($b, $p + 4 + 96, 48) = $green;
                $patched = 1;
                last;
            }
            $o = $p + $size;
        }
        die "$src: no WarpRectilinear in OpcodeList3\n" unless $patched;
        open(my $out, ">:raw", $dst) or die "write $dst: $!\n";
        print $out $b; close($out) or die "close $dst: $!\n";
        print "  WarpRectilinear red and blue sets overwritten by green (OpcodeList3 at byte $list)\n";
    ' "$src" "$dst" || die "could not patch the warp in $src"
}

run_stage3() {
    input=$1
    stem=$2
    rm -f "$stem.tif"
    "$validate" -3 "$stem" "$input" >"$stem.log" 2>&1 \
        || die "dng_validate failed on $input (see $stem.log)"
    [ -s "$stem.tif" ] || die "dng_validate wrote no $stem.tif (see $stem.log)"
}

count=0
for entry in \
    "q2:LEICA-Q2/L1000750.DNG" \
    "q3:LEICA-Q3-PB/leica_q3_01.dng" \
    "sl3:LEICA-SL3-PB/leica_sl3_01.dng"; do
    tag=${entry%%:*}
    path=$corpus/${entry#*:}
    [ -f "$path" ] || die "missing corpus file: $path"
    echo "$tag: $path"

    run_stage3 "$path" "$out_abs/$tag-full"
    green_only_warp "$path" "$out_abs/$tag-green.dng"
    run_stage3 "$out_abs/$tag-green.dng" "$out_abs/$tag-green"
    echo "  wrote $tag-full.tif and $tag-green.tif"
    count=$((count + 1))
done

[ "$count" -eq 3 ] || die "expected 3 files, processed $count"
echo "sdk-stage3: 3 files done; set IRRADIANCE_SDK_STAGE3_DIR=$out_abs"
