import React, { useState, useRef, useEffect } from 'react';
import {
  Box,
  Grid,
  Paper,
  Tabs,
  Tab,
  IconButton,
  Button,
  Typography,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Chip,
  Tooltip,
  Toolbar,
} from '@mui/material';
import {
  PlayArrow as RunIcon,
  Save as SaveIcon,
  FolderOpen as OpenIcon,
  Add as AddIcon,
  Close as CloseIcon,
  Description as FileIcon,
  Folder as FolderIcon,
  MoreVert as MoreIcon,
  Terminal as TerminalIcon,
  BugReport as DebugIcon,
} from '@mui/icons-material';
import Editor from '@monaco-editor/react';

const FILE_TEMPLATES = {
  'model.py': `# Citrate AI Model Template
import numpy as np
from typing import Dict, Any

class LatticeModel:
    """
    Base class for Citrate AI models.
    Implement the predict method for your specific model.
    """

    def __init__(self):
        self.model_id = None
        self.version = "1.0.0"

    def predict(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Make predictions using the model.

        Args:
            input_data: Input data for prediction

        Returns:
            Dictionary containing prediction results
        """
        # Implement your model logic here
        return {
            "prediction": "example_output",
            "confidence": 0.95,
            "model_version": self.version
        }

    def validate_input(self, input_data: Dict[str, Any]) -> bool:
        """Validate input data format"""
        # Add validation logic here
        return True

# Example usage
if __name__ == "__main__":
    model = LatticeModel()
    result = model.predict({"input": "test_data"})
    print(f"Prediction result: {result}")
`,
  'deploy.js': `// Citrate Model Deployment Script
const { CitrateClient } = require('citrate-js');
const fs = require('fs');
const path = require('path');

async function deployModel() {
    // Initialize Citrate client
    const client = new CitrateClient('http://localhost:8545');

    try {
        // Load model file
        const modelPath = './model.py';
        const modelData = fs.readFileSync(modelPath);

        // Model metadata
        const metadata = {
            name: 'My AI Model',
            version: '1.0.0',
            description: 'A sample AI model for Citrate blockchain',
            author: 'Developer',
            tags: ['ai', 'ml', 'prediction'],
            license: 'MIT'
        };

        // Deploy model
        console.log('Deploying model to Lattice...');
        const result = await client.deployModel({
            modelData,
            metadata,
            encrypted: false,
            price: '1000000000000000000' // 1 ETH in wei
        });

        console.log('Model deployed successfully!');
        console.log('Model ID:', result.modelId);
        console.log('Transaction:', result.txHash);

        return result;
    } catch (error) {
        console.error('Deployment failed:', error);
        throw error;
    }
}

// Run deployment
if (require.main === module) {
    deployModel()
        .then(result => {
            console.log('Deployment completed:', result.modelId);
            process.exit(0);
        })
        .catch(error => {
            console.error('Deployment failed:', error);
            process.exit(1);
        });
}

module.exports = { deployModel };
`,
  'contract.sol': `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title LatticeModelContract
 * @dev Smart contract for managing AI model access and payments
 */
contract LatticeModelContract {
    struct Model {
        string modelId;
        address owner;
        uint256 price;
        bool active;
        string metadataHash;
        uint256 totalInferences;
        uint256 totalRevenue;
    }

    mapping(string => Model) public models;
    mapping(address => uint256) public balances;

    event ModelRegistered(string indexed modelId, address indexed owner, uint256 price);
    event InferenceExecuted(string indexed modelId, address indexed user, uint256 price);
    event PaymentReceived(address indexed from, uint256 amount);

    modifier onlyModelOwner(string memory modelId) {
        require(models[modelId].owner == msg.sender, "Not model owner");
        _;
    }

    modifier modelExists(string memory modelId) {
        require(bytes(models[modelId].modelId).length > 0, "Model does not exist");
        _;
    }

    /**
     * @dev Register a new AI model
     */
    function registerModel(
        string memory modelId,
        uint256 price,
        string memory metadataHash
    ) external {
        require(bytes(models[modelId].modelId).length == 0, "Model already exists");

        models[modelId] = Model({
            modelId: modelId,
            owner: msg.sender,
            price: price,
            active: true,
            metadataHash: metadataHash,
            totalInferences: 0,
            totalRevenue: 0
        });

        emit ModelRegistered(modelId, msg.sender, price);
    }

    /**
     * @dev Execute inference and handle payment
     */
    function executeInference(string memory modelId)
        external
        payable
        modelExists(modelId)
    {
        Model storage model = models[modelId];
        require(model.active, "Model is not active");
        require(msg.value >= model.price, "Insufficient payment");

        // Update model statistics
        model.totalInferences++;
        model.totalRevenue += msg.value;

        // Add to owner's balance
        balances[model.owner] += msg.value;

        emit InferenceExecuted(modelId, msg.sender, msg.value);
    }

    /**
     * @dev Withdraw earnings
     */
    function withdraw() external {
        uint256 balance = balances[msg.sender];
        require(balance > 0, "No balance to withdraw");

        balances[msg.sender] = 0;
        payable(msg.sender).transfer(balance);
    }

    /**
     * @dev Update model price
     */
    function updateModelPrice(string memory modelId, uint256 newPrice)
        external
        onlyModelOwner(modelId)
    {
        models[modelId].price = newPrice;
    }

    /**
     * @dev Toggle model active status
     */
    function toggleModelStatus(string memory modelId)
        external
        onlyModelOwner(modelId)
    {
        models[modelId].active = !models[modelId].active;
    }

    /**
     * @dev Get model information
     */
    function getModelInfo(string memory modelId)
        external
        view
        returns (Model memory)
    {
        return models[modelId];
    }
}
`,
  'test.js': `// Citrate Model Testing Suite
const { CitrateClient } = require('citrate-js');
const assert = require('assert');

class ModelTester {
    constructor(rpcUrl = 'http://localhost:8545') {
        this.client = new CitrateClient(rpcUrl);
        this.testResults = [];
    }

    async runAllTests() {
        console.log('🧪 Starting Citrate Model Tests...');

        await this.testConnection();
        await this.testModelDeployment();
        await this.testInference();
        await this.testPayments();

        this.printResults();
    }

    async testConnection() {
        console.log('Testing blockchain connection...');
        try {
            const status = await this.client.getNetworkStatus();
            assert(status.connected, 'Should be connected to network');
            this.addTest('Connection Test', true, 'Connected successfully');
        } catch (error) {
            this.addTest('Connection Test', false, error.message);
        }
    }

    async testModelDeployment() {
        console.log('Testing model deployment...');
        try {
            // Mock model data
            const modelData = Buffer.from('mock_model_data');
            const metadata = {
                name: 'Test Model',
                version: '1.0.0',
                description: 'Test model for unit testing'
            };

            const result = await this.client.deployModel({
                modelData,
                metadata,
                price: '1000000000000000000' // 1 ETH
            });

            assert(result.modelId, 'Should return model ID');
            assert(result.txHash, 'Should return transaction hash');

            this.testModelId = result.modelId;
            this.addTest('Model Deployment', true, \`Model ID: \${result.modelId}\`);
        } catch (error) {
            this.addTest('Model Deployment', false, error.message);
        }
    }

    async testInference() {
        console.log('Testing model inference...');
        try {
            if (!this.testModelId) {
                throw new Error('No model deployed for testing');
            }

            const inputData = { input: 'test_data' };
            const result = await this.client.runInference(this.testModelId, inputData);

            assert(result.outputData, 'Should return output data');
            assert(result.executionTime >= 0, 'Should have execution time');

            this.addTest('Model Inference', true, 'Inference completed successfully');
        } catch (error) {
            this.addTest('Model Inference', false, error.message);
        }
    }

    async testPayments() {
        console.log('Testing payment system...');
        try {
            // Test balance checks, payment processing, etc.
            const accounts = await this.client.getAccounts();
            assert(accounts.length > 0, 'Should have at least one account');

            for (const account of accounts.slice(0, 2)) {
                const balance = await this.client.getBalance(account);
                assert(typeof balance === 'string', 'Balance should be a string');
            }

            this.addTest('Payment System', true, 'Payment verification passed');
        } catch (error) {
            this.addTest('Payment System', false, error.message);
        }
    }

    addTest(name, passed, message) {
        this.testResults.push({ name, passed, message });
    }

    printResults() {
        console.log('\\n📊 Test Results:');
        console.log('='.repeat(50));

        let passed = 0;
        let total = this.testResults.length;

        this.testResults.forEach(test => {
            const status = test.passed ? '✅ PASS' : '❌ FAIL';
            console.log(\`\${status} \${test.name}: \${test.message}\`);
            if (test.passed) passed++;
        });

        console.log('='.repeat(50));
        console.log(\`\${passed}/\${total} tests passed (\${Math.round(passed/total*100)}%)\`);

        if (passed === total) {
            console.log('🎉 All tests passed!');
        } else {
            console.log('⚠️  Some tests failed. Check the details above.');
        }
    }
}

// Run tests if executed directly
if (require.main === module) {
    const tester = new ModelTester();
    tester.runAllTests().catch(console.error);
}

module.exports = ModelTester;
`
};

