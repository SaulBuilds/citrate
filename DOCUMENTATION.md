# DOCUMENTATION.md

## Documentation Governance Protocol

This document defines the **single source of truth** for all Citrate documentation and establishes protocols to prevent documentation sprawl, confusion, and duplicate sources of truth.

**Last Updated**: October 26, 2025
**Status**: Active Governance Document

---

## 🎯 Core Principles

### 1. **One Source of Truth Per Topic**
Each topic has **exactly ONE** authoritative document. All other references must **link** to this source, not duplicate content.

### 2. **Clear Document Lifecycle**
```
Active Development → Current (top-level)
Completed/Historical → Archive (with date)
Outdated/Superseded → Delete (after 30-day archive)
```

### 3. **Mandatory Review Before Creation**
Before creating any new documentation file, check:
- ✅ Does this topic already have a source of truth?
- ✅ If yes, should I update the existing doc or create a new section?
- ✅ If no, am I the right person to establish this source of truth?

---

## 📚 Documentation Matrix - Single Sources of Truth

### **Technical Architecture & Design**
| Topic | Source of Truth | Location | Owner |
|-------|----------------|----------|-------|
| AI Assistant Context | `CLAUDE.md` | `/CLAUDE.md` | Engineering |
| Overall Architecture | `citrate/README.md` | `/citrate/README.md` | Engineering |
| Current Roadmap | `roadmap-p0.md` | `/citrate/docs/roadmap-p0.md` | Product/Eng |
| Transaction Pipeline | `CLAUDE.md` §Transaction Pipeline | `/CLAUDE.md` | Engineering |
| Genesis Model | `genesis-model.md` | `/citrate/docs/technical/genesis-model.md` | Engineering |

### **User & Operator Documentation**
| Topic | Source of Truth | Location | Owner |
|-------|----------------|----------|-------|
| Installation | `installation.md` | `/citrate/docs/guides/installation.md` | DevRel |
| Deployment | `deployment.md` | `/citrate/docs/guides/deployment.md` | DevOps |
| Quick Start | `DEVNET_QUICKSTART.md` | `/DEVNET_QUICKSTART.md` | DevRel |
| Genesis Setup | `genesis-startup.md` | `/citrate/docs/guides/genesis-startup.md` | DevOps |
| Wallet & Rewards | `wallet-and-rewards.md` | `/citrate/docs/guides/wallet-and-rewards.md` | Product |
| Structured Docs | docs-portal/docs/ | `/docs-portal/docs/` | DevRel |

### **Developer Documentation**
| Topic | Source of Truth | Location | Owner |
|-------|----------------|----------|-------|
| SDK (JavaScript) | `sdks/javascript/README.md` | `/citrate/sdks/javascript/citrate-js/README.md` | SDK Team |
| SDK (Python) | `sdks/python/README.md` | `/citrate/sdks/python/README.md` | SDK Team |
| RPC API | docs-portal/docs/developers/rpc.md | `/docs-portal/docs/developers/rpc.md` | API Team |
| Smart Contracts | `contracts/README.md` | `/citrate/contracts/README.md` | Contracts Team |
| Contributing | `CONTRIBUTING.md` | `/CONTRIBUTING.md` | Community |

### **Project Governance**
| Topic | Source of Truth | Location | Owner |
|-------|----------------|----------|-------|
| Code of Conduct | `CODE_OF_CONDUCT.md` | `/CODE_OF_CONDUCT.md` | Community |
| Documentation Protocol | `DOCUMENTATION.md` | `/DOCUMENTATION.md` | Engineering |
| Changelog | `CHANGELOG.md` | `/citrate/CHANGELOG.md` | Engineering |
| Whitepaper | `lattice-whitepaper-final.md` | `/citrate-docs-v3/lattice-whitepaper-final.md` | Leadership |

---

## 📋 Document Naming Conventions

### Standard Names (Use Exactly These)
```
README.md                              - Project/component overview
CONTRIBUTING.md                        - Contribution guidelines
CHANGELOG.md                           - Version history
CLAUDE.md                              - AI assistant context
DOCUMENTATION.md                       - This file
docs/guides/deployment.md              - Deployment guide
docs/guides/installation.md            - Installation instructions
docs/guides/genesis-startup.md         - Genesis node setup
docs/guides/wallet-and-rewards.md      - Wallet guide
docs/roadmap-p0.md                     - Current P0 roadmap
docs/technical/genesis-model.md        - Technical deep-dives
```

