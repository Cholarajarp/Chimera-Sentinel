# NOTICE — Chimera Sentinel

## Clean-Room Disclosure

**Chimera Sentinel** is a clean-room implementation created for the
**Google All Things Agentic Hackathon (2026)** — Fortified Enterprise Fleet Track.

### Prior Research

The Chimera project family includes **Chimera Vanguard**, a prior research prototype
created for OpenAI Build Week. Vanguard explored adversarial agent evaluation and
side-effect verification concepts in a different technology stack.

### What Was Reused

Sentinel reuses **architectural lessons and security concepts** learned from Vanguard:

- The idea of binding agent identity, model, tools, and policy into an immutable
  manifest before certification
- The concept of a deterministic business-logic oracle that does not trust model text
- Defense-in-depth layering (content safety → identity/policy → side-effect
  verification)
- Separation of AI-produced observations from trusted authorization decisions

### What Was NOT Reused

- **Zero lines of Vanguard source code** were copied, ported, or translated
- **No proprietary prompts, datasets, or model configurations** from Vanguard
- **No Vanguard infrastructure-as-code, Docker images, or CI/CD pipelines**
- **No Vanguard UI assets, design tokens, or frontend code**
- **No Vanguard test fixtures, evaluation corpora, or evidence schemas**

### Technology Boundary

| Aspect | Vanguard | Sentinel |
|---|---|---|
| Language | Python | Rust + Python (ADK only) |
| Agent Framework | Custom | Google ADK |
| Cloud Platform | Azure/OpenAI | Google Cloud |
| Decision Engine | Python | Deterministic Rust |
| Signing | Local keys | Cloud KMS |
| Identity | Shared credentials | Google Agent Identity |
| Content Safety | Custom filters | Google Model Armor |
| Tool Gateway | None | Google Agent Gateway |

All Sentinel code, schemas, configurations, test fixtures, infrastructure, and
documentation were authored from scratch during the hackathon eligibility period.

## Third-Party Licenses

This project uses open-source dependencies managed through:

- **Rust**: `Cargo.lock` — see individual crate licenses on [crates.io](https://crates.io)
- **Python**: `uv.lock` — see individual package licenses on [PyPI](https://pypi.org)
- **Node.js**: `pnpm-lock.yaml` — see individual package licenses on [npm](https://www.npmjs.com)

All dependencies are used under their respective open-source licenses
(MIT, Apache-2.0, BSD, ISC). No proprietary or restrictively licensed
dependencies are included.

## Copyright

Copyright 2026 Chimera Sentinel Contributors.
Licensed under the Apache License, Version 2.0.
