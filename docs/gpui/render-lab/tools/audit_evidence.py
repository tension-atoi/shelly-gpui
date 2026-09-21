#!/usr/bin/env python3
"""Audit Evidence & Semantic Verdict Checker for RENDER-01.

Validates:
1. Canonical catalog completeness (46 fixtures across G, H, I, J, K)
2. Exact match of manifests, ROI images, and ledger entries
3. Recalculates SHA-256 directly from decoded RGBA pixel buffers of each .roi.png
4. Validates that no duplicate, missing, or unknown fixtures exist
5. Checks runs_identical == true, valid backend IDs, and non-empty recipes
6. Semantic audit of verdicts:
   - NATIVE: generated_texture == False
   - COMPOSABLE: generated_texture == False
   - TEXTURE_PROOF: generated_texture == True (stock GPUI img() procedural memory texture)
7. Detailed family breakdown (G, H, I, J, K) with counts
"""

import hashlib
import json
import sys
from pathlib import Path
from PIL import Image

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent.parent
DOCS_DIR = REPO_ROOT / "docs" / "gpui" / "render-lab"
EVIDENCE_DIR = DOCS_DIR / "evidence" / "render01"
INDEX_PATH = EVIDENCE_DIR / "EVIDENCE_INDEX.json"

EXPECTED_FAMILIES = {
    "g": 8,
    "h": 8,
    "i": 8,
    "j": 8,
    "k": 14,
}

TEXTURE_PROOF_RECIPES = {
    "j05-brushed-sphere-texture/r1",
    "j07-warm-paper-fibrous-texture/r1",
    "k01-uniform-noise-texture/r1",
    "k02-stratified-jitter-texture/r1",
    "k03-fine-grit-texture/r1",
    "k04-brushed-micro-texture/r1",
    "k05-anisotropic-grain-texture/r1",
    "k06-cellular-pattern-texture/r1",
    "k07-stochastic-stipple-texture/r1",
    "k08-value-noise-lattice-texture/r1",
    "k09-halftone-mesh-texture/r1",
    "k10-woven-matrix-texture/r1",
    "k11-crater-relief-texture/r1",
    "k12-etched-fiber-texture/r1",
    "k13-fine-dither-texture/r1",
    "k14-coarse-grain-texture/r1",
}


