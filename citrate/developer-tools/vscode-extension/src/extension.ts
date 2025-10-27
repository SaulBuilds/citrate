import * as vscode from 'vscode';
import { LatticeConnectionProvider } from './providers/connectionProvider';
import { LatticeModelProvider } from './providers/modelProvider';
import { LatticeNetworkProvider } from './providers/networkProvider';
import { LatticeCodeLensProvider } from './providers/codeLensProvider';
import { CitrateClient } from './citrateClient';
import { ProjectTemplates } from './templates/projectTemplates';

let citrateClient: CitrateClient;
let connectionProvider: LatticeConnectionProvider;
let modelProvider: LatticeModelProvider;
let networkProvider: LatticeNetworkProvider;

export function activate(context: vscode.ExtensionContext) {
    console.log('Citrate AI Blockchain extension is now active!');

    // Initialize Citrate client
    const config = vscode.workspace.getConfiguration('lattice');
    const rpcUrl = config.get<string>('rpcUrl', 'http://localhost:8545');
    citrateClient = new CitrateClient(rpcUrl);

    // Initialize providers
    connectionProvider = new LatticeConnectionProvider(citrateClient);
    modelProvider = new LatticeModelProvider(citrateClient);
    networkProvider = new LatticeNetworkProvider(citrateClient);

    // Register tree data providers
    vscode.window.createTreeView('latticeConnection', {
        treeDataProvider: connectionProvider,
        showCollapseAll: true
    });

    vscode.window.createTreeView('latticeModels', {
        treeDataProvider: modelProvider,
        showCollapseAll: true
    });

    vscode.window.createTreeView('latticeNetwork', {
        treeDataProvider: networkProvider,
        showCollapseAll: true
    });

    // Register CodeLens provider
    const codeLensProvider = new LatticeCodeLensProvider(citrateClient);
    vscode.languages.registerCodeLensProvider(
        { language: 'python', scheme: 'file' },
        codeLensProvider
    );

    // Register commands
    registerCommands(context);

    // Auto-connect if enabled
    if (config.get<boolean>('autoConnect', true)) {
        vscode.commands.executeCommand('lattice.connectNode');
    }

    // Watch for configuration changes
    vscode.workspace.onDidChangeConfiguration(event => {
        if (event.affectsConfiguration('lattice.rpcUrl')) {
            const newRpcUrl = vscode.workspace.getConfiguration('lattice').get<string>('rpcUrl');
            if (newRpcUrl) {
                citrateClient.updateRpcUrl(newRpcUrl);
            }
        }
    });

    // Status bar
    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBarItem.command = 'lattice.connectNode';
    context.subscriptions.push(statusBarItem);

    // Update status bar based on connection
    citrateClient.onConnectionChange((connected: boolean) => {
        statusBarItem.text = connected ? '$(symbol-misc) Lattice: Connected' : '$(symbol-misc) Lattice: Disconnected';
        statusBarItem.tooltip = connected ? 'Connected to Citrate node' : 'Click to connect to Citrate node';
        statusBarItem.show();

        // Update context
        vscode.commands.executeCommand('setContext', 'lattice.connected', connected);
    });

    statusBarItem.show();
}

