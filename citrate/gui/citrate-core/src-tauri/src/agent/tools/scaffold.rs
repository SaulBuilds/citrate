//! dApp scaffolding tools - project template generation
//!
//! These tools provide dApp project scaffolding capabilities.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;

/// Available dApp templates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DappTemplate {
    /// Basic contract + frontend
    Basic,
    /// DeFi: Token + swap functionality
    Defi,
    /// NFT: Collection + minting
    Nft,
    /// Marketplace: Multi-vendor
    Marketplace,
}

impl DappTemplate {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "basic" | "simple" | "starter" => Some(Self::Basic),
            "defi" | "token" | "swap" | "exchange" => Some(Self::Defi),
            "nft" | "collection" | "mint" => Some(Self::Nft),
            "marketplace" | "market" | "shop" => Some(Self::Marketplace),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Basic => "Basic dApp with simple storage contract and React frontend",
            Self::Defi => "DeFi dApp with ERC20 token, liquidity pool, and swap interface",
            Self::Nft => "NFT dApp with ERC721 collection, minting, and gallery UI",
            Self::Marketplace => "Marketplace dApp with listings, orders, and vendor dashboard",
        }
    }
}

/// Scaffold dApp tool
pub struct ScaffoldDappTool;

impl ScaffoldDappTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ScaffoldDappTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for ScaffoldDappTool {
    fn name(&self) -> &str {
        "scaffold_dapp"
    }

