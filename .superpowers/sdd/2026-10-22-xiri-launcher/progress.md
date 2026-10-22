# SDD ledger — plan: docs/superpowers/plans/2026-10-22-xiri-launcher.md

Execution approved: direct, 2026-10-22. Base ee2d72a; branch feat/xiri-launcher.
Ruling: use existing dedicated clean Xiri checkout and feature branch — it was cloned exclusively for this task and contains no user product edits; avoids a redundant checkout.
Ruling: prepare portable GNU Rust under D:/IA/.tools — no installed toolchain found; no PATH/system changes, user authorized implementation and testing.
Pre-flight: tasks 1/2/3/5 share catalog records, defined in core. Tasks 1/4 share package contract. Tasks 4/5 share validated installed state. Task 6 consumes all preceding crates.
Ruling: retain upstream legacy models/modules for regression tests and add owned Xiri AppInfo model consumed by UI and XiriBackend — avoids changing working platform installers that Xiri does not use. No legacy catalog or feed will be used by the shipped Xiri app. Cost: legacy source remains for provenance and tests.
Task 1: started; manifest and catalog tests written before implementation.
