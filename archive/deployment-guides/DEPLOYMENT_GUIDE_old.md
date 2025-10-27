# 🚀 Citrate Deployment & Publishing Guide

This guide covers publishing the Citrate SDKs and deploying the developer tools frontends to production.

## 📦 SDK Publishing

### Prerequisites
- Python 3.8+ with pip
- Node.js 16+ with npm
- PyPI account with API token
- NPM account with publish permissions

### 1. **Python SDK (PyPI)**

```bash
# Install build tools
pip install build twine

# Navigate to Python SDK
cd sdks/python

# Build and publish
python -m build
twine check dist/*
twine upload dist/*
```

**Automated Publishing:**
```bash
# Use the automated script
./scripts/publish-sdks.sh python
```

### 2. **JavaScript SDK (NPM)**

```bash
# Navigate to JavaScript SDK
cd sdk/javascript

# Install dependencies and build
npm install
npm run build

# Publish
npm publish --access public
```

**Automated Publishing:**
```bash
# Use the automated script
./scripts/publish-sdks.sh javascript
```

### 3. **Publish Both SDKs**

```bash
# Publish both SDKs with confirmation prompts
./scripts/publish-sdks.sh both

# For test publishing
./scripts/publish-sdks.sh both --test
```

---

## 🌐 Frontend Deployment (Vercel)

### Prerequisites
- Vercel account
- Vercel CLI installed: `npm install -g vercel`
- Domain names configured (optional)

### 1. **Setup Vercel**

```bash
# Login to Vercel
vercel login

# Link projects (run in each frontend directory)
cd developer-tools/lattice-studio
vercel link

cd ../documentation-portal
vercel link

cd ../debug-dashboard
vercel link
```

### 2. **Environment Variables**

Configure these environment variables in Vercel dashboard:

#### Citrate Studio
- `REACT_APP_LATTICE_RPC`: https://rpc.lattice.ai
- `REACT_APP_IPFS_URL`: https://ipfs.lattice.ai
- `REACT_APP_WEBSOCKET_URL`: wss://ws.lattice.ai

#### Documentation Portal
- `NEXT_PUBLIC_LATTICE_RPC`: https://rpc.lattice.ai
- `NEXT_PUBLIC_IPFS_URL`: https://ipfs.lattice.ai

#### Debug Dashboard
- `VITE_LATTICE_RPC`: https://rpc.lattice.ai
- `VITE_WEBSOCKET_URL`: wss://ws.lattice.ai

### 3. **Deploy Frontends**

```bash
# Deploy all frontends
./scripts/deploy-frontends.sh

# Deploy individual frontends
./scripts/deploy-frontends.sh studio
./scripts/deploy-frontends.sh docs
./scripts/deploy-frontends.sh dashboard
```

### 4. **Custom Domains** (Optional)

1. Add domains in Vercel dashboard
2. Configure DNS records:
   - `studio.lattice.ai` → Citrate Studio
   - `docs.lattice.ai` → Documentation Portal
   - `debug.lattice.ai` → Debug Dashboard

---

## 🔄 CI/CD Pipeline

### GitHub Actions Setup

Both SDKs include GitHub Actions workflows for automated publishing:

#### Python SDK
- **Workflow**: `.github/workflows/publish.yml`
- **Trigger**: Release creation or manual dispatch
- **Tests**: Python 3.8-3.12 matrix
- **Publishing**: PyPI with API token

#### JavaScript SDK
- **Workflow**: `.github/workflows/publish.yml`
- **Trigger**: Release creation or manual dispatch
- **Tests**: Node.js 16, 18, 20 matrix
- **Publishing**: NPM with access token

### Secrets Configuration

Add these secrets to your GitHub repository:

```bash
# Python SDK
PYPI_API_TOKEN=pypi-...

# JavaScript SDK
NPM_TOKEN=npm_...
```

---

## 📋 Pre-Deployment Checklist

### SDKs
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Version numbers bumped
- [ ] Changelog entries added
- [ ] No sensitive data in package

### Frontends
- [ ] Environment variables configured
- [ ] Build process successful
- [ ] No API keys in client-side code
- [ ] Performance optimized
- [ ] SEO metadata configured

---

## 🚦 Deployment Commands Quick Reference

```bash
# SDK Publishing
./scripts/publish-sdks.sh both          # Publish both SDKs
./scripts/publish-sdks.sh python        # Python only
./scripts/publish-sdks.sh javascript    # JavaScript only
./scripts/publish-sdks.sh both --test   # Test repositories

# Frontend Deployment
./scripts/deploy-frontends.sh           # Deploy all frontends
./scripts/deploy-frontends.sh studio    # Studio only
./scripts/deploy-frontends.sh docs      # Docs only
./scripts/deploy-frontends.sh dashboard # Dashboard only

# Verification
npm view @lattice-ai/sdk                # Check NPM package
pip show lattice-sdk                     # Check PyPI package
vercel ls                                # List Vercel deployments
```

---

## 🛠️ Troubleshooting

### Common Issues

**SDK Publishing Fails:**
- Verify API tokens are correct
- Check version numbers aren't already published
- Ensure package names are available

**Vercel Deployment Fails:**
- Check build logs for errors
- Verify environment variables are set
- Ensure dependencies are properly installed

**Build Errors:**
- Clear node_modules and reinstall
- Check for TypeScript errors
- Verify all environment variables are present

### Getting Help

- **SDK Issues**: Check package-specific README files
- **Deployment Issues**: Refer to Vercel documentation
- **General Issues**: Create GitHub issue with logs

---

## 🎯 Post-Deployment

After successful deployment:

1. **Test all endpoints** and functionality
2. **Update documentation** with new URLs
3. **Notify community** about releases
4. **Monitor logs** for any issues
5. **Set up monitoring** and alerts

---

*This completes the deployment setup for Citrate SDKs and developer tools. All components are now ready for production use.*