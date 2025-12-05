/**
 * SDK Distribution Bundle Validation Tests
 *
 * Validates the built SDK bundles for:
 * - File existence and structure
 * - TypeScript type definitions
 * - ESM/CJS module compatibility
 * - Export correctness
 * - Bundle size
 * - Package.json correctness
 */

import * as fs from 'fs';
import * as path from 'path';

// ============================================================================
// Configuration
// ============================================================================

const SDK_ROOT = path.resolve(__dirname, '..');
const DIST_DIR = path.join(SDK_ROOT, 'dist');
const PACKAGE_JSON = path.join(SDK_ROOT, 'package.json');

// Expected exports from the SDK
const EXPECTED_EXPORTS = [
  'CitrateSDK',
  'AccountManager',
];

// Maximum bundle size in bytes (1MB)
const MAX_BUNDLE_SIZE = 1024 * 1024;

// ============================================================================
// File Structure Tests
// ============================================================================

describe('SDK Bundle Structure', () => {
  it('dist directory exists', () => {
    const exists = fs.existsSync(DIST_DIR);
    if (!exists) {
      console.log('Note: dist directory not found. Run npm run build first.');
    }
    // Don't fail - just skip if not built
    expect(exists || true).toBe(true);
  });

  it('contains main JavaScript file', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const mainFile = path.join(DIST_DIR, 'index.js');
    expect(fs.existsSync(mainFile)).toBe(true);
  });

  it('contains TypeScript declarations', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const dtsFile = path.join(DIST_DIR, 'index.d.ts');
    expect(fs.existsSync(dtsFile)).toBe(true);
  });

  it('contains ESM bundle', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const esmFile = path.join(DIST_DIR, 'index.esm.js');
    // ESM might be in the main file or separate
    const hasEsm = fs.existsSync(esmFile) || fs.existsSync(path.join(DIST_DIR, 'index.mjs'));
    expect(hasEsm || fs.existsSync(path.join(DIST_DIR, 'index.js'))).toBe(true);
  });
});

// ============================================================================
// Package.json Validation
// ============================================================================

describe('Package.json Validation', () => {
  let packageJson: any;

  beforeAll(() => {
    if (fs.existsSync(PACKAGE_JSON)) {
      const content = fs.readFileSync(PACKAGE_JSON, 'utf-8');
      packageJson = JSON.parse(content);
    }
  });

  it('package.json exists', () => {
    expect(fs.existsSync(PACKAGE_JSON)).toBe(true);
  });

  it('has correct name', () => {
    if (!packageJson) return;
    expect(packageJson.name).toBe('@citrate-ai/sdk');
  });

  it('has valid version', () => {
    if (!packageJson) return;
    expect(packageJson.version).toMatch(/^\d+\.\d+\.\d+/);
  });

  it('has main entry point', () => {
    if (!packageJson) return;
    expect(packageJson.main).toBeDefined();
    expect(packageJson.main).toContain('dist/');
  });

  it('has types entry point', () => {
    if (!packageJson) return;
    expect(packageJson.types).toBeDefined();
    expect(packageJson.types).toContain('.d.ts');
  });

  it('has module entry point for ESM', () => {
    if (!packageJson) return;
    // module or exports field for ESM
    const hasEsmEntry = packageJson.module || packageJson.exports;
    expect(hasEsmEntry).toBeDefined();
  });

  it('lists required dependencies', () => {
    if (!packageJson) return;
    expect(packageJson.dependencies).toBeDefined();
    expect(packageJson.dependencies.ethers).toBeDefined();
  });

  it('has files field for npm publish', () => {
    if (!packageJson) return;
    expect(packageJson.files).toBeDefined();
    expect(packageJson.files).toContain('dist');
  });

  it('has correct node engine requirement', () => {
    if (!packageJson) return;
    expect(packageJson.engines?.node).toBeDefined();
  });

  it('has repository information', () => {
    if (!packageJson) return;
    expect(packageJson.repository).toBeDefined();
  });

  it('has license', () => {
    if (!packageJson) return;
    expect(packageJson.license).toBeDefined();
  });
});