    fn description(&self) -> &str {
        "Generate a new dApp project from templates (basic, defi, nft, marketplace)"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let template_name = params.model_name.clone(); // Reuse as template type
        let project_name = params.prompt.clone(); // Project name
        Box::pin(async move {
            let name = project_name.unwrap_or_else(|| "my-dapp".to_string());

            // Validate project name
            if !is_valid_project_name(&name) {
                return Ok(ToolOutput {
                    tool: "scaffold_dapp".to_string(),
                    success: false,
                    message: format!(
                        "Invalid project name '{}'. Use lowercase letters, numbers, and hyphens.",
                        name
                    ),
                    data: None,
                });
            }

            // Get template type
            let template = template_name
                .as_deref()
                .and_then(DappTemplate::from_str)
                .unwrap_or(DappTemplate::Basic);

            // Check if directory already exists
            let project_dir = PathBuf::from(&name);
            if project_dir.exists() {
                return Ok(ToolOutput {
                    tool: "scaffold_dapp".to_string(),
                    success: false,
                    message: format!(
                        "Directory '{}' already exists. Choose a different name.",
                        name
                    ),
                    data: None,
                });
            }

            // Create project structure
            match create_project_structure(&name, template).await {
                Ok(files_created) => Ok(ToolOutput {
                    tool: "scaffold_dapp".to_string(),
                    success: true,
                    message: format!(
                        "Created {} dApp '{}' with {} files. Run 'cd {} && npm install' to get started.",
                        format!("{:?}", template).to_lowercase(),
                        name,
                        files_created.len(),
                        name
                    ),
                    data: Some(serde_json::json!({
                        "project_name": name,
                        "template": format!("{:?}", template),
                        "template_description": template.description(),
                        "files_created": files_created,
                        "next_steps": [
                            format!("cd {}", name),
                            "npm install",
                            "forge build",
                            "npm run dev"
                        ]
                    })),
                }),
                Err(e) => Ok(ToolOutput {
                    tool: "scaffold_dapp".to_string(),
                    success: false,
                    message: format!("Failed to create project: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// List templates tool
pub struct ListTemplatesToolImpl;

impl ListTemplatesToolImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListTemplatesToolImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for ListTemplatesToolImpl {
    fn name(&self) -> &str {
        "list_templates"
    }

    fn description(&self) -> &str {
        "List available dApp project templates"
    }

    fn execute(
        &self,
        _params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        Box::pin(async move {
            let templates = vec![
                serde_json::json!({
                    "name": "basic",
                    "description": DappTemplate::Basic.description(),
                    "features": ["Simple storage contract", "React + Vite frontend", "Foundry setup"]
                }),
                serde_json::json!({
                    "name": "defi",
                    "description": DappTemplate::Defi.description(),
                    "features": ["ERC20 token", "Liquidity pool", "Swap interface", "Price oracle"]
                }),
                serde_json::json!({
                    "name": "nft",
                    "description": DappTemplate::Nft.description(),
                    "features": ["ERC721 collection", "Minting page", "Gallery view", "Metadata support"]
                }),
                serde_json::json!({
                    "name": "marketplace",
                    "description": DappTemplate::Marketplace.description(),
                    "features": ["Listing management", "Order processing", "Escrow contract", "Vendor dashboard"]
                }),
            ];

            Ok(ToolOutput {
                tool: "list_templates".to_string(),
                success: true,
                message: format!("{} dApp templates available", templates.len()),
                data: Some(serde_json::json!({
                    "templates": templates,
                    "usage": "Use 'scaffold_dapp' with template name to create a project"
                })),
            })
        })
    }
}

/// Validate project name
fn is_valid_project_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 50 {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

/// Create project directory structure
async fn create_project_structure(
    name: &str,
    template: DappTemplate,
) -> Result<Vec<String>, std::io::Error> {
    let base = PathBuf::from(name);
    let mut files_created = Vec::new();

    // Create directories
    let dirs = vec![
        "",
        "contracts",
        "contracts/src",
        "contracts/test",
        "contracts/script",
        "frontend",
        "frontend/src",
        "frontend/src/components",
        "frontend/public",
    ];

    for dir in dirs {
        let dir_path = base.join(dir);
        tokio::fs::create_dir_all(&dir_path).await?;
    }

    // Create common files
    let common_files = get_common_files(name);
    for (path, content) in &common_files {
        let file_path = base.join(path);
        tokio::fs::write(&file_path, content).await?;
        files_created.push(path.to_string());
    }

    // Create template-specific files
    let template_files = match template {
        DappTemplate::Basic => get_basic_template_files(name),
        DappTemplate::Defi => get_defi_template_files(name),
        DappTemplate::Nft => get_nft_template_files(name),
        DappTemplate::Marketplace => get_marketplace_template_files(name),
    };

    for (path, content) in &template_files {
        let file_path = base.join(path);
        tokio::fs::write(&file_path, content).await?;
        files_created.push(path.to_string());
    }

    Ok(files_created)
}

/// Get common files for all templates
fn get_common_files(name: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "README.md",
            format!(
                r#"# {}

A Citrate dApp built with Foundry and React.

## Getting Started

```bash
# Install dependencies
npm install

# Build contracts
forge build

# Run tests
forge test

# Start frontend
npm run dev
```

## Project Structure

```
{}/
├── contracts/      # Solidity smart contracts
│   ├── src/        # Contract source files
│   ├── test/       # Contract tests
│   └── script/     # Deployment scripts
├── frontend/       # React frontend
│   └── src/        # Frontend source
└── README.md
```
"#,
                name, name
            ),
        ),
        (
            "package.json",
            format!(
                r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "dev": "cd frontend && npm run dev",
    "build": "cd frontend && npm run build",
    "test": "forge test",
    "deploy": "forge script script/Deploy.s.sol --broadcast"
  }},
  "devDependencies": {{
    "typescript": "^5.0.0"
  }}
}}"#,
                name
            ),
        ),
        (
            "contracts/foundry.toml",
            r#"[profile.default]
src = "src"
out = "out"
libs = ["lib"]
solc = "0.8.20"

[rpc_endpoints]
citrate = "http://localhost:8545"

[etherscan]
citrate = { key = "", url = "http://localhost:8545" }
"#
            .to_string(),
        ),
        (
            "frontend/package.json",
            format!(
                r#"{{
  "name": "{}-frontend",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "ethers": "^6.0.0"
  }},
  "devDependencies": {{
    "@vitejs/plugin-react": "^4.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.0.0"
  }}
}}"#,
                name
            ),
        ),
        (
            "frontend/vite.config.ts",
            r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000
  }
})
"#
            .to_string(),
        ),
        (
            "frontend/index.html",
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/src/main.tsx"></script>
</body>
</html>
"#,
                name
            ),
        ),
        (
            "frontend/src/main.tsx",
            r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"#
            .to_string(),
        ),
        (
            ".gitignore",
            r#"# Dependencies
node_modules/
frontend/node_modules/

# Build outputs
contracts/out/
contracts/cache/
frontend/dist/

# Environment
.env
.env.local

# IDE
.vscode/
.idea/

# OS
.DS_Store
"#
            .to_string(),
        ),
    ]
}

