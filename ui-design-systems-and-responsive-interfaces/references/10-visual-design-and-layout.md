# Visual Design and Layout Foundations

## Visual Hierarchy

- Prioritize content by size, contrast, spacing, and placement.
- Keep primary actions visually dominant and secondary actions clearly subordinate.
- Use consistent hierarchy patterns across screens to reduce cognitive load.

## CTA Hierarchy and Labeling

- Define one primary CTA for each key decision context.
- Keep secondary actions available but visually subordinate.
- Reserve destructive CTA styling for irreversible/high-risk actions.
- Use explicit CTA labels that describe outcome (for example “Save Draft”, “Confirm Payment”).
- Avoid multiple competing primary CTAs in the same viewport section.

## CTA Color and Positioning Rules

- Use semantic CTA color mapping consistently:
  - primary action -> primary brand/action color
  - secondary action -> neutral/subdued treatment
  - destructive action -> danger/destructive color only for destructive outcomes
- Keep CTA text/background contrast compliant with accessibility targets.
- Keep primary CTA placement predictable within a flow (users should know where to look next).
- Keep safe spacing between primary and destructive actions to reduce accidental clicks/taps.
- Avoid visually loud CTA styling that competes with critical content hierarchy.

## Layout Systems

- Use grid systems with predictable columns/gutters/margins.
- Keep spacing scales consistent through tokens, not ad-hoc values.
- Design for content growth, not fixed sample text lengths.

## Typography and Readability

- Use an explicit type scale for headings/body/captions.
- Keep line length and spacing readable across viewport sizes.
- Ensure text remains legible at zoom and system text scaling settings.
- Prefer fluid typography with bounded scaling for large/small screens.

## Visual Rhythm and Density

- Use consistent spacing rhythm to reduce cognitive friction between sections.
- Tune density by task type:
  - high-frequency workflows may need denser layouts
  - learning/decision-heavy workflows need more breathing room
- Keep primary actions and critical data visible without forcing decorative complexity.

## Color and Contrast

- Use semantic color tokens (success, warning, error, info) rather than raw color values.
- Validate contrast ratio for text and interactive elements against target accessibility level.
- Avoid color-only meaning; pair with labels/icons/patterns.
- Keep color role mapping stable across screens (same color role, same meaning).

## Theme-Mode Legibility Rules (Light/Dark)

- Validate all text tiers (heading/body/meta/hint/error) in both light and dark modes.
- Validate CTA/button state visibility (default/hover/focus/active/disabled/loading) in both modes.
- Ensure icons and low-emphasis controls remain visible in both modes.
- Keep semantic status meaning (success/warning/error/info) consistent across modes.
- Avoid token overrides that fix one mode while breaking the other.

## Authentic Visual Direction (Non-Generic)

- Avoid defaulting to trend-heavy patterns without product rationale.
- Use restrained gradients, glow, and blur only when brand guidelines justify them.
- Favor clear hierarchy, purposeful whitespace, and content clarity over decorative effects.
- Benchmark real-world products for principles, not for copying visual signatures.
- Remove visual noise that weakens task clarity (unnecessary badges, icons, shadows, or decorative wrappers).

## Motion and Feedback

- Use motion to clarify state change and hierarchy, not decorative noise.
- Keep transitions fast and predictable.
- Provide immediate feedback for user actions and asynchronous states.
- Respect reduced-motion user preferences and provide equivalent non-motion cues.

## Copy and Flow Defaults

Keep UI language short and useful:
- Prefer short labels over descriptive slogans
- Default to 1-4 word headings and direct button text
- Use helper text only when the next action, requirement, or consequence is not obvious
- Avoid adding a descriptive sentence under every heading by default
- Do not narrate the interface with lines that sound generated, promotional, or overly clever
- Use verbs for actions and nouns for navigation
- Say what happens next: `Save draft`, `Create report`, `Send invite`, `Retry payment`
- Keep destructive or high-trust actions explicit: `Delete workspace`, `Pay now`, `Share publicly`
- Prefer familiar product language that users already know in that category

Treat flow as more important than decorative copy:
- Remove steps before adding explanation
- Keep one main action per area
- Place supporting text near the field, toggle, or decision it explains
- If users need repeated explanation, simplify the layout, labels, or defaults first
- Break long setup and forms into short grouped steps when the task has distinct decisions
- Preserve momentum with visible progress, saved state, and clear recovery paths
- Keep dashboards and dense tools scannable: short labels, stable layout, obvious filters, obvious next action

Before finalizing, prune copy aggressively:
- Remove any sentence that does not change a decision, reduce an error, build trust, or improve comprehension
- Replace abstract section intros with concrete labels
- Turn multi-sentence helper text into bullets only when users truly need stepwise guidance
- If the interface still feels wordy, cut text before adding more styling
