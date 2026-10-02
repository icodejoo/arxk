# AGENTS.md

Follow the contributor setup in [CONTRIBUTING.md](CONTRIBUTING.md). Tracker, triage, and domain docs live under [`docs/agents/`](docs/agents/).

## User-visible text

Write interface text through `t!("…")` and add its Chinese translation to
`crates/desktop_app/src/i18n/`. Rules and examples: [`docs/i18n.md`](docs/i18n.md).

## Model

Use Grok 4.6 for every task, including subagents. Do not switch to another model family unless the user explicitly names a different model.

## Videos

Do not record, generate, or attach videos (screen recordings, walkthrough clips, or similar) unless the user explicitly asks for a video.
