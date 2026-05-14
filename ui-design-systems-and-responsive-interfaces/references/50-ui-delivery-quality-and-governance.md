# UI Delivery, Quality, and Governance

## Handoff Standards

- Provide component specifications tied to tokens and states.
- Provide responsive behavior notes and breakpoint rules.
- Provide accessibility acceptance criteria per feature.
- Provide interaction and error-state behavior for each flow.

## UI Quality Gates

Require at least:

1. Visual regression checks on critical surfaces
2. Accessibility checks (automated and manual sampling)
3. Responsive checks on target viewport matrix
4. Interaction checks for keyboard and touch paths
5. Performance checks for user-critical pages/flows
6. Cross-page consistency checks for shared components and tokens
7. Localized content expansion and truncation checks for key flows

## Collaboration Workflow

- Pair design and engineering early during architecture definition.
- Review implementation against spec before release.
- Capture variance decisions and feed them back into the design system.
- Use reviewer findings to create concrete follow-up tasks, not generic “polish later” notes.

## Production Monitoring

- Track UX quality indicators (task completion, UI error frequency, interaction drop-off).
- Track web performance/latency indicators for UI-heavy flows.
- Use real-user feedback and usability signals to prioritize improvements.
- Include regressions from support tickets and user recordings in UI debt prioritization.

## Tools and Testing

### Design Tools
- Figma, Sketch, Adobe XD for design
- Design tokens (Style Dictionary, Theo)
- Component libraries (Storybook, Bit)
- Pencil, when the workspace already uses it, for code-first design artifacts

### Testing Tools
- Storybook, Ladle, or Histoire for isolated component states
- **Accessibility**: axe DevTools, Lighthouse, WAVE
- **Contrast**: WebAIM Contrast Checker
- **Screen Readers**: NVDA (Windows), JAWS, VoiceOver (Mac/iOS)
- **Responsive**: Browser DevTools, real devices
- **Visual Regression**: Percy, Chromatic, BackstopJS

## Team Best Practices

1. Mobile First: design for smallest screen, enhance up
2. Semantic HTML: use correct elements for meaning
3. Design Tokens: centralize design decisions
4. Component Reuse: don't duplicate, extend
5. Accessibility: build in from start, not retrofit
6. Real Testing: test on actual devices and assistive tech
7. Performance: optimize images, minimize layout shifts
8. Documentation: keep design system docs current
9. Consistency: follow established patterns
10. Verify Components in Isolation: use Storybook, Ladle, or Histoire when compatible
11. Design for Brownfield Change: modernize surgically, preserve proven assets
12. User Focus: design for real users, not just aesthetics
