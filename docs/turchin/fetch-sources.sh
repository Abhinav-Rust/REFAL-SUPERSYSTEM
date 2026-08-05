#!/usr/bin/env bash
#
# fetch-sources.sh — reproducibly download Valentin Turchin's primary works.
#
# The compiler in this repository is built to Turchin's own design. This script
# retrieves the primary sources that design is drawn from, so any contributor can
# verify a citation against the original text.
#
# Usage:
#   ./docs/turchin/fetch-sources.sh [target-dir]      # default: docs/turchin/pdf
#
# The PDFs are NOT committed to this repository. See docs/turchin/README.md for why.
#
# Two quirks are handled automatically; both are required:
#
#   1. pat.keldysh.ru and refal.botik.ru redirect HTTP->HTTPS with an incomplete
#      certificate chain, so curl needs -k.
#
#   2. Six documents 404 on the live mirror (link rot) and are recovered from the
#      Wayback Machine. Wayback truncates these PDFs at exactly 1 MiB: the download
#      reports success but the file is a broken PDF. The fix is to re-issue the
#      request with `curl -C -` until `pdfinfo` reports a page count. This script
#      retries automatically and verifies every file.
#
# Requires: curl, and poppler-utils (pdfinfo) for verification.

set -uo pipefail

TARGET="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pdf}"
mkdir -p "$TARGET"
cd "$TARGET" || exit 1

K="http://pat.keldysh.ru/~roman/doc"
B="http://refal.botik.ru/library"
W="https://web.archive.org/web/2019id_"

UA="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/126.0 Safari/537.36"

ok=0; fail=0

have_pages() { pdfinfo "$1" 2>/dev/null | awk '/^Pages/{print $2}'; }

# get <url> <local> <expected-pages>
get() {
  local url="$1" out="$2" want="$3" pages=""

  if [ -f "$out" ] && [ -n "$(have_pages "$out")" ]; then
    printf '  %-42s cached\n' "$out"; ok=$((ok+1)); return
  fi

  # First attempt.
  curl -kfsSL -m 300 -A "$UA" -o "$out" "$url" >/dev/null 2>&1
  pages="$(have_pages "$out")"

  # Wayback truncation: resume until the PDF validates.
  local try=0
  while [ -z "$pages" ] && [ "$try" -lt 8 ]; do
    try=$((try+1))
    curl -kfsSL -m 300 -A "$UA" -C - -o "$out" "$url" >/dev/null 2>&1
    pages="$(have_pages "$out")"
    [ -z "$pages" ] && sleep 2
  done

  if [ -z "$pages" ]; then
    printf '  %-42s FAILED\n' "$out"; rm -f "$out"; fail=$((fail+1)); return
  fi

  if [ -n "$want" ] && [ "$pages" != "$want" ]; then
    printf '  %-42s %s pages (expected %s) WARN\n' "$out" "$pages" "$want"
  else
    printf '  %-42s %s pages\n' "$out" "$pages"
  fi
  ok=$((ok+1))
}

echo "Turchin primary sources -> $TARGET"
echo
echo "Foundational papers"
get "$K/Turchin/1968-Turchin--Metaalgoritmicheskij_yazyk--ru.pdf" \
    "1968_metaalgorithmic_language.pdf" 10
get "$W/$K/Turchin/1968-Turchin--Translyator_s_Algola_napisannyj_na_yazyke_Refal.pdf" \
    "1968_algol_translator_in_refal.pdf" 20

echo
echo "Programming in the Refal Language (1971, five preprints) — the original book"
get "$W/$K/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_1_Neformal%27noe_vvedenie_v_programmirovanie_na_yazyke_refal.pdf" \
    "1971_part1_informal_intro.pdf" 57
get "$W/$K/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_2_Formal%27noe_opisanie_i_principy_realizacii_refala.pdf" \
    "1971_part2_formal_description.pdf" 60
get "$W/$K/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_3_Programmirovanie_na_bazisnom_refale.pdf" \
    "1971_part3_basic_refal.pdf" 54
get "$W/$K/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_4_Ispol%27zovanie_rekursivnyx_peremennyx_v_yazyke_refal.pdf" \
    "1971_part4_recursive_variables.pdf" 48
get "$W/$K/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_5_Ispol%27zovanie_metafunkcij_v_yazyke_refal.pdf" \
    "1971_part5_metafunctions.pdf" 56

echo
echo "Equivalence transformation and the road to supercompilation"
get "$K/Turchin/1972-Turchin--E%27kvivalentnye_preobrazovaniya_rekursivnyx_funkcij__opisannyx_na_yazyke_Refal--facsimile--ru.pdf" \
    "1972_equivalent_transformations.pdf" 15
get "$K/Turchin/1974-Turchin--E%27kvivalentnye_preobrazovaniya_programm_na_Refale--CNIPIASS--ru.pdf" \
    "1974_equivalent_transformations.pdf" 37
get "$W/$K/Turchin/1975-Turchin--Refal-makrokod.pdf" \
    "1975_refal_macrocode.pdf" 19

echo
echo "The compilation theory"
get "$K/Turchin/1980-Turchin--The_Language_REFAL--The_Theory_of_Compilation_and_Metasystem_Analysis.pdf" \
    "1980_courant_monograph.pdf" 261
get "$B/1978-Romanenko--Mashinno-nezavisimyj_kompilyator_s_yazyka_rekursivnyx_funkcij--PhD_thesis--LaTeX.pdf" \
    "1978_romanenko_compiler_thesis.pdf" 148

echo
echo "The mature supercompiler"
get "$W/$K/Turchin/1988-Turchin--The_Algorithm_of_Generalization_in_the_Supercompiler.pdf" \
    "1988_generalization_algorithm.pdf" 19
get "$W/$K/Turchin/1990-Turchin--The_Basics_of_Metacomputation--Obninsk_ch3.pdf" \
    "1990_basics_metacomputation.pdf" 63
get "$W/$K/Turchin/1990-Turchin--The_Supercompiler--Obninsk_ch6.pdf" \
    "1990_the_supercompiler.pdf" 48
get "$W/$K/Turchin/1996-Turchin--On_generalization_of_lists_and_strings_in_supercompilation.pdf" \
    "1996_generalization_lists.pdf" 28
get "$B/Turchin-Metacomputation_Metasystem_transitions_plus_supercompilation_(LNCS_vol_1110,_1996,_pp_481-509).pdf" \
    "1996_metacomputation_MST.pdf" 34
get "$B/Nemytykh-Pinchuk-Turchin_A_Self-Applicable_Supercompiler_(LNCS_vol_1110,_1996,_pp_322-337).pdf" \
    "1996_self_applicable_scp.pdf" 20

echo
echo "Later analysis"
get "http://refal.botik.ru/preprints/Antonina_Nepeivoda-On_Turchin_Theorem-06042013v1.pdf" \
    "2013_on_turchin_theorem.pdf" 13

echo
echo "-------------------------------------------------------------"
printf 'retrieved %s, failed %s\n' "$ok" "$fail"
if [ "$fail" -gt 0 ]; then
  echo
  echo "Some documents could not be retrieved. These archives are old and"
  echo "occasionally unavailable; re-run later. If a URL has rotted for good,"
  echo "try the Wayback Machine for it and open a PR updating this script."
  exit 1
fi
echo "All sources verified. Extract text with:  pdftotext <file>.pdf -"
