# Web Accessibility Audit

An audit of web content, run on 2026-11-19. Read the executive summary for the verdict, the priorities section for what to do first, and the findings tables for detail.

## Metadata

What was audited, with what, and under which settings.

| Field | Value |
|---|---|
| Date | 2026-11-19 |
| Scope | 3 items |
| Skills run | keyboard-navigator |
| Tooling | node v24.15.0, a11y-mcp-server 7.1.0 |
| Scan configuration | defaults, no config file found |
| Checks counted | 10 |

Audited:

- `native/civ/mixins/civ/civ-horizon/src/ui/map.rs`
- `native/civ/mixins/civ/civ-horizon/src/ui/mod.rs`
- `native/civ/mixins/civ/glue.rs`

## Executive summary

Pass. No critical or serious findings.

The headline numbers for this run.

| Measure | Value |
|---|---|
| Score | 97 out of 100 |
| Grade | A |
| Findings | 1 |
| Blocking (critical plus serious) | 0 |
| Verdict | Pass |

## Severity breakdown

Counts by severity, highest impact first.

| Severity | Count | What it means |
|---|---:|---|
| critical | 0 | Blocks a person from completing the task |
| serious | 0 | Major barrier with a difficult workaround |
| moderate | 1 | Real friction, workaround exists |
| minor | 0 | Noticeable but rarely blocking |

## Findings

1 findings, ordered by severity then by rule.

### Moderate (1)

Each row names the rule, the criterion it fails, where it is, and what to change.

| Rule | WCAG | Location | Problem | Fix |
|---|---|---|---|---|
| `KBD-SHORTCUT-001` | 2.1.4 A | native/civ/mixins/civ/glue.rs:screen_key | Map pan and zoom shortcuts cannot currently be disabled or remapped while the Carte tab is active. | Consider an explicit map-control focus or preference that enables, disables, or remaps the arrow and zoom shortcuts. |

## Remediation priorities

The remaining 1 moderate and minor findings can follow in normal work.

## What passed

5 checks ran and found nothing.

- Auto-fit, resize, pan, and zoom keep a focused province visible.
- Pan and zoom shortcuts are scoped to the Carte tab and documented in the view.
- Tab cycles province focus; Enter and Space activate the focused province.
- Tests cover map fitting, clipping, focus, selection, and polygon rendering.
- The focus outline is rendered independently of hover and selection.

## Needs a human

Items a skill could not decide, or could not check automatically.

- **keyboard-navigator**: Keyboard behavior was reviewed in source and tests; no native Horizon build or in-game interaction was performed.

## Next steps

1. Keep the current checks in continuous integration so this does not regress.
2. Re-audit when the interface changes materially, or quarterly.
3. Confirm the automated result with a manual screen reader and keyboard pass.

Automated checks find a minority of accessibility barriers. A passing score is a floor, not a guarantee.

## Findings data

The merged findings, embedded so the next run can compute a delta without parsing this document.

```json a11y-findings
{
  "skills": [
    "keyboard-navigator"
  ],
  "scope": [
    "native/civ/mixins/civ/civ-horizon/src/ui/map.rs",
    "native/civ/mixins/civ/civ-horizon/src/ui/mod.rs",
    "native/civ/mixins/civ/glue.rs"
  ],
  "findings": [
    {
      "rule": "KBD-SHORTCUT-001",
      "wcag": "2.1.4 A",
      "severity": "moderate",
      "confidence": "medium",
      "location": "native/civ/mixins/civ/glue.rs:screen_key",
      "summary": "Map pan and zoom shortcuts cannot currently be disabled or remapped while the Carte tab is active.",
      "fix": "Consider an explicit map-control focus or preference that enables, disables, or remaps the arrow and zoom shortcuts.",
      "evidence": "The shortcuts are now limited to the Carte tab and are listed in the map's on-screen keyboard hint, but no disable or remap option exists.",
      "foundBy": [
        "keyboard-navigator"
      ]
    }
  ],
  "passed": [
    "Auto-fit, resize, pan, and zoom keep a focused province visible.",
    "Pan and zoom shortcuts are scoped to the Carte tab and documented in the view.",
    "Tab cycles province focus; Enter and Space activate the focused province.",
    "Tests cover map fitting, clipping, focus, selection, and polygon rendering.",
    "The focus outline is rendered independently of hover and selection."
  ],
  "notes": [
    {
      "skill": "keyboard-navigator",
      "note": "Keyboard behavior was reviewed in source and tests; no native Horizon build or in-game interaction was performed."
    }
  ],
  "counts": {
    "critical": 0,
    "serious": 0,
    "moderate": 1,
    "minor": 0
  },
  "components": [],
  "score": 97,
  "grade": "A",
  "penalty": 1.4,
  "checks": 10,
  "generated": "2026-11-19",
  "template": "web"
}
```