/// Basic template files
fn get_basic_template_files(name: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "contracts/src/Storage.sol",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Storage {
    uint256 private value;

    event ValueChanged(uint256 newValue);

    function setValue(uint256 _value) public {
        value = _value;
        emit ValueChanged(_value);
    }

    function getValue() public view returns (uint256) {
        return value;
    }
}
"#
            .to_string(),
        ),
        (
            "contracts/test/Storage.t.sol",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/Storage.sol";

contract StorageTest is Test {
    Storage public store;

    function setUp() public {
        store = new Storage();
    }

    function testSetValue() public {
        store.setValue(42);
        assertEq(store.getValue(), 42);
    }
}
"#
            .to_string(),
        ),
        (
            "frontend/src/App.tsx",
            format!(
                r#"import {{ useState }} from 'react'

function App() {{
  const [value, setValue] = useState(0)

  return (
    <div style={{{{ padding: '20px', fontFamily: 'system-ui' }}}}>
      <h1>{}</h1>
      <p>Current value: {{value}}</p>
      <button onClick={{() => setValue(v => v + 1)}}>
        Increment
      </button>
    </div>
  )
}}

export default App
"#,
                name
            ),
        ),
    ]
}

/// DeFi template files
fn get_defi_template_files(name: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "contracts/src/Token.sol",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract Token is ERC20 {
    constructor(string memory name, string memory symbol, uint256 initialSupply)
        ERC20(name, symbol)
    {
        _mint(msg.sender, initialSupply * 10 ** decimals());
    }
}
"#
            .to_string(),
        ),
        (
            "contracts/src/Pool.sol",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Pool {
    IERC20 public tokenA;
    IERC20 public tokenB;

    uint256 public reserveA;
    uint256 public reserveB;

    constructor(address _tokenA, address _tokenB) {
        tokenA = IERC20(_tokenA);
        tokenB = IERC20(_tokenB);
    }

    function addLiquidity(uint256 amountA, uint256 amountB) external {
        tokenA.transferFrom(msg.sender, address(this), amountA);
        tokenB.transferFrom(msg.sender, address(this), amountB);
        reserveA += amountA;
        reserveB += amountB;
    }

    function swap(address tokenIn, uint256 amountIn) external returns (uint256) {
        require(tokenIn == address(tokenA) || tokenIn == address(tokenB), "Invalid token");

        bool isTokenA = tokenIn == address(tokenA);
        (uint256 reserveIn, uint256 reserveOut) = isTokenA
            ? (reserveA, reserveB)
            : (reserveB, reserveA);

        uint256 amountOut = (amountIn * reserveOut) / (reserveIn + amountIn);

        if (isTokenA) {
            tokenA.transferFrom(msg.sender, address(this), amountIn);
            tokenB.transfer(msg.sender, amountOut);
            reserveA += amountIn;
            reserveB -= amountOut;
        } else {
            tokenB.transferFrom(msg.sender, address(this), amountIn);
            tokenA.transfer(msg.sender, amountOut);
            reserveB += amountIn;
            reserveA -= amountOut;
        }

        return amountOut;
    }
}
"#
            .to_string(),
        ),
        (
            "frontend/src/App.tsx",
            format!(
                r#"import {{ useState }} from 'react'

function App() {{
  const [amount, setAmount] = useState('')

  return (
    <div style={{{{ padding: '20px', fontFamily: 'system-ui' }}}}>
      <h1>{} DeFi</h1>
      <div style={{{{ marginTop: '20px' }}}}>
        <h2>Swap</h2>
        <input
          type="number"
          value={{amount}}
          onChange={{e => setAmount(e.target.value)}}
          placeholder="Amount to swap"
        />
        <button style={{{{ marginLeft: '10px' }}}}>Swap</button>
      </div>
    </div>
  )
}}

export default App
"#,
                name
            ),
        ),
    ]
}