### Prohibited Names (Archive Instead)
```
❌ *_PROGRESS.md        → archive/phase-history/
❌ *_COMPLETION.md      → archive/phase-history/
❌ *_SUMMARY.md         → archive/phase-history/
❌ *_REPORT.md          → archive/testing/ or archive/audits/
❌ *_PLAN.md            → archive/implementations/
❌ *_GUIDE_v2.md        → Use version control, not filename versions
```

---

## 🗂️ Archive Structure

All historical/completed documentation must be moved to `/archive/` with the following structure:

```
archive/
├── audits/              # Dated audit reports
│   └── YYYY-MM-audit-name.md
├── deployment-guides/   # Old deployment docs
├── gui-docs/            # GUI-specific archived docs
├── implementations/     # Implementation plans
├── phase-history/       # Phase completion reports
├── roadmaps/            # Superseded roadmaps (dated)
│   └── YYYY-MM-roadmap-name.md
├── testing/             # Test reports
└── whitepapers/         # Old whitepaper versions
```

### Archive Naming Convention
```
YYYY-MM-descriptive-name.md
```

**Examples**:
- `2025-03-comprehensive-audit.md`
- `2024-10-global-roadmap.md`
- `2025-01-phase3-completion.md`

---

## ✅ Documentation Creation Checklist

Before creating **any** new documentation file:

1. ✅ **Check Documentation Matrix** - Does this topic already have a source of truth?
2. ✅ **Check Archive** - Is there a historical version I should review?
3. ✅ **Consult CLAUDE.md** - Does this architectural info belong there?
4. ✅ **Consider docs-portal** - Should this be user-facing structured docs?
5. ✅ **Review Naming** - Am I using a standard approved name?
6. ✅ **Plan Lifecycle** - When will this doc be archived? Who will maintain it?

**If in doubt**: Ask in #documentation channel or consult the Engineering Lead.

---

## 🔄 Document Update Protocol

### When Updating an Existing Document
1. Update the "Last Updated" date at the top
2. If it's a major change, note it in `CHANGELOG.md`
3. If superseding entire sections, consider moving old content to archive

### When Information Becomes Historical
1. Move file to appropriate `/archive/` subdirectory
2. Rename with date: `YYYY-MM-original-name.md`
3. Update any links in other docs to point to archive or new source
4. Add entry to `CHANGELOG.md` noting the archival

### When Merging/Consolidating Docs
1. Identify the **most comprehensive** document as the source of truth
2. Merge relevant content from others into it
3. Archive all other versions
4. Update Documentation Matrix in this file
5. Search codebase for links to old docs and update them

---

## 🚫 Anti-Patterns to Avoid

### ❌ Creating "Version 2" Files
**Wrong**: `DEPLOYMENT_GUIDE_v2.md`
**Right**: Update `DEPLOYMENT.md` via git (version control handles versions)

### ❌ Duplicating Content Across Files
**Wrong**: Copying deployment steps into README, QUICKSTART, and DEPLOYMENT.md
**Right**: Have one detailed guide, others **link** to it

### ❌ Creating Progress/Status Docs for Active Work
**Wrong**: Creating `PHASE5_PROGRESS.md` to track ongoing work
**Right**: Use GitHub Issues, Projects, or update `ROADMAP_P0.md` with checkboxes

### ❌ Leaving Obsolete Docs in Root
**Wrong**: Keeping `FINAL_TEST_REPORT.md` in root after testing is complete
**Right**: Archive it to `archive/testing/2025-03-final-test-report.md`

### ❌ Creating README Files Everywhere
**Wrong**: README.md in every subdirectory duplicating parent README
**Right**: README only where it provides unique value for that directory

---

## 🔍 Audit Schedule

**Quarterly Documentation Audit** (Q1, Q2, Q3, Q4):
1. Review all root-level `.md` files
2. Identify candidates for archival
3. Verify Documentation Matrix is current
4. Check for duplicate content
5. Update this governance document

**Responsible**: Engineering Lead + DevRel

---

## 📞 Questions & Enforcement

### Questions About This Protocol
- Open an issue with label `documentation`
- Ask in #engineering or #documentation channels
- Consult Engineering Lead

### Protocol Violations
If you find docs that violate this protocol:
1. Open a PR to fix it
2. Tag @engineering-lead for review
3. Update this document if clarification is needed

### Exceptions
Exceptions to this protocol require:
- Written justification in PR description
- Approval from Engineering Lead
- Documentation of exception in this file

---

## 📝 Changelog

| Date | Change | Author |
|------|--------|--------|
| 2025-10-26 | Initial documentation governance protocol created | Claude Code |

---

**Remember**: Good documentation is about **finding**, not **writing**. Every new file makes finding harder. Default to updating, not creating.