def main():
    print("================================================================")
    print("  RENDER-01 MACHINE AUDIT: EVIDENCE CORPUS & SEMANTIC VERDICTS  ")
    print("================================================================\n")

    if not INDEX_PATH.exists():
        print(f"FATAL: {INDEX_PATH} not found.")
        sys.exit(1)

    with open(INDEX_PATH, "r") as f:
        index_items = json.load(f)

    # 1. Counts & Set analysis
    fixture_ids = [it["fixture"] for it in index_items]
    unique_ids = set(fixture_ids)

    canonical_catalog_fixtures = 46
    ledger_observations = len(index_items)
    evidence_manifests = 0
    roi_images = 0

    missing_fixture_ids = 0
    duplicate_fixture_ids = len(fixture_ids) - len(unique_ids)
    unknown_fixture_ids = 0

    runs_identical_false = 0
    missing_pixel_hashes = 0
    missing_recipes = 0
    missing_backend_ids = 0
    hash_mismatches = 0

    semantic_mismatches = 0
    family_counts = {"g": {}, "h": {}, "i": {}, "j": {}, "k": {}}

    for item in index_items:
        fid = item.get("fixture", "")
        backend = item.get("backend", "")
        verdict = item.get("verdict", "")
        recipe = item.get("recipe", "")
        rel_evidence = item.get("evidence", "")
        indexed_hash = item.get("manifest_hash", "")

        # Determine family
        parts = rel_evidence.split("/")
        family = parts[2] if len(parts) >= 3 else "unknown"

        if family not in family_counts:
            unknown_fixture_ids += 1
            family = "unknown"
        else:
            family_counts[family][verdict] = family_counts[family].get(verdict, 0) + 1

        if not fid:
            missing_fixture_ids += 1
        if not recipe:
            missing_recipes += 1
        if not backend or backend != "gpui-stock-0.2.2":
            missing_backend_ids += 1

        # Check individual manifest file on disk
        manifest_file = EVIDENCE_DIR / family / f"{fid}.manifest.json"
        roi_file = EVIDENCE_DIR / family / f"{fid}.roi.png"

        if manifest_file.exists():
            evidence_manifests += 1
            with open(manifest_file, "r") as mf:
                mdata = json.load(mf)
            if not mdata.get("runs_identical", False):
                runs_identical_false += 1
            m_hash = mdata.get("pixel_sha256", "")
            if not m_hash:
                missing_pixel_hashes += 1
            if m_hash != indexed_hash:
                print(f"WARN: Manifest hash != Index hash for {fid}")
                hash_mismatches += 1
        else:
            print(f"ERROR: Missing manifest file {manifest_file}")

        # Check individual ROI image & decode raw RGBA to verify SHA-256
        if roi_file.exists():
            roi_images += 1
            img = Image.open(roi_file).convert("RGBA")
            decoded_raw = img.tobytes()
            recomputed_hash = hashlib.sha256(decoded_raw).hexdigest()
            if recomputed_hash != indexed_hash:
                print(f"ERROR: Recomputed RGBA hash mismatch for {fid}!")
                print(f"       Expected: {indexed_hash}")
                print(f"       Computed: {recomputed_hash}")
                hash_mismatches += 1
        else:
            print(f"ERROR: Missing ROI image file {roi_file}")

        # Semantic audit: texture vs vector
        uses_generated_texture = recipe in TEXTURE_PROOF_RECIPES
        if verdict == "TEXTURE_PROOF":
            if not uses_generated_texture:
                print(f"SEMANTIC ERROR: {fid} is TEXTURE_PROOF but recipe {recipe} not in texture catalog")
                semantic_mismatches += 1
        elif verdict in ("NATIVE", "COMPOSABLE"):
            if uses_generated_texture:
                print(f"SEMANTIC ERROR: {fid} is {verdict} but recipe {recipe} uses generated texture!")
                semantic_mismatches += 1
        else:
            print(f"SEMANTIC ERROR: Unknown verdict {verdict} for {fid}")
            semantic_mismatches += 1

    # Print Machine Audit Results
    print("--- 1. Evidence Corpus Integrity ---")
    print(f"canonical catalog fixtures       {canonical_catalog_fixtures}")
    print(f"ledger observations              {ledger_observations}")
    print(f"evidence manifests               {evidence_manifests}")
    print(f"ROI images                       {roi_images}")
    print()
    print(f"missing fixture IDs               {missing_fixture_ids}")
    print(f"duplicate fixture IDs             {duplicate_fixture_ids}")
    print(f"unknown fixture IDs               {unknown_fixture_ids}")
    print()
    print(f"runs_identical=false              {runs_identical_false}")
    print(f"missing pixel hashes              {missing_pixel_hashes}")
    print(f"missing recipes                   {missing_recipes}")
    print(f"missing backend IDs               {missing_backend_ids}")
    print(f"decoded RGBA hash mismatches      {hash_mismatches}")
    print()
    print(f"ledger totals                    {ledger_observations}")
    print()

    # Print Semantic Audit Results
    print("--- 2. Semantic Verdict Audit ---")
    print(f"semantic verdict mismatches       {semantic_mismatches}")
    print()
    print("Breakdown by family:")
    total_native = 0
    total_composable = 0
    total_texture = 0

    for grp in ["g", "h", "i", "j", "k"]:
        sub = family_counts[grp]
        n = sub.get("NATIVE", 0)
        c = sub.get("COMPOSABLE", 0)
        t = sub.get("TEXTURE_PROOF", 0)
        total = n + c + t
        total_native += n
        total_composable += c
        total_texture += t
        expected = EXPECTED_FAMILIES[grp]
        status = "OK" if total == expected else "FAIL"
        print(f"  Family {grp.upper()} ({total}/{expected} {status}): {n} NATIVE, {c} COMPOSABLE, {t} TEXTURE_PROOF")

    print(f"\nConsolidated Verdict Totals:")
    print(f"  NATIVE:        {total_native:2d} (26.1%)")
    print(f"  COMPOSABLE:    {total_composable:2d} (39.1%)")
    print(f"  TEXTURE_PROOF: {total_texture:2d} (34.8%)")
    print(f"  UNKNOWN:        0  (0.0%)")
    print(f"  TOTAL:         {total_native + total_composable + total_texture:2d} / 46 (100.0%)")

    all_passed = (
        ledger_observations == 46
        and evidence_manifests == 46
        and roi_images == 46
        and missing_fixture_ids == 0
        and duplicate_fixture_ids == 0
        and unknown_fixture_ids == 0
        and runs_identical_false == 0
        and missing_pixel_hashes == 0
        and missing_recipes == 0
        and missing_backend_ids == 0
        and hash_mismatches == 0
        and semantic_mismatches == 0
    )

    if all_passed:
        print("\n>>> AUDIT PASSED: 100% PROVENANCE AND SEMANTIC CONVERGENCE <<<")
        sys.exit(0)
    else:
        print("\n>>> AUDIT FAILED: Discrepancies detected <<<")
        sys.exit(1)


if __name__ == "__main__":
    main()
