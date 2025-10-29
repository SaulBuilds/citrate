# Sprint 3: File Changes Tracking

This document tracks all files created, modified, or deleted during Sprint 3.

---

## Files to Create

### Components (9 files)
- [ ] `gui/citrate-core/src/components/Contracts.tsx` - Main contracts tab with sub-navigation
- [ ] `gui/citrate-core/src/components/ContractEditor.tsx` - Monaco editor for Solidity code
- [ ] `gui/citrate-core/src/components/ContractDeployer.tsx` - Contract deployment UI
- [ ] `gui/citrate-core/src/components/ContractInteraction.tsx` - Contract function calling UI
- [ ] `gui/citrate-core/src/components/FunctionCall.tsx` - Dynamic function parameter form
- [ ] `gui/citrate-core/src/components/EventLog.tsx` - Contract event viewer
- [ ] `gui/citrate-core/src/components/TransactionBuilder.tsx` - Transaction creation UI
- [ ] `gui/citrate-core/src/components/TransactionQueue.tsx` - Pending transaction tracker
- [ ] `gui/citrate-core/src/components/TransactionReceipt.tsx` - Receipt details display
- [ ] `gui/citrate-core/src/components/DAGFilters.tsx` - DAG filtering controls
- [ ] `gui/citrate-core/src/components/BlockDetails.tsx` - Block details sidebar
- [ ] `gui/citrate-core/src/components/ModelBrowser.tsx` - AI model registry browser
- [ ] `gui/citrate-core/src/components/InferenceRunner.tsx` - AI inference interface

### Utilities (7 files)
- [ ] `gui/citrate-core/src/utils/contractCompiler.ts` - Solidity compilation
- [ ] `gui/citrate-core/src/utils/contractDeployer.ts` - Contract deployment logic
- [ ] `gui/citrate-core/src/utils/abiParser.ts` - ABI parsing and function extraction
- [ ] `gui/citrate-core/src/utils/transactionManager.ts` - Transaction creation and signing
- [ ] `gui/citrate-core/src/utils/dagRenderer.ts` - DAG visualization optimizations
- [ ] `gui/citrate-core/src/utils/modelLoader.ts` - AI model loading and caching
- [ ] `gui/citrate-core/src/utils/gasEstimator.ts` - Gas estimation utilities

### Types (1 file)
- [ ] `gui/citrate-core/src/types/contracts.ts` - Contract-related TypeScript types

### Tauri Backend (2 files)
- [ ] `gui/citrate-core/src-tauri/src/websocket.rs` - WebSocket subscription handler
- [ ] `gui/citrate-core/src-tauri/src/compiler.rs` - Solidity compiler integration (optional)

---

## Files to Modify

### Core Application
- [ ] `gui/citrate-core/src/App.tsx` - Add Contracts tab to sidebar
- [ ] `gui/citrate-core/package.json` - Add new dependencies

### Existing Components
- [ ] `gui/citrate-core/src/components/Wallet.tsx` - Integrate transaction builder
- [ ] `gui/citrate-core/src/components/DAGVisualization.tsx` - Add WebSocket updates, filters, search
- [ ] `gui/citrate-core/src/components/Models.tsx` - Integrate model browser
- [ ] `gui/citrate-core/src/components/ChatBot.tsx` - Use inference runner

### Services
- [ ] `gui/citrate-core/src/services/tauri.ts` - Add new Tauri command wrappers

### Styles
- [ ] `gui/citrate-core/src/App.css` - Add styles for new components

---

## Dependencies to Add

```json
{
  "dependencies": {
    "@monaco-editor/react": "^4.6.0",
    "d3": "^7.9.0",
    "recharts": "^2.10.0"
  },
  "devDependencies": {
    "solc": "^0.8.24"
  }
}
```

---

## File Size Estimates

| File | Estimated Lines | Complexity |
|------|-----------------|------------|
| Contracts.tsx | ~200 | Medium |
| ContractEditor.tsx | ~150 | Low |
| ContractDeployer.tsx | ~300 | High |
| ContractInteraction.tsx | ~350 | High |
| TransactionBuilder.tsx | ~250 | Medium |
| TransactionQueue.tsx | ~200 | Medium |
| contractCompiler.ts | ~150 | High |
| abiParser.ts | ~200 | High |
| transactionManager.ts | ~180 | High |
| dagRenderer.ts | ~250 | Very High |

**Total Estimated New Code:** ~2,500 lines

---

## Deployment Checklist

### Before Merging
- [ ] All new files added to git
- [ ] No files contain hardcoded credentials
- [ ] All imports use relative paths correctly
- [ ] TypeScript compiles without errors
- [ ] ESLint passes
- [ ] Build succeeds
- [ ] All tests pass

### After Merging
- [ ] Tag release (v3.1.0)
- [ ] Update CHANGELOG.md
- [ ] Deploy to staging
- [ ] Smoke test on staging
- [ ] Deploy to production

---

## Rollback Plan

If critical issues arise:
1. Revert merge commit
2. Re-deploy previous version
3. Investigate issue
4. Fix in new branch
5. Re-test thoroughly
6. Deploy again

**Previous Stable Version:** v3.0.0 (after Sprint 2)

---

**Document Version:** 1.0
**Last Updated:** February 11, 2026
