# llms.json and llms.txt

Create AI guidance helper files at the repo root:

- `llms.json` -> JSON
- `llms.txt` -> markdown text

Purpose: Provide AI tools with a clean, curated map of its most important content.

Help large language models (LLMs) read, understand, and cite a site's documentation or resources without getting bogged down 

File size:  < 40k bytes.

Link targets: the workspace-root `llms.txt`/`llms.json` use repo-relative
links (e.g. `README.md`), which only resolve inside the git checkout.
Serving that exact text from `*.github.io/llms.txt` would ship links that
don't resolve on the live site, so `*.github.io/static/` instead carries
website-appropriate versions of both files — pointing each entry at
wherever it actually resolves from the site's own domain, not a copy of
the root files' text.