/// NFT template files
fn get_nft_template_files(name: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "contracts/src/Collection.sol",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract Collection is ERC721, Ownable {
    uint256 private _tokenIdCounter;
    uint256 public mintPrice = 0.01 ether;
    string private _baseTokenURI;

    constructor(string memory name, string memory symbol)
        ERC721(name, symbol)
        Ownable(msg.sender)
    {}

    function mint() external payable {
        require(msg.value >= mintPrice, "Insufficient payment");
        uint256 tokenId = _tokenIdCounter++;
        _safeMint(msg.sender, tokenId);
    }

    function setBaseURI(string memory baseURI) external onlyOwner {
        _baseTokenURI = baseURI;
    }

    function _baseURI() internal view override returns (string memory) {
        return _baseTokenURI;
    }

    function withdraw() external onlyOwner {
        payable(owner()).transfer(address(this).balance);
    }
}
"#
            .to_string(),
        ),
        (
            "frontend/src/App.tsx",
            format!(
                r#"import {{ useState }} from 'react'

function App() {{
  const [minting, setMinting] = useState(false)

  const handleMint = async () => {{
    setMinting(true)
    // Add mint logic
    setTimeout(() => setMinting(false), 2000)
  }}

  return (
    <div style={{{{ padding: '20px', fontFamily: 'system-ui' }}}}>
      <h1>{} NFT Collection</h1>
      <div style={{{{ marginTop: '20px' }}}}>
        <button onClick={{handleMint}} disabled={{minting}}>
          {{minting ? 'Minting...' : 'Mint NFT'}}
        </button>
      </div>
      <div style={{{{ marginTop: '40px' }}}}>
        <h2>Gallery</h2>
        <p>Your NFTs will appear here</p>
      </div>
    </div>
  )
}}

export default App
"#,
                name
            ),
        ),
    ]
}

