// Local SDK reference (SPEC-085): print the SDK's camera->PCS (XYZ D50) matrix.
//
// This repo's own code. It is compiled and linked against a locally built
// Adobe DNG SDK 1.7.1 by scripts/sdk-colour.sh, never in CI, and the binary
// and its output are never committed (DEC-069 §6, DEC-076). It prints, for the
// default camera profile at the SDK's own NeutralToXY(AsShotNeutral) white:
//   white_xy <x> <y>
//   neutral  <r> <g> <b>          the camera neutral (AsShotNeutral)
//   cam2pcs  <three entries>      three rows of CameraToPCS()
#include "dng_host.h"
#include "dng_info.h"
#include "dng_negative.h"
#include "dng_file_stream.h"
#include "dng_color_spec.h"
#include "dng_camera_profile.h"
#include "dng_xy_coord.h"
#include <cstdio>
int main(int argc, char **argv) {
  if (argc < 2) return 2;
  try {
    dng_file_stream stream(argv[1]);
    dng_host host;
    dng_info info;
    info.Parse(host, stream);
    info.PostParse(host);
    if (!info.IsValidDNG()) { fprintf(stderr, "not valid\n"); return 1; }
    AutoPtr<dng_negative> neg(host.Make_dng_negative());
    neg->Parse(host, stream, info);
    neg->PostParse(host, stream, info);
    AutoPtr<dng_color_spec> spec(neg->MakeColorSpec(dng_camera_profile_id()));
    dng_xy_coord w = spec->NeutralToXY(neg->CameraNeutral());
    spec->SetWhiteXY(w);
    const dng_matrix &m = spec->CameraToPCS();
    printf("white_xy %.12f %.12f\n", w.x, w.y);
    const dng_vector &n = neg->CameraNeutral();
    printf("neutral %.12f %.12f %.12f\n", n[0], n[1], n[2]);
    for (uint32 r = 0; r < 3; r++) printf("cam2pcs %.12f %.12f %.12f\n", m[r][0], m[r][1], m[r][2]);
  } catch (...) { fprintf(stderr, "exception\n"); return 1; }
  return 0;
}
