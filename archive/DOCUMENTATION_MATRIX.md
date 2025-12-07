# Documentation Matrix - Quick Reference

> **For full governance protocol, see [DOCUMENTATION.md](./DOCUMENTATION.md)**

## 🎯 Single Sources of Truth

**Last Updated**: October 26, 2025

This matrix provides a quick reference for finding the authoritative document on any topic. **Never create duplicate documents** - always link to the source of truth.

---

## Quick Lookup Table

| What You Need | Authoritative Document | Path |
|---------------|------------------------|------|
| **AI Assistant Context** | CLAUDE.md | `/CLAUDE.md` |
| **Project Overview** | citrate/README.md | `/citrate/README.md` |
| **Quick Start Guide** | DEVNET_QUICKSTART.md | `/DEVNET_QUICKSTART.md` |
| **Installation** | installation.md | `/citrate/docs/guides/installation.md` |
| **Deployment** | deployment.md | `/citrate/docs/guides/deployment.md` |
| **Current Roadmap (P0)** | roadmap-p0.md | `/citrate/docs/roadmap-p0.md` |
| **Contributing** | CONTRIBUTING.md | `/CONTRIBUTING.md` |
| **Code of Conduct** | CODE_OF_CONDUCT.md | `/CODE_OF_CONDUCT.md` |
| **Whitepaper** | lattice-whitepaper-final.md | `/citrate-docs-v3/lattice-whitepaper-final.md` |
| **Changelog** | CHANGELOG.md | `/citrate/CHANGELOG.md` |
| **Genesis Setup** | genesis-startup.md | `/citrate/docs/guides/genesis-startup.md` |
| **Wallet Guide** | wallet-and-rewards.md | `/citrate/docs/guides/wallet-and-rewards.md` |
| **Transaction Pipeline** | CLAUDE.md §Transaction Pipeline | `/CLAUDE.md` (section) |
| **Network Modes** | NETWORK_MODES.md | `/docs/NETWORK_MODES.md` |
| **Genesis Model** | genesis-model.md | `/citrate/docs/technical/genesis-model.md` |

## SDK & API Documentation

| Resource | Location |
|----------|----------|
| **JavaScript SDK** | `/citrate/sdks/javascript/citrate-js/README.md` |
| **Python SDK** | `/citrate/sdks/python/README.md` |
| **RPC API Reference** | `/docs-portal/docs/developers/rpc.md` |
| **Smart Contracts** | `/citrate/contracts/README.md` |
| **Testing Guide** | `/docs-portal/docs/developers/testing.md` |

## Structured User Documentation

All comprehensive user documentation lives in:

```
/docs-portal/docs/
├── developers/    # SDK, RPC, testing, contracts
├── operators/     # Node operations, monitoring
├── providers/     # Model registration, pricing
├── users/         # Wallet, rewards, getting started
├── overview/      # Architecture, vision, tokenomics
└── security/      # Threat model, audits
```

---

## 🗂️ Where Historical Docs Live

| Type of Doc | Archive Location |
|-------------|------------------|
| Phase Progress Reports | `/archive/phase-history/` |
| Test Reports | `/archive/testing/` |
| Audit Reports | `/archive/audits/` |
| Old Roadmaps | `/archive/roadmaps/` |
| Implementation Plans | `/archive/implementations/` |
| Old Deployment Guides | `/archive/deployment-guides/` |
| GUI Historical Docs | `/archive/gui-docs/` |
| Old Whitepapers | `/archive/whitepapers/` |

---

## 🚨 Quick Rules

1. **One topic = One document** - No duplicates
2. **Link, don't copy** - Reference the source of truth
3. **Archive, don't delete** - Historical docs go to `/archive/` with dates
4. **Update in place** - Use git for versions, not `_v2.md` files
5. **Check matrix first** - Before creating any new documentation

---

## 🔄 When to Update This Matrix

- After creating a new source of truth document
- After archiving an old document
- After consolidating multiple docs into one
- During quarterly documentation audits

**How to update**:
1. Edit this file
2. Edit the full table in [DOCUMENTATION.md](./DOCUMENTATION.md)
3. Commit both in same PR

---

## ❓ Can't Find What You Need?

1. **Search the archive**: `/archive/` subdirectories
2. **Check docs-portal**: `/docs-portal/docs/` (organized by role)
3. **Ask in #documentation**: Engineering or DevRel team
4. **Read CLAUDE.md**: Often has detailed technical context

---

**Remember**: If you're about to create a new .md file, check this matrix first! 🎯
