# Theta Agent Implementation Review — Pending Gaps Only

## Pending Lower-Risk Gaps

1. **Manual parity verification still pending**
   - Re-run and record manual scenarios to confirm no regressions:
     - `/skill:git-commit` + minimal follow-up (`2`) proceeds with git inspection/actions.
     - “Implement X” triggers immediate tool execution (not acknowledgement-only).
     - Tool-call parsing mismatch surfaces explicit error/diagnostic (no silent idle).

2. **Optional UX diagnostics parity still pending (non-blocking)**
   - Decide whether to add extra visible breadcrumbs for:
     - tool-call detected count
     - tool-round transitions
   - Current error visibility exists; this is only additional observability parity.

## Pending Process Parity Outputs (Docs)

The following review artifacts are still missing and remain pending:

- `docs/review/repro-cases.md`
- `docs/review/theta-vs-pi-contract-diff.md`
- `docs/review/root-cause-analysis.md`
- `docs/review/fix-plan.md` (reduced to residual/non-blocking items only)
- `docs/review/validation-matrix.md`
- `docs/review/final-summary.md`

## Pending Completion Criteria

Done when all pending items above are completed:
- Manual parity scenarios executed and recorded.
- Process parity documents created with evidence.
- Residual risk list finalized (only lower-risk/non-blocking items).