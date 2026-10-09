---
name: changelog
description: Maintains strict user-facing CHANGELOG.md files in Keep a Changelog 2.0.0 style. Use for “update the changelog”, “add a changelog entry”, “create a changelog”, or to view a changelog without file changes.
user-invocable: true
license: CC-BY-SA-4.0
metadata:
  version: 1.0.0
---

# Changelog

Maintain a strict, user-facing `CHANGELOG.md` in Keep a Changelog 2.0.0 style.

- Write for end users in plain language. Fast reading, easy comprehension. Emphasize clarity, brevity, and consistency.
- Be conservative: never invent entries; when evidence conflicts, prefer code and scoped diffs over commit subjects.
- Keep released history frozen.
- Leave internal work out.

## Scope

Resolve the target in this order:

1. Explicit `CHANGELOG.md` path from the request.
2. Explicit repo root or path inside a repo from the request.
3. The current repo root (`git rev-parse --show-toplevel`).

If still unclear after checking the request and current directory, ask one short question.

## View mode

When the user only wants to see the changelog (no file changes):

1. Read the resolved `CHANGELOG.md`.
2. Report latest release, date, `Unreleased` highlights, and notable breaking changes.
3. Do not edit anything.

View mode is complete when the summary covers the newest release and everything currently under `Unreleased`.

## Update mode

### Gather

1. Collect repo context: latest semver-like tags (`git tag --sort=-version:refname | head -n 20`), status, and origin URL.
2. Define:
   - `latest_repo_tag`: newest semver-like git tag in the repo.
   - `last_listed_tag`: newest released version already present in `CHANGELOG.md`.
3. Normalize tags by stripping one leading `v` only for headings and link labels; use raw tags for git commands and URLs.
4. For content sources, prefer explicit user notes, then code, docs, existing release notes, and scoped diffs over commit subjects. Never dump git logs into the changelog.

### Edit boundaries

Edit only these regions:

- `Unreleased`
- the release insertion point at the second `<!-- next-header -->`
- the link reference block at `<!-- next-url -->`

Preserve marker comments and all past released sections unchanged.

### Version rule

If `last_listed_tag` is older than `latest_repo_tag`:

1. Insert exactly one new released section for `latest_repo_tag` after the release insertion marker.
2. Use the tag date in ISO `YYYY-MM-DD`; add `[YANKED]` if that tag is a yanked release.
3. Point `[Unreleased]`'s compare link at `latest_repo_tag...HEAD`.
4. Add or update the new release link under `<!-- next-url -->`.

Do not backfill multiple missing releases in one run unless the user explicitly asks.

Otherwise update only `Unreleased`; do not create a released section or link.

### File creation

If no changelog exists, create it with this skeleton:

```markdown
<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

<!-- next-header -->

<!-- next-url -->

[Unreleased]: {{REPO_URL}}/compare/v{{LAST_TAG}}...HEAD
```

Reference whichever versioning scheme the project actually uses instead of Semantic Versioning when it differs. If there is no existing link style and the remote is not GitHub-style, ask one short question about release URLs instead of inventing them.

### Select changes

Include only notable, user-visible changes since the last listed release. Translate technical implementation into user outcomes. One bullet per change, starting with the outcome; keep bullets short and plain.

Good: `- Added keyboard shortcuts for zoom.`
Bad: `- Refactored accelerator handling and normalized keymaps.`

Exclude invisible work: refactors, tests, CI, docs-only changes, formatting, file moves, internal cleanup, internal tooling, dependency bumps without user-visible effect. If nothing user-visible changed, report that instead of padding the file.

Use the six types as subsections, omitting empty ones: `Added` (new features), `Changed` (changed existing behavior), `Deprecated`, `Removed`, `Fixed` (behavior was wrong, now correct), `Security` (addresses a vulnerability). No other section names. When unsure between them: old behavior was a bug → `Fixed`; intentional difference → `Changed`; vulnerability → `Security`, leading with its CVE identifier when one exists.

Describe dependency updates by their user-visible effect under the right type, not as a separate type. Mark breaking changes inline within their type with a short `**Breaking:**` marker and what breaks; keep upgrade steps brief in the entry or link out to a migration guide.

Replace pre-existing `{{placeholders}}` whose data is now available; use `{{placeholders}}` for unknown values like repository URLs.

### Complete

The update is complete when the file contains every notable user-visible change since `last_listed_tag` sorted into type sections, released sections untouched, links consistent, and no invented entries. Report the target file, whether you updated `Unreleased` or created a release section, and anything that blocked accuracy. Keep the response concise.
