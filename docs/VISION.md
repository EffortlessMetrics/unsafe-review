# Vision

The end state is a low-cost unsafe review layer between `cargo-geiger` and Miri.

```text
cargo-geiger:
  where is unsafe?

unsafe-review:
  what unsafe contract changed, what evidence is missing, and which witness should run?

Miri / careful / sanitizer / Loom / Kani / Crux:
  does a concrete execution or harness validate the changed seam?
```

`unsafe-review` should become the common front panel for unsafe Rust review:

- PR summaries show only changed, actionable unsafe review cards.
- LSP hovers explain the contract and missing evidence at the unsafe seam.
- Agent packets constrain LLMs to one bounded repair and one verify command.
- Repo badges track open unsafe-review gaps without pretending the repo is safe.
- Witness receipts make Miri/sanitizer/Loom/Kani work visible and scoped.
