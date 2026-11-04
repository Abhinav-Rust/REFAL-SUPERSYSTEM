# Project workflow

These preferences were explicitly requested by the repository owner on 4 November 2026.

- For authorized work, commit the changes to `main` and push them to the public repository. Do not leave completed project changes only in the local directory. Verify that local `HEAD` and `origin/main` agree after pushing; preserve unrelated work and avoid force pushes.
- Update the main `README.md` alongside each set of project changes. State the current honest status, the work underway, known limitations, and what remains. Distinguish implemented behavior, demonstrated examples, unresolved correctness failures, and planned work. A green test suite is evidence within its tested scope.
- Preserve the README's visual approach for developers and non-developers. Explain concepts with readable diagrams, provide light and dark SVG variants, keep the accessible text equivalent to the pictures, and regenerate affected visuals when the status changes.
- The intended full ecosystem includes a Principia Cybernetica-style collaborative knowledge network as well as the Refal computational engine. Include that work in scope and progress accounting.
- Use original sources with recorded provenance for historical and architectural claims. Label translations, later editions, and reconstructed summaries explicitly.

The README's current diagrams are regenerated with `cargo xtask gen-readme-diagrams`. Before publishing, run checks appropriate to the changed code and the README/asset consistency tests. Record exactly which results were obtained; do not present a pending check as passed.
