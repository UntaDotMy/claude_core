# Accessibility and Inclusive UI

## Accessibility Baseline

- Define target conformance level and audit criteria early.
- Ensure full keyboard support for all interactive paths.
- Ensure semantic structure and labeling for assistive technologies.
- Ensure focus order is logical and visible.
- Ensure focus is not fully obscured by sticky headers/overlays.
- Ensure minimum target size and spacing for touch-heavy interfaces.

## Inclusive Interaction

- Support multiple input modes (touch, keyboard, pointer, voice where relevant).
- Keep target sizes large enough for reliable interaction.
- Avoid time-sensitive interactions without user control.
- Provide clear error prevention, detection, and recovery cues.
- Respect user preferences such as reduced motion and contrast-related needs.

## Content and Language

- Use plain language and predictable terminology.
- Support localization, text expansion, and right-to-left rendering when required.
- Ensure icon-only controls include clear accessible names.
- Keep form guidance actionable with specific error messages and recovery paths.

## Accessibility QA

- Combine automated checks with manual audits.
- Include screen reader checks for critical flows.
- Include zoom/reflow checks and color-contrast validation.
- Track accessibility issues in backlog with severity and user impact.
- Validate keyboard interaction patterns against common WAI-ARIA practices where relevant.
- Include dark/light mode checks for text legibility and control visibility.
- Include button/CTA state checks (focus/disabled/loading/error) in both modes.

## Semantic HTML

```html
<header>, <nav>, <main>, <article>, <section>, <aside>, <footer>
<button> for actions, <a> for navigation
<h1>-<h6> for headings (logical hierarchy)
<label> for form inputs
```

## ARIA (Use Sparingly)

- Use semantic HTML first
- Add ARIA when HTML semantics insufficient
- Common: `aria-label`, `aria-describedby`, `aria-live`, `role`

## Keyboard Navigation

- Tab order follows visual order
- All interactive elements keyboard accessible
- Escape closes modals/dropdowns
- Enter/Space activates buttons
- Arrow keys for custom controls

## Screen Reader Testing

- Test with actual screen readers (NVDA, JAWS, VoiceOver)
- Ensure logical reading order
- Verify all content accessible
- Check form labels and error messages
