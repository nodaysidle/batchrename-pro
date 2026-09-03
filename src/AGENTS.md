# src — Application/frontend source

## Purpose

Owns the main application UI/runtime source for this project.

## Ownership

- `App.tsx`
- `assets`
- `components`
- `contexts`
- `hooks`
- `index.css`
- `lib`
- `main.tsx`
- `state`
- `types.ts`
- `vite-env.d.ts`

## Local Contracts

- Preserve the current frontend stack and component architecture.
- Keep UI polished, accessible, and dark-mode friendly where applicable.
- Do not introduce new frameworks without approval.
- Job processing flags live in `state/reducer.ts`. `COMPLETE_JOB` wins over a later `START_JOB` for the same id (tiny-job event reorder). Prove that with `npm run test:unit`.

## Work Guidance

- Read this file after the root `AGENTS.md` before editing this subtree.
- Prefer extending existing modules/files over creating parallel duplicate systems.
- Update this `AGENTS.md` only when durable ownership, contracts, or verification guidance changes.

## Verification

- Frontend/build check from root package manifest when behavior changes.
- Job state machine: `npm run test:unit`

## Child DOX Index

None.
