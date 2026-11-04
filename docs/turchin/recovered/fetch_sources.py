"""Fetch original publications for this local audit; preserve source bytes and provenance."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parent
SOURCES = [
    {
        "file": "1977_The_Phenomenon_of_Science.pdf",
        "title": "The Phenomenon of Science: A Cybernetic Approach to Human Evolution",
        "authors": ["Valentin F. Turchin"],
        "url": "https://pespmc1.vub.ac.be/POS/TurPOS.pdf",
        "kind": "pdf",
        "publication_year": 1977,
        "language": "en",
        "edition_note": "Electronic edition hosted by Principia Cybernetica; PDF page numbers are not assumed to match a print edition.",
    },
    {
        "file": "1989_The_Cybernetic_Manifesto.html",
        "title": "The Cybernetic Manifesto",
        "authors": ["Valentin Turchin", "Cliff Joslyn"],
        "url": "https://pespmc1.vub.ac.be/MANIFESTO.html",
        "kind": "html",
        "publication_year": 1989,
        "language": "en",
        "edition_note": "Author/coauthor page dated October 1989; the downloaded HTML also contains site navigation and later reader comments.",
    },
    {
        "file": "1996_Metacomputation_Metasystem_Transitions_plus_Supercompilation.pdf",
        "title": "Metacomputation: Metasystem Transitions plus Supercompilation",
        "authors": ["Valentin F. Turchin"],
        "url": "http://refal.botik.ru/library/Turchin-Metacomputation_Metasystem_transitions_plus_supercompilation_(LNCS_vol_1110,_1996,_pp_481-509).pdf",
        "kind": "pdf",
        "publication_year": 1996,
        "language": "en",
        "edition_note": "Full paper from the Refal Botik research library; PDF pagination differs from the LNCS volume pagination.",
    },
    {
        "file": "1996_A_Self_Applicable_Supercompiler.pdf",
        "title": "A Self-Applicable Supercompiler",
        "authors": ["Andrei P. Nemytykh", "Victoria A. Pinchuk", "Valentin F. Turchin"],
        "url": "http://refal.botik.ru/library/Nemytykh-Pinchuk-Turchin_A_Self-Applicable_Supercompiler_(LNCS_vol_1110,_1996,_pp_322-337).pdf",
        "kind": "pdf",
        "publication_year": 1996,
        "language": "en",
        "edition_note": "Full coauthored paper from the Refal Botik research library; PDF pagination differs from the LNCS volume pagination.",
    },
    {
        "file": "1995_Metavariables_Their_Implementation_and_Use_in_Program_Transformation.pdf",
        "title": "Metavariables: Their Implementation and Use in Program Transformation",
        "authors": ["Valentin F. Turchin", "Andrei P. Nemytykh"],
        "url": "http://refal.botik.ru/library/Turchin-Nemytykh-Metavariables_their_Implementation_and_Use_in_Program_Transformation_(City%20College%20of%20the%20City%20University%20of%20New%20York__N_TR%2095-012__1995).pdf",
        "kind": "pdf",
        "publication_year": 1995,
        "language": "en",
        "edition_note": "Image scan of CCNY technical report TR 95-012. Title and both authors checked visually on PDF page 1; pdftotext yields no searchable body text.",
    },
    {
        "file": "1993_Introduction_to_Principia_Cybernetica.html",
        "title": "Introduction to Principia Cybernetica (created 1993, revised 2003)",
        "authors": ["Francis Heylighen", "Cliff Joslyn", "Valentin Turchin"],
        "url": "https://pespmc1.vub.ac.be/INTRO.html",
        "kind": "html",
        "created_date": "1993-08-01",
        "revised_date": "2003-01-09",
        "language": "en",
        "edition_note": "Official coauthored Principia Cybernetica introduction; distinguishes the initial creation date from this revised edition.",
    },
    {
        "file": "1999_A_Dialogue_on_Metasystem_Transition.pdf",
        "title": "A Dialogue on Metasystem Transition",
        "authors": ["Valentin F. Turchin"],
        "url": "https://pespmc1.vub.ac.be/Papers/Turchin/dialog.pdf",
        "kind": "pdf",
        "original_publication_year": 1995,
        "version_date": "1999-07-12",
        "language": "en",
        "edition_note": "The PCP introduction references the dialogue as 1995, but this downloaded PDF's title page is dated July 12, 1999; its filename follows the downloaded version.",
    },
    {
        "file": "1993_On_Cybernetic_Epistemology_English.tex",
        "title": "On Cybernetic Epistemology",
        "authors": ["Valentin F. Turchin"],
        "url": "https://pespmc1.vub.ac.be/Papers/Turchin/Cybernetic_Epistemology.tex",
        "kind": "tex",
        "publication_year": 1993,
        "language": "en",
        "edition_note": "Original English TeX source hosted by Principia Cybernetica. Header cites Systems Research 10(1), 1993, pp. 3-28. The author biography links the same filename via the former FTP archive. Stored as text without executing TeX.",
    },
    {
        "file": "1993_On_Cybernetic_Epistemology_Russian_translation.pdf",
        "title": "On Cybernetic Epistemology",
        "authors": ["Valentin F. Turchin"],
        "url": "http://refal.net/origins/epystmlg/epystmlg.pdf",
        "kind": "pdf",
        "original_publication_year": 1993,
        "language": "ru",
        "translator": "Mikhail Burtsev",
        "edition_note": "Russian translation, not the original English article. Page 1 credits the translator and cites Systems Research 10(1), 1993, pp. 3-28. Translation date not established.",
    },
]


def fetch(source):
    request = urllib.request.Request(source["url"], headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(request, timeout=40) as response:
        data = response.read()
        metadata = {
            **source,
            "retrieved_utc": datetime.now(timezone.utc).isoformat(),
            "resolved_url": response.url,
            "http_status": response.status,
            "content_type": response.headers.get("Content-Type"),
            "bytes": len(data),
            "sha256": sha256(data).hexdigest(),
        }
    if source["kind"] == "pdf" and not data.startswith(b"%PDF-"):
        raise ValueError("Response is not a PDF")
    return metadata, data


def main():
    ROOT.mkdir(parents=True, exist_ok=True)
    (ROOT / "text").mkdir(exist_ok=True)
    manifest_path = ROOT / "sources.json"
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else []
    existing = {entry["file"]: entry for entry in manifest}
    pending = []
    for source in SOURCES:
        destination = ROOT / source["file"]
        expected = existing.get(source["file"])
        if destination.exists() and expected:
            if sha256(destination.read_bytes()).hexdigest() != expected["sha256"]:
                raise ValueError(f"Local source differs from the recorded edition: {destination.name}")
        else:
            pending.append(source)
    failures = 0
    with ThreadPoolExecutor(max_workers=3) as pool:
        futures = [(source, pool.submit(fetch, source)) for source in pending]
        for source, future in futures:
            try:
                metadata, data = future.result()
                expected = existing.get(source["file"])
                if expected and metadata["sha256"] != expected["sha256"]:
                    raise ValueError("The online source has changed; review its edition before updating the recorded hash")
                destination = ROOT / source["file"]
                destination.write_bytes(data)
                if source["kind"] == "pdf":
                    info = subprocess.run(["pdfinfo", str(destination)], capture_output=True, text=True, check=True)
                    metadata["pages"] = next(int(line.split(":", 1)[1]) for line in info.stdout.splitlines() if line.startswith("Pages:"))
                    text_path = ROOT / "text" / (destination.stem + ".txt")
                    subprocess.run(["pdftotext", "-layout", str(destination), str(text_path)], check=True, capture_output=True)
                    if text_path.read_text().strip():
                        metadata["text_extraction"] = "pdftotext -layout; may contain extraction errors"
                    else:
                        text_path.unlink()
                        metadata["text_extraction"] = "unavailable: image scan; no OCR performed"
                if not expected:
                    manifest.append(metadata)
                    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
                print(json.dumps(metadata, ensure_ascii=False), flush=True)
            except Exception as error:
                failures += 1
                print(json.dumps({"file": source["file"], "error": str(error)}), flush=True)
    print(json.dumps({"verified_cached": len(SOURCES) - len(pending), "downloaded": len(pending) - failures, "failed": failures}), flush=True)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
