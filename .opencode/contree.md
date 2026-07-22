# Directions

Eagerly use these skills to fulfil operator requests, where applicable:

- **change** — any behaviour change, before any code is discussed or written
- **tdd** — implementing behaviour, writing code, or writing tests
- **sync** — drift, gaps, staleness, or completeness
- **setup** — no test framework is configured or TEST_TREES.md is absent
- **workflow** — the full arc from idea to verified working software

# Rules

- **Decide, don't ask** — an obvious question is yours to answer, not the user's. Run the ladder before asking: consult these rules and the mental model first; if they don't settle it, use your own best judgment from the code in front of you; only escalate a consequential, genuinely under-determined choice that neither resolves.
- **Don't manufacture flags** — apply the same ladder to anything you'd flag, caveat, or surface "just in case": fix it if these rules or the mental model direct it; otherwise use your judgment; if neither makes it matter, stay silent rather than reporting it.
- **KISS** — complexity is bad; simplicity above almost all else
- **YAGNI** — don't future-proof; implement only what you need now
- **Subtract, don't add** — can this be achieved by simplification instead?
- **No fake code** — no skeletons, placeholders, or temporary implementations
- **Avoid indirection** — direct is better than conforming to arbitrary patterns
- **Data over control flow** — push variability into data so the hot path stays uniform.
- **Fail fast** — don't swallow errors; let the system fail when unexpected things happen
- **Avoid nullability** — make things required; don't program defensively
- **Explicit and expressive** — name for what things do, not how they're implemented
- **No comments** — descriptive tests and expressive code obviate comments; set expectations in test trees, enforce them in tests, express them in code
- **Composition over inheritance** — no `extends`; use hooks, functional utilities, component composition
- **Typing** — type everything; no `any`
- **Read docs** — use Context7 before using any library; don't guess API usage
- **Trees are the contract** — every expected behaviour and side effect goes in `## Test Trees`; every tree is verified by a test; every test drives the real implementation
- **Behaviour, not internals** — every tree describes what crosses its level's interface (inputs, outputs, side-effects), never the implementation inside
- **Debugging means a test gap** — before fixing, find the tree path that should have caught the bug, write the failing test, then fix the code
- **Hexagonal** — domain pure; I/O in adapters; dependencies point inward; each driven port ships with an in-memory twin
- **Functional first** — the outermost layer is real, max-validity functional testing; the floor and outside-in entry point is the **journey**
- **Outside-in** — start each capability from its outermost failing test; it pulls the inner layers into being
- **Layer completeness** — every layer owns complete coverage of its own behaviour; journey/functional coverage is never coverage of the layers beneath
- **One tree, one test file** — each tree reifies exactly one test file; the describe/it hierarchy mirrors the tree verbatim

# Working with the Mental Model and Test Trees

- Use the mental model's existing concepts, vocabulary, and decisions rather than inventing parallel ones.
- Preserve the mental model's invariants. If a task appears to require breaking one, surface the conflict rather than routing around it.
- If the mental model is wrong, incomplete, or misleading for this task, flag it rather than silently reshaping it through code.
- Treat test trees as the authoritative behaviour contract — do not diverge from them silently.
