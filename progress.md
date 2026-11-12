# Copyfie — Progress Log

> Read this file and `task.md` at the start of every session.
> Append a new dated section per session. Never rewrite history; correct it with a new entry.

---

## 2026-11-12 — Session 1 (Phase 0: docs and setup)

**Scope:** Phase 0 only — no app code, no scaffolding.

### Done

- **Paddle docs MCP:** added `paddle-docs` (`https://paddlehq.mcp.kapa.ai`) to the global OpenCode config via `opencode mcp add paddle-docs --global --url …`. Status: **needs authentication** — sign in with `/mcps` (Google/GitHub) when convenient. No Paddle account or API key was created.
- **PLAN.md edits:**
  - Naming clearance moved to v1.0 task 0; release gate, site, Paddle onboarding, and all later tasks shifted by one (v1.0 now 0–29).
  - Locked mode now allows **Clear history** with a confirmation (clears all items, including pinned, because pins are hidden while locked). Updated the decisions table, §7 locked-mode table, §14 matrix, acceptance criteria, and the manual checklist.
- **Task tracking:** created `task.md` (all milestone tasks, IDs M1/M2/MX/M3/M4) and this `progress.md`.
- **Git:** `.gitignore` created; repo initialized; `origin` set to `https://github.com/dhruv-kashyap47/copyfie.git`; PLAN.md, task.md, progress.md, .gitignore committed and pushed.

### Decisions

- Milestone IDs: `M1` = MVP, `M2` = v1.0, `MX` = deferred v1.x, `M3` = v1.1, `M4` = v1.2.
- `logo/` is deliberately **not** committed yet (untracked local asset); decide later whether it belongs in the repo.
- Paddle work (account, backend, checkout) stays deferred to M2; consult the docs MCP first when it starts.

### Blockers

- `paddle-docs` MCP awaits OAuth sign-in (`/mcps`). Not blocking Phase 0.

### Measurements

- None yet (no app build).

### Next up

- Stop and wait for **"start Milestone 1"**.
- When approved: **M1-01** (repo/workspace, Tauri hello, CI skeleton, `cargo-deny`, NOTICE/THIRD_PARTY), then M1-02 onward in order.