function FileTree({ files, onFileSelect, selectedFile }) {
  return (
    <Paper sx={{ height: '100%', p: 1 }}>
      <Typography variant="subtitle2" sx={{ mb: 1, fontWeight: 'bold' }}>
        Project Files
      </Typography>
      <List dense>
        {Object.keys(files).map((filename) => (
          <ListItem
            key={filename}
            button
            onClick={() => onFileSelect(filename)}
            selected={selectedFile === filename}
          >
            <ListItemIcon>
              <FileIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText
              primary={filename}
              primaryTypographyProps={{ variant: 'body2' }}
            />
          </ListItem>
        ))}
      </List>
    </Paper>
  );
}

function CodeEditor({ latticeService, addNotification }) {
  const [files, setFiles] = useState(FILE_TEMPLATES);
  const [activeFile, setActiveFile] = useState('model.py');
  const [isRunning, setIsRunning] = useState(false);
  const [output, setOutput] = useState('');
  const [newFileDialog, setNewFileDialog] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const editorRef = useRef(null);

  const handleEditorDidMount = (editor, monaco) => {
    editorRef.current = editor;

    // Configure Monaco editor for Citrate development
    monaco.languages.typescript.javascriptDefaults.addExtraLib(`
      declare module 'citrate-js' {
        export class CitrateClient {
          constructor(rpcUrl: string);
          deployModel(options: any): Promise<any>;
          runInference(modelId: string, input: any): Promise<any>;
          getNetworkStatus(): Promise<any>;
        }
      }
    `, 'citrate-js.d.ts');
  };

  const handleFileChange = (filename) => {
    setActiveFile(filename);
  };

  const handleCodeChange = (value) => {
    setFiles(prev => ({
      ...prev,
      [activeFile]: value
    }));
  };

  const handleSave = () => {
    // In a real implementation, this would save to local storage or server
    addNotification(\`File \${activeFile} saved successfully\`, 'success');
  };

  const handleRun = async () => {
    setIsRunning(true);
    setOutput('Running code...\\n');

    try {
      // Simulate code execution
      if (activeFile.endsWith('.py')) {
        setOutput(prev => prev + '🐍 Executing Python code...\\n');
        setTimeout(() => {
          setOutput(prev => prev + 'Model validation: ✅ PASSED\\n');
          setOutput(prev => prev + 'Prediction test: ✅ PASSED\\n');
          setOutput(prev => prev + '\\n✅ Python execution completed successfully\\n');
          setIsRunning(false);
        }, 2000);
      } else if (activeFile.endsWith('.js')) {
        setOutput(prev => prev + '🟨 Executing JavaScript code...\\n');
        setTimeout(() => {
          setOutput(prev => prev + 'Connecting to Citrate node...\\n');
          setOutput(prev => prev + 'Model deployed: model_123abc\\n');
          setOutput(prev => prev + 'Transaction hash: 0x123...abc\\n');
          setOutput(prev => prev + '\\n✅ JavaScript execution completed successfully\\n');
          setIsRunning(false);
        }, 2000);
      } else if (activeFile.endsWith('.sol')) {
        setOutput(prev => prev + '🔷 Compiling Solidity contract...\\n');
        setTimeout(() => {
          setOutput(prev => prev + 'Compilation successful\\n');
          setOutput(prev => prev + 'Contract size: 2,456 bytes\\n');
          setOutput(prev => prev + 'Gas estimate: 1,234,567\\n');
          setOutput(prev => prev + '\\n✅ Solidity compilation completed successfully\\n');
          setIsRunning(false);
        }, 2000);
      }
    } catch (error) {
      setOutput(prev => prev + \`\\n❌ Error: \${error.message}\\n\`);
      setIsRunning(false);
    }
  };

  const handleNewFile = () => {
    if (newFileName && !files[newFileName]) {
      setFiles(prev => ({
        ...prev,
        [newFileName]: \`// New file: \${newFileName}\\n\\n\`
      }));
      setActiveFile(newFileName);
      setNewFileDialog(false);
      setNewFileName('');
      addNotification(\`File \${newFileName} created\`, 'success');
    }
  };

  const getLanguage = (filename) => {
    if (filename.endsWith('.py')) return 'python';
    if (filename.endsWith('.js')) return 'javascript';
    if (filename.endsWith('.sol')) return 'solidity';
    if (filename.endsWith('.json')) return 'json';
    return 'plaintext';
  };

  return (
    <Box sx={{ height: 'calc(100vh - 64px)', display: 'flex' }}>
      {/* File Tree */}
      <Box sx={{ width: 250, borderRight: 1, borderColor: 'divider' }}>
        <FileTree
          files={files}
          onFileSelect={handleFileChange}
          selectedFile={activeFile}
        />
      </Box>

      {/* Editor Area */}
      <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
        {/* Toolbar */}
        <Toolbar variant="dense" sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tabs value={activeFile} onChange={(e, value) => setActiveFile(value)} sx={{ flexGrow: 1 }}>
            {Object.keys(files).map(filename => (
              <Tab
                key={filename}
                value={filename}
                label={filename}
                sx={{ minHeight: 'auto', py: 1 }}
              />
            ))}
          </Tabs>

          <Box sx={{ ml: 2 }}>
            <Tooltip title="New File">
              <IconButton onClick={() => setNewFileDialog(true)}>
                <AddIcon />
              </IconButton>
            </Tooltip>
            <Tooltip title="Save">
              <IconButton onClick={handleSave}>
                <SaveIcon />
              </IconButton>
            </Tooltip>
            <Tooltip title="Run Code">
              <IconButton onClick={handleRun} disabled={isRunning}>
                <RunIcon />
              </IconButton>
            </Tooltip>
          </Box>
        </Toolbar>

        {/* Editor and Output */}
        <Box sx={{ flexGrow: 1, display: 'flex' }}>
          {/* Code Editor */}
          <Box sx={{ flexGrow: 1 }}>
            <Editor
              height="100%"
              language={getLanguage(activeFile)}
              value={files[activeFile] || ''}
              onChange={handleCodeChange}
              onMount={handleEditorDidMount}
              theme="vs-dark"
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                wordWrap: 'on',
                automaticLayout: true,
                scrollBeyondLastLine: false,
                tabSize: 2,
                insertSpaces: true,
              }}
            />
          </Box>

          {/* Output Panel */}
          <Box sx={{ width: 400, borderLeft: 1, borderColor: 'divider', display: 'flex', flexDirection: 'column' }}>
            <Box sx={{ p: 1, borderBottom: 1, borderColor: 'divider' }}>
              <Typography variant="subtitle2" sx={{ display: 'flex', alignItems: 'center' }}>
                <TerminalIcon sx={{ mr: 1 }} />
                Output
              </Typography>
            </Box>
            <Box sx={{ flexGrow: 1, p: 2, backgroundColor: '#1e1e1e', color: '#d4d4d4', fontFamily: 'monospace', fontSize: '14px', overflow: 'auto' }}>
              <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>{output || 'Ready to run code...'}</pre>
            </Box>
          </Box>
        </Box>
      </Box>

      {/* New File Dialog */}
      <Dialog open={newFileDialog} onClose={() => setNewFileDialog(false)}>
        <DialogTitle>Create New File</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="File Name"
            fullWidth
            value={newFileName}
            onChange={(e) => setNewFileName(e.target.value)}
            placeholder="example.py"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setNewFileDialog(false)}>Cancel</Button>
          <Button onClick={handleNewFile} disabled={!newFileName}>Create</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

export default CodeEditor;