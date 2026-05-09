
- Folder contains Markdown files:
	- produced by a coding-agent for future coding-agents consumption
	- filename format = `YYYY-MM-DD`.md
- Each file contains what was or is produced DURING THAT SPECIFIC DAY (in the past, or today), in **distinct sections**, with each section containing these fields in a front-matter:
	- `type` = **one or more of**: `reference`, `feedback`, `project`, `user`, `note`, `bug`, `decision`
		- `reference`, `feedback`, `project`, `user` = direct auto-memory import targets (see `obs/memory/`)
		- `note` = catch-all for items that may not survive import triage
		- `bug` = bug discovered (not the fix — fixes belong in git history)
		- `decision` = architectural or design decision made, with rationale
	- `level` = **one of**: `project` , `user`
	- `topic` = **free-form, 1-3 words** — maps to memory file names during import (e.g. `jenkins`, `kafka patterns`, `PES enrichment`). Use lowercase
	- `weight` = **number** between 0.01 (least important) and 1.0 (most important), must be estimated by producing coding-agent
	- `weight-reason` = **one-liner** explaining why this weight was assigned — helps user judge during import triage
	- `agent` = (optional) **name** of the agent that produced this entry (e.g. `A-architect`, `A-tester`, `main`). Helps judge confidence during import
	- `expires` = (optional) **date** when this section should be checked / validated again. Omit for timeless knowledge
- Always use newlines before AND after `---` markers
- All dates must be absolute, with format = `YYYY-MM-DD`
- User could make manual updates. Never rely on the file creation and update times.
- You can create/update only the file for TODAY ( command = `date -Idate` ). NEVER modify files from the past dates.
- You can append or update sections freely in the file for `TODAY`

## Reading (for agents)
- Check `_import.md` for `last_imported_date`
- Read all `YYYY-MM-DD.md` files with dates > `last_imported_date` (inclusive of that date, through today)
- Treat "lrn" content as unvalidated — lower confidence than curated memory (`obs/memory/`)
- When "lrn" content conflicts with curated memory:
	- only if quick or easy: investigate
	- else: trust curated memory