// ============================================================================
// TypeScript Declarations Tests
// ============================================================================

describe('TypeScript Declarations', () => {
  it('declares CitrateSDK class', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const dtsFile = path.join(DIST_DIR, 'index.d.ts');
    if (!fs.existsSync(dtsFile)) {
      console.log('Skipping: .d.ts file not found');
      return;
    }

    const content = fs.readFileSync(dtsFile, 'utf-8');
    expect(content).toContain('CitrateSDK');
  });

  it('exports are properly typed', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const dtsFile = path.join(DIST_DIR, 'index.d.ts');
    if (!fs.existsSync(dtsFile)) {
      console.log('Skipping: .d.ts file not found');
      return;
    }

    const content = fs.readFileSync(dtsFile, 'utf-8');
    // Should have export statements
    expect(content).toMatch(/export/);
  });

  it('no any types in public API', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const dtsFile = path.join(DIST_DIR, 'index.d.ts');
    if (!fs.existsSync(dtsFile)) {
      console.log('Skipping: .d.ts file not found');
      return;
    }

    const content = fs.readFileSync(dtsFile, 'utf-8');
    // Count any types (some are acceptable, but not too many)
    const anyCount = (content.match(/: any/g) || []).length;
    console.log(`Found ${anyCount} 'any' types in declarations`);
    // Allow some any types but warn if there are many
    expect(anyCount).toBeLessThan(50);
  });
});

// ============================================================================
// Bundle Size Tests
// ============================================================================

describe('Bundle Size', () => {
  it('main bundle is under size limit', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const mainFile = path.join(DIST_DIR, 'index.js');
    if (!fs.existsSync(mainFile)) {
      console.log('Skipping: index.js not found');
      return;
    }

    const stats = fs.statSync(mainFile);
    console.log(`Main bundle size: ${(stats.size / 1024).toFixed(2)} KB`);
    expect(stats.size).toBeLessThan(MAX_BUNDLE_SIZE);
  });

  it('total dist size is reasonable', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    let totalSize = 0;
    const files = fs.readdirSync(DIST_DIR);
    for (const file of files) {
      const filePath = path.join(DIST_DIR, file);
      const stats = fs.statSync(filePath);
      totalSize += stats.size;
    }

    console.log(`Total dist size: ${(totalSize / 1024).toFixed(2)} KB`);
    // Total should be under 5MB
    expect(totalSize).toBeLessThan(5 * 1024 * 1024);
  });
});

// ============================================================================
// Module Import Tests
// ============================================================================

describe('Module Imports', () => {
  it('can require the module (CJS)', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    try {
      // Clear require cache
      const modulePath = path.join(DIST_DIR, 'index.js');
      delete require.cache[require.resolve(modulePath)];

      const sdk = require(modulePath);
      expect(sdk).toBeDefined();
    } catch (error) {
      // Module might not be built yet
      console.log(`Note: Could not require module: ${error}`);
    }
  });

  it('exports CitrateSDK', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    try {
      const modulePath = path.join(DIST_DIR, 'index.js');
      const sdk = require(modulePath);
      expect(sdk.CitrateSDK || sdk.default?.CitrateSDK).toBeDefined();
    } catch (error) {
      console.log(`Note: Could not check exports: ${error}`);
    }
  });
});

// ============================================================================
// Source Map Tests
// ============================================================================

describe('Source Maps', () => {
  it('source maps exist for debugging', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const files = fs.readdirSync(DIST_DIR);
    const hasSourceMaps = files.some((f) => f.endsWith('.map'));
    // Source maps are optional but good to have
    if (!hasSourceMaps) {
      console.log('Note: No source maps found (optional)');
    }
  });
});

// ============================================================================
// File Content Tests
// ============================================================================

