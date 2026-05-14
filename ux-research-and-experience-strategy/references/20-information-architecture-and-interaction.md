# Information Architecture and Interaction Strategy

## Information Architecture (IA)

- Organize information by user goals and task frequency.
- Keep navigation labels clear, predictable, and unambiguous.
- Minimize depth for high-frequency tasks.
- Validate IA with card sorting/tree testing where applicable.

## User Journey and Task Flows

- Map end-to-end user journeys including pre/post interaction context.
- Identify friction points, dead ends, and error-prone steps.
- Reduce task steps where possible while preserving user control.
- Include alternate and recovery paths, not only happy paths.

## Interaction Design Rules

- Keep interactions consistent across similar contexts.
- Make system status visible after user actions.
- Provide undo/cancel where errors are expensive.
- Prevent errors before they occur (constraints, defaults, confirmation patterns).
- Prefer recognition over recall for repeated tasks (labels, prefill, progressive disclosure).
- Make next-step actions obvious: users should always understand the primary CTA and likely outcome.
- Keep action sets concise so users can choose confidently without analysis paralysis.
- Ensure theme-mode transitions (dark/light) preserve hierarchy, readability, and action confidence.

## Content and Microcopy

- Use plain, action-oriented language.
- Keep feedback messages specific and recoverable.
- Match language to user mental model, not internal system terminology.
- Keep labels and interaction terms consistent across the whole journey.
- Keep CTA wording specific and intent-revealing, especially in high-risk or paid flows.
- Remove redundant copy that repeats visible UI context without adding decision value.

## Journey Mapping

### Components
| Element | What to capture |
|---|---|
| Persona | Who is this journey for? |
| Scenario | What are they trying to accomplish? |
| Phases | Key stages of the journey |
| Actions | What users do at each phase |
| Touchpoints | Where they interact with the product |
| Thoughts/Emotions | What they're thinking/feeling |
| Pain Points | Where they struggle |
| Opportunities | Where we can improve |

### Creation Steps
1. Research actual user behavior (don't assume)
2. Identify key phases and touchpoints
3. Map actions, thoughts, emotions at each phase
4. Highlight pain points and opportunities
5. Prioritize improvements by impact
6. Validate with real users

## Flow-First UX Defaults

Prefer simpler flow over more explanation:
- Cut steps, choices, and fields before adding more copy
- Keep one clear next step on each screen or section
- Group related decisions so users do not scan the same page twice
- Keep form questions close to the input they affect
- Use progressive disclosure only when it reduces overload, not to hide core decisions
- Preserve progress across validation, auth, payment, or connectivity interruptions
- Keep recovery nearby: users should not have to hunt for retry, edit, or back actions

Keep product language practical:
- Default to short page titles, labels, and button text
- Avoid adding a supporting sentence under every heading
- Use helper text only when it prevents a likely mistake or answers a trust question
- Prefer familiar category language over invented feature names
- Avoid clever, chatty, or promotional copy in task-heavy flows
- If a section is clear without extra text, do not add filler

Use concise UX writing rules in recommendations:
- Headings should identify the task, not explain the whole screen
- CTA text should describe the result
- Warning and confirmation text should answer the user's likely risk in one pass
- Empty states should point to the next useful action, not just describe absence
- Success states should confirm what happened and what users can do next

## UX Writing Best Practices

| Goal | Rule |
|---|---|
| Clarity | Use simple, everyday language; avoid jargon; be specific; front-load important information |
| Action-Oriented | Use verbs for buttons ("Save Changes" not "OK"); tell users what will happen; make CTAs clear and distinct |
| Helpful | Explain why you're asking for information; provide error messages with solutions; guide through complex tasks; offer examples and defaults |
| Concise | Remove unnecessary words; one idea per sentence; short paragraphs; scannable content |
