# Design Systems, Components, and Tokens

## Design System Structure

Define layers clearly:

1. Foundations (color, type, spacing, elevation, motion)
2. Primitive components (button, input, icon, text)
3. Composite patterns (forms, navigation, tables, cards)
4. Page templates and domain-specific compositions

## Token Strategy

- Use semantic tokens first, raw tokens second.
- Support theme and brand variation through token inheritance.
- Keep token naming stable and behavior-focused.
- Version token changes and document migration impact.
- Keep explicit light/dark and high-contrast semantics where supported by product requirements.

## Component Quality Rules

- Define explicit states: default, hover, focus, active, disabled, error, loading.
- Define empty/overflow/error data states.
- Define accessibility requirements for each component.
- Define content constraints and responsive behavior.
- Define when a variation is a true variant vs a new component (to prevent noisy duplication).

## Reuse Decision Framework

Use this sequence before creating a new component:

1. Existing component supports required semantics and behavior -> reuse.
2. Existing component supports semantics but needs constrained visual/context variation -> create variant.
3. Existing component cannot safely cover behavior without harming shared use cases -> create specialized component.
4. Document rationale and ownership for any new component or variant.

## Governance and Change Management

- Require design and engineering review before promoting new patterns.
- Track adoption and deprecate obsolete components with migration guides.
- Prevent duplicate components with overlapping behavior.
- Maintain a single source of truth for canonical components.
- Audit component sprawl and remove stale variants on a regular cadence.

## Common UI Patterns

### Layout
| Pattern | Use |
|---|---|
| Container | Max-width wrapper with padding |
| Grid | Multi-column responsive layouts |
| Stack | Vertical spacing between elements |
| Cluster | Horizontal grouping with wrapping |
| Sidebar | Fixed/collapsible side navigation |

### Navigation
| Pattern | Use |
|---|---|
| Header | Logo, primary nav, user actions |
| Breadcrumbs | Show hierarchy, aid navigation |
| Tabs | Switch between related views |
| Pagination | Navigate large datasets |
| Menu | Dropdown/flyout for actions |

### Forms
| Pattern | Use |
|---|---|
| Input | Text, number, email with validation |
| Select | Dropdown for options |
| Checkbox/Radio | Multiple/single selection |
| Textarea | Multi-line text input |
| Validation | Inline errors, clear messaging |

### Feedback
| Pattern | Use |
|---|---|
| Toast/Snackbar | Temporary notifications |
| Modal | Focused task/confirmation |
| Alert | Important system messages |
| Progress | Loading states, progress bars |
| Empty States | Helpful guidance when no content |

## Design System Workflow

| Step | Action |
|---|---|
| 1. Audit | Check if component/pattern already exists; review tokens |
| 2. Design/Extend | Use existing tokens; create variants systematically; consider all states |
| 3. Implement | Build reusable components; use tokens consistently; include accessibility |
| 4. Test | Visual regression, accessibility audit, responsive testing, theme testing |
| 5. Document | Usage guidelines, props/API, examples, accessibility notes |