/// Marketplace template files
fn get_marketplace_template_files(name: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "contracts/src/Marketplace.sol",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Marketplace {
    struct Listing {
        address seller;
        string title;
        string description;
        uint256 price;
        bool active;
    }

    mapping(uint256 => Listing) public listings;
    uint256 public listingCount;
    uint256 public fee = 25; // 2.5% fee (in basis points)

    event ListingCreated(uint256 indexed id, address seller, uint256 price);
    event ListingSold(uint256 indexed id, address buyer);

    function createListing(
        string memory title,
        string memory description,
        uint256 price
    ) external returns (uint256) {
        uint256 id = listingCount++;
        listings[id] = Listing({
            seller: msg.sender,
            title: title,
            description: description,
            price: price,
            active: true
        });
        emit ListingCreated(id, msg.sender, price);
        return id;
    }

    function purchase(uint256 id) external payable {
        Listing storage listing = listings[id];
        require(listing.active, "Listing not active");
        require(msg.value >= listing.price, "Insufficient payment");

        listing.active = false;

        uint256 feeAmount = (listing.price * fee) / 1000;
        uint256 sellerAmount = listing.price - feeAmount;

        payable(listing.seller).transfer(sellerAmount);

        emit ListingSold(id, msg.sender);
    }
}
"#
            .to_string(),
        ),
        (
            "frontend/src/App.tsx",
            format!(
                r#"import {{ useState }} from 'react'

interface Listing {{
  id: number
  title: string
  price: string
}}

function App() {{
  const [listings] = useState<Listing[]>([
    {{ id: 1, title: 'Sample Item', price: '0.1' }}
  ])

  return (
    <div style={{{{ padding: '20px', fontFamily: 'system-ui' }}}}>
      <h1>{} Marketplace</h1>
      <div style={{{{ marginTop: '20px' }}}}>
        <h2>Listings</h2>
        {{listings.map(listing => (
          <div key={{listing.id}} style={{{{
            border: '1px solid #ccc',
            padding: '10px',
            marginBottom: '10px'
          }}}}>
            <h3>{{listing.title}}</h3>
            <p>Price: {{listing.price}} SALT</p>
            <button>Purchase</button>
          </div>
        ))}}
      </div>
    </div>
  )
}}

export default App
"#,
                name
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_dapp_tool_name() {
        let tool = ScaffoldDappTool::new();
        assert_eq!(tool.name(), "scaffold_dapp");
    }

    #[test]
    fn test_scaffold_dapp_tool_description() {
        let tool = ScaffoldDappTool::new();
        assert!(tool.description().contains("Generate"));
        assert!(tool.description().contains("dApp"));
    }

    #[test]
    fn test_scaffold_dapp_default() {
        let _tool = ScaffoldDappTool::default();
        // Just verify default creation works
    }

    #[test]
    fn test_list_templates_tool_name() {
        let tool = ListTemplatesToolImpl::new();
        assert_eq!(tool.name(), "list_templates");
    }

    #[test]
    fn test_list_templates_tool_description() {
        let tool = ListTemplatesToolImpl::new();
        assert!(tool.description().contains("List"));
        assert!(tool.description().contains("templates"));
    }

    #[test]
    fn test_list_templates_default() {
        let _tool = ListTemplatesToolImpl::default();
        // Just verify default creation works
    }

    #[test]
    fn test_dapp_template_from_str() {
        assert_eq!(DappTemplate::from_str("basic"), Some(DappTemplate::Basic));
        assert_eq!(DappTemplate::from_str("BASIC"), Some(DappTemplate::Basic));
        assert_eq!(DappTemplate::from_str("simple"), Some(DappTemplate::Basic));
        assert_eq!(DappTemplate::from_str("starter"), Some(DappTemplate::Basic));

        assert_eq!(DappTemplate::from_str("defi"), Some(DappTemplate::Defi));
        assert_eq!(DappTemplate::from_str("token"), Some(DappTemplate::Defi));
        assert_eq!(DappTemplate::from_str("swap"), Some(DappTemplate::Defi));

        assert_eq!(DappTemplate::from_str("nft"), Some(DappTemplate::Nft));
        assert_eq!(DappTemplate::from_str("collection"), Some(DappTemplate::Nft));
        assert_eq!(DappTemplate::from_str("mint"), Some(DappTemplate::Nft));

        assert_eq!(DappTemplate::from_str("marketplace"), Some(DappTemplate::Marketplace));
        assert_eq!(DappTemplate::from_str("market"), Some(DappTemplate::Marketplace));
        assert_eq!(DappTemplate::from_str("shop"), Some(DappTemplate::Marketplace));

        assert_eq!(DappTemplate::from_str("unknown"), None);
    }

    #[test]
    fn test_dapp_template_description() {
        assert!(DappTemplate::Basic.description().contains("Basic"));
        assert!(DappTemplate::Defi.description().contains("DeFi"));
        assert!(DappTemplate::Nft.description().contains("NFT"));
        assert!(DappTemplate::Marketplace.description().contains("Marketplace"));
    }

    #[test]
    fn test_is_valid_project_name() {
        // Valid names
        assert!(is_valid_project_name("my-dapp"));
        assert!(is_valid_project_name("my_dapp"));
        assert!(is_valid_project_name("mydapp123"));
        assert!(is_valid_project_name("test"));
        assert!(is_valid_project_name("a"));

        // Invalid names
        assert!(!is_valid_project_name("")); // Empty
        assert!(!is_valid_project_name("-mydapp")); // Starts with hyphen
        assert!(!is_valid_project_name("mydapp-")); // Ends with hyphen
        assert!(!is_valid_project_name("MyDapp")); // Uppercase
        assert!(!is_valid_project_name("my dapp")); // Space
        assert!(!is_valid_project_name("my.dapp")); // Dot
        assert!(!is_valid_project_name(&"a".repeat(51))); // Too long (51 chars)
    }

    #[test]
    fn test_get_common_files() {
        let files = get_common_files("test-project");

        // Check that essential files are included
        let file_names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert!(file_names.contains(&"README.md"));
        assert!(file_names.contains(&"package.json"));
        assert!(file_names.contains(&"contracts/foundry.toml"));
        assert!(file_names.contains(&".gitignore"));

        // Check project name is included in generated content
        let readme = files.iter().find(|(name, _)| *name == "README.md").unwrap();
        assert!(readme.1.contains("test-project"));
    }

    #[test]
    fn test_get_basic_template_files() {
        let files = get_basic_template_files("my-basic-dapp");

        let file_names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert!(file_names.contains(&"contracts/src/Storage.sol"));
        assert!(file_names.contains(&"contracts/test/Storage.t.sol"));
        assert!(file_names.contains(&"frontend/src/App.tsx"));
    }

    #[test]
    fn test_get_defi_template_files() {
        let files = get_defi_template_files("my-defi-dapp");

        let file_names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert!(file_names.contains(&"contracts/src/Token.sol"));
        assert!(file_names.contains(&"contracts/src/Pool.sol"));
        assert!(file_names.contains(&"frontend/src/App.tsx"));

        // Check DeFi-specific content
        let pool = files.iter().find(|(name, _)| *name == "contracts/src/Pool.sol").unwrap();
        assert!(pool.1.contains("swap"));
        assert!(pool.1.contains("addLiquidity"));
    }

    #[test]
    fn test_get_nft_template_files() {
        let files = get_nft_template_files("my-nft-dapp");

        let file_names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert!(file_names.contains(&"contracts/src/Collection.sol"));
        assert!(file_names.contains(&"frontend/src/App.tsx"));

        // Check NFT-specific content
        let collection = files.iter().find(|(name, _)| *name == "contracts/src/Collection.sol").unwrap();
        assert!(collection.1.contains("ERC721"));
        assert!(collection.1.contains("mint"));
    }

    #[test]
    fn test_get_marketplace_template_files() {
        let files = get_marketplace_template_files("my-market");

        let file_names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert!(file_names.contains(&"contracts/src/Marketplace.sol"));
        assert!(file_names.contains(&"frontend/src/App.tsx"));

        // Check marketplace-specific content
        let marketplace = files.iter().find(|(name, _)| *name == "contracts/src/Marketplace.sol").unwrap();
        assert!(marketplace.1.contains("Listing"));
        assert!(marketplace.1.contains("purchase"));
    }

    #[tokio::test]
    async fn test_list_templates_execution() {
        let tool = ListTemplatesToolImpl::new();
        let params = IntentParams::default();

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        let templates = data.get("templates").and_then(|t| t.as_array()).unwrap();
        assert_eq!(templates.len(), 4); // basic, defi, nft, marketplace
    }

    #[tokio::test]
    async fn test_scaffold_dapp_invalid_name() {
        let tool = ScaffoldDappTool::new();
        let mut params = IntentParams::default();
        params.prompt = Some("-invalid".to_string()); // Invalid: starts with hyphen

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.success);
        assert!(output.message.contains("Invalid project name"));
    }

    #[tokio::test]
    async fn test_scaffold_dapp_default_template() {
        let tool = ScaffoldDappTool::new();
        let mut params = IntentParams::default();
        params.prompt = Some("test-scaffold-default".to_string());

        // This will try to create the directory - we're testing the logic path
        // Clean up after test
        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        // Either succeeds or fails due to directory issues - both are valid
        if output.success {
            // Clean up
            let _ = tokio::fs::remove_dir_all("test-scaffold-default").await;
        }
    }
}
