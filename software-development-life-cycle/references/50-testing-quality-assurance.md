# Testing and Quality Assurance

## Testing Fundamentals

Apply layered testing:

- Unit tests: verify smallest behavior units in isolation.
- Integration tests: verify module/service boundaries.
- System tests: verify end-to-end behavior in realistic environments.
- Regression tests: preserve previous behavior after change.
- UAT: validate business fit with users/stakeholders.

## Testing Pyramid

Prefer:

- Many fast unit tests
- Fewer integration tests
- Small number of end-to-end tests for critical journeys

Avoid the inverted pyramid anti-pattern (too many brittle end-to-end tests).

## Coverage Meaning and Limits

- Treat coverage as a signal, not proof of correctness.
- High line coverage can still miss:
  - missing assertions
  - logic path gaps
  - race conditions
  - integration faults
- Track branch/condition coverage for critical modules.

## Test Quality Standards

- One behavior focus per test.
- Deterministic setup/teardown.
- Explicit assertions.
- Minimal mocking at integration boundaries only.
- Clear naming tied to expected behavior.

## What to Test

- Critical business logic
- Edge cases and error conditions
- Integration points (APIs, databases)
- Security boundaries

## When to Write Tests

- New critical functionality
- Bug fixes (test should fail before fix, pass after)
- Complex logic with edge cases
- Public APIs

## Mandatory Release Ladder

Run the applicable ladder in this order for release-ready work:

```
Smoke testing → Functional testing → Integration testing → UI testing → Load testing → Stress testing → Security testing
```

Treat the ladder as fail-closed: if any required rung fails, stays blocked, or is skipped without a justified not-applicable reason, the change is no-go.

## Definition of Done (Testing)

Require all of the following for changed behavior:

1. New/updated unit tests
2. Relevant integration tests
3. Regression protection for bug fixes
4. CI gate pass
5. Known residual risks documented
