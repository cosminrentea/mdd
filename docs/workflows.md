# Usage and agent workflows

Updated: 2026-10-31

These examples run from the repository root using the included fixtures. Replace the paths and patterns with your own.

## Read only relevant sections

Updated: 2026-10-31

```bash
mdd toc tests/fixtures/simple.md
mdd sec tests/fixtures/simple.md "Section Two" "Nested Subsection"
```

The outline shows heading levels and section ranges. In agent format, `[tests/fixtures/simple.md:16,10]` identifies a section starting at line 16 with a 10-line limit. Use those values with a reader that accepts a 1-based offset and line count, or use `sec` to print the section. A parent section includes its descendants.

## Expand a search result

Updated: 2026-10-31

```bash
grep -n 'Content in a nested' tests/fixtures/simple.md
mdd at tests/fixtures/simple.md 35
mdd at tests/fixtures/simple.md 35 1
```

Use the line number returned by `grep`; the first `at` call selects the innermost section and the second climbs one heading level. In human format, the breadcrumb appears on standard error; JSON output includes it as a field.

## Explore a collection

Updated: 2026-10-31

```bash
mdd toc tests/fixtures/
mdd find tests/fixtures/ --type reference --has-table
mdd agg tests/fixtures/ --heading "Section Two" --budget 50
mdd fm tests/fixtures/ --list-values type
mdd links tests/fixtures/cross-refs.md --broken
```

`find` returns matching section locations, while `agg` includes their content. Its budget is a target, not a hard cap on the first emitted section. `links --broken` checks local path existence only. See [directory traversal and filter details](commands.md#query-a-directory).

## Inspect multi-entry documents

Updated: 2026-10-31

```bash
mdd parts tests/fixtures/lrn-two-entries.md
mdd parts tests/fixtures/lrn-two-entries.md --filter type=reference
```

Entries are separated by YAML frontmatter blocks. `parts` reports entry locations and heading summaries so a reader can select an entry without loading the entire file.