function registerCommands(context: vscode.ExtensionContext) {
    // Connect to node
    const connectCommand = vscode.commands.registerCommand('lattice.connectNode', async () => {
        const config = vscode.workspace.getConfiguration('lattice');
        const rpcUrl = config.get<string>('rpcUrl', 'http://localhost:8545');

        try {
            vscode.commands.executeCommand('setContext', 'lattice.connecting', true);
            await citrateClient.connect();
            vscode.window.showInformationMessage(`Connected to Citrate node at ${rpcUrl}`);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to connect to Citrate node: ${error}`);
        } finally {
            vscode.commands.executeCommand('setContext', 'lattice.connecting', false);
        }
    });

    // Deploy model
    const deployCommand = vscode.commands.registerCommand('lattice.deployModel', async (uri?: vscode.Uri) => {
        if (!citrateClient.isConnected) {
            vscode.window.showWarningMessage('Please connect to Citrate node first');
            return;
        }

        const fileUri = uri || vscode.window.activeTextEditor?.document.uri;
        if (!fileUri) {
            vscode.window.showErrorMessage('No file selected for deployment');
            return;
        }

        try {
            const document = await vscode.workspace.openTextDocument(fileUri);
            const modelCode = document.getText();

            // Show deployment options
            const options = await vscode.window.showQuickPick([
                { label: 'Deploy as Public Model', value: 'public' },
                { label: 'Deploy as Private Model', value: 'private' },
                { label: 'Deploy with Custom Settings', value: 'custom' }
            ], { placeHolder: 'Select deployment option' });

            if (!options) return;

            let deploymentConfig = {
                encrypted: options.value === 'private',
                price: '1000000000000000000', // 1 ETH default
                metadata: {
                    name: fileUri.path.split('/').pop()?.replace('.py', '') || 'Unnamed Model',
                    description: 'Model deployed from VS Code',
                    version: '1.0.0'
                }
            };

            if (options.value === 'custom') {
                // Get custom configuration
                const name = await vscode.window.showInputBox({
                    prompt: 'Model name',
                    value: deploymentConfig.metadata.name
                });
                if (!name) return;

                const price = await vscode.window.showInputBox({
                    prompt: 'Price in ETH',
                    value: '1.0'
                });
                if (!price) return;

                deploymentConfig.metadata.name = name;
                deploymentConfig.price = (parseFloat(price) * 1e18).toString();
            }

            // Show progress
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Deploying model to Lattice...',
                cancellable: false
            }, async (progress) => {
                progress.report({ increment: 0, message: 'Preparing deployment...' });

                const result = await citrateClient.deployModel(
                    Buffer.from(modelCode),
                    deploymentConfig
                );

                progress.report({ increment: 100, message: 'Deployment complete!' });

                vscode.window.showInformationMessage(
                    `Model deployed successfully! Model ID: ${result.modelId}`,
                    'View in Explorer'
                ).then(selection => {
                    if (selection === 'View in Explorer') {
                        vscode.commands.executeCommand('lattice.viewModels');
                    }
                });

                // Refresh model provider
                modelProvider.refresh();
            });

        } catch (error) {
            vscode.window.showErrorMessage(`Deployment failed: ${error}`);
        }
    });

    // Run inference
    const inferenceCommand = vscode.commands.registerCommand('lattice.runInference', async () => {
        if (!citrateClient.isConnected) {
            vscode.window.showWarningMessage('Please connect to Citrate node first');
            return;
        }

        try {
            // Get available models
            const models = await citrateClient.getModels();
            if (models.length === 0) {
                vscode.window.showInformationMessage('No models available for inference');
                return;
            }

            // Select model
            const modelItems = models.map(model => ({
                label: model.name || model.id,
                description: model.description,
                value: model.id
            }));

            const selectedModel = await vscode.window.showQuickPick(modelItems, {
                placeHolder: 'Select a model for inference'
            });

            if (!selectedModel) return;

            // Get input data
            const inputData = await vscode.window.showInputBox({
                prompt: 'Enter input data (JSON format)',
                placeHolder: '{"input": "your_data_here"}'
            });

            if (!inputData) return;

            // Run inference
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Running inference...',
                cancellable: false
            }, async (progress) => {
                progress.report({ increment: 0, message: 'Sending request...' });

                let parsedInput;
                try {
                    parsedInput = JSON.parse(inputData);
                } catch {
                    parsedInput = { input: inputData };
                }

                const result = await citrateClient.runInference(selectedModel.value, parsedInput);

                progress.report({ increment: 100, message: 'Inference complete!' });

                // Show result in new document
                const resultDoc = await vscode.workspace.openTextDocument({
                    content: JSON.stringify(result, null, 2),
                    language: 'json'
                });
                vscode.window.showTextDocument(resultDoc);
            });

        } catch (error) {
            vscode.window.showErrorMessage(`Inference failed: ${error}`);
        }
    });

    // View models
    const viewModelsCommand = vscode.commands.registerCommand('lattice.viewModels', () => {
        vscode.commands.executeCommand('workbench.view.extension.lattice');
    });

    // Create project
    const createProjectCommand = vscode.commands.registerCommand('lattice.createProject', async () => {
        const templates = Object.keys(ProjectTemplates);
        const selectedTemplate = await vscode.window.showQuickPick(templates, {
            placeHolder: 'Select a project template'
        });

        if (!selectedTemplate) return;

        const folderUri = await vscode.window.showOpenDialog({
            canSelectFolders: true,
            canSelectFiles: false,
            canSelectMany: false,
            openLabel: 'Select Project Folder'
        });

        if (!folderUri || folderUri.length === 0) return;

        const projectName = await vscode.window.showInputBox({
            prompt: 'Enter project name',
            value: 'my-lattice-project'
        });

        if (!projectName) return;

        try {
            await ProjectTemplates[selectedTemplate](folderUri[0], projectName);
            vscode.window.showInformationMessage(`Project "${projectName}" created successfully!`);

            // Open the new project
            const projectUri = vscode.Uri.joinPath(folderUri[0], projectName);
            vscode.commands.executeCommand('vscode.openFolder', projectUri);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to create project: ${error}`);
        }
    });

    // Run tests
    const runTestsCommand = vscode.commands.registerCommand('lattice.runTests', async () => {
        const terminal = vscode.window.createTerminal({
            name: 'Citrate Tests',
            cwd: vscode.workspace.workspaceFolders?.[0].uri.fsPath
        });

        terminal.sendText('python -m pytest tests/ -v');
        terminal.show();
    });

    // Refresh models
    const refreshCommand = vscode.commands.registerCommand('lattice.refreshModels', () => {
        modelProvider.refresh();
        networkProvider.refresh();
    });

    // Show explorer
    const showExplorerCommand = vscode.commands.registerCommand('lattice.showExplorer', () => {
        vscode.commands.executeCommand('workbench.view.extension.lattice');
    });

    // Register all commands
    context.subscriptions.push(
        connectCommand,
        deployCommand,
        inferenceCommand,
        viewModelsCommand,
        createProjectCommand,
        runTestsCommand,
        refreshCommand,
        showExplorerCommand
    );
}

export function deactivate() {
    if (citrateClient) {
        citrateClient.disconnect();
    }
}