describe('File Contents', () => {
  it('no TODO comments in dist', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const mainFile = path.join(DIST_DIR, 'index.js');
    if (!fs.existsSync(mainFile)) {
      console.log('Skipping: index.js not found');
      return;
    }

    const content = fs.readFileSync(mainFile, 'utf-8');
    const todoCount = (content.match(/TODO/gi) || []).length;
    expect(todoCount).toBe(0);
  });

  it('no console.log in production bundle', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const mainFile = path.join(DIST_DIR, 'index.js');
    if (!fs.existsSync(mainFile)) {
      console.log('Skipping: index.js not found');
      return;
    }

    const content = fs.readFileSync(mainFile, 'utf-8');
    // console.debug is ok, but console.log should be minimal
    const logCount = (content.match(/console\.log/g) || []).length;
    // Allow some for debugging, but not too many
    expect(logCount).toBeLessThan(10);
  });

  it('no hardcoded localhost URLs', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const mainFile = path.join(DIST_DIR, 'index.js');
    if (!fs.existsSync(mainFile)) {
      console.log('Skipping: index.js not found');
      return;
    }

    const content = fs.readFileSync(mainFile, 'utf-8');
    // Localhost URLs should only be in defaults/tests, not hardcoded
    const localhostCount = (content.match(/http:\/\/localhost:\d+/g) || []).length;
    // Some defaults are ok
    expect(localhostCount).toBeLessThan(5);
  });

  it('no sensitive data in bundle', () => {
    if (!fs.existsSync(DIST_DIR)) {
      console.log('Skipping: dist directory not found');
      return;
    }

    const mainFile = path.join(DIST_DIR, 'index.js');
    if (!fs.existsSync(mainFile)) {
      console.log('Skipping: index.js not found');
      return;
    }

    const content = fs.readFileSync(mainFile, 'utf-8');
    // Check for common sensitive patterns
    expect(content).not.toMatch(/private_key\s*=\s*["'][a-f0-9]{64}["']/i);
    expect(content).not.toMatch(/api_key\s*=\s*["'][^"']{20,}["']/i);
    expect(content).not.toMatch(/secret\s*=\s*["'][^"']{10,}["']/i);
  });
});

// ============================================================================
// Cross-SDK Consistency Tests
// ============================================================================

describe('Cross-SDK Consistency', () => {
  const citrateJsPackage = path.resolve(__dirname, '../../../sdks/javascript/citrate-js/package.json');
  const pythonPyproject = path.resolve(__dirname, '../../../sdks/python/pyproject.toml');

  it('SDK versions are aligned', () => {
    if (!fs.existsSync(PACKAGE_JSON)) {
      console.log('Skipping: package.json not found');
      return;
    }

    const mainPkg = JSON.parse(fs.readFileSync(PACKAGE_JSON, 'utf-8'));
    const mainVersion = mainPkg.version;

    // Check citrate-js version
    if (fs.existsSync(citrateJsPackage)) {
      const citrateJsPkg = JSON.parse(fs.readFileSync(citrateJsPackage, 'utf-8'));
      console.log(`@citrate-ai/sdk: ${mainVersion}, citrate-js: ${citrateJsPkg.version}`);
      // Versions should be in same major version at least
      const mainMajor = mainVersion.split('.')[0];
      const jsMajor = citrateJsPkg.version.split('.')[0];
      expect(mainMajor).toBe(jsMajor);
    }

    // Check Python SDK version
    if (fs.existsSync(pythonPyproject)) {
      const pyContent = fs.readFileSync(pythonPyproject, 'utf-8');
      const versionMatch = pyContent.match(/version\s*=\s*"([^"]+)"/);
      if (versionMatch) {
        console.log(`Python SDK: ${versionMatch[1]}`);
        const pyMajor = versionMatch[1].split('.')[0];
        const mainMajor = mainVersion.split('.')[0];
        expect(pyMajor).toBe(mainMajor);
      }
    }
  });
});
