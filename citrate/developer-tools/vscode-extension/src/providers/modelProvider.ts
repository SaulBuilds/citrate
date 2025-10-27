import * as vscode from 'vscode';
import { CitrateClient, LatticeModel } from '../citrateClient';

export class LatticeModelProvider implements vscode.TreeDataProvider<ModelItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<ModelItem | undefined | null | void> = new vscode.EventEmitter<ModelItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ModelItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor(private citrateClient: CitrateClient) {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: ModelItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: ModelItem): Promise<ModelItem[]> {
        if (!this.citrateClient.isConnected) {
            return [];
        }

        if (!element) {
            // Root level - show models
            try {
                const models = await this.citrateClient.getModels();
                return models.map(model => new ModelItem(
                    model.name || model.id,
                    model,
                    vscode.TreeItemCollapsibleState.Collapsed
                ));
            } catch (error) {
                return [new ModelItem('Failed to load models', undefined, vscode.TreeItemCollapsibleState.None)];
            }
        } else if (element.model) {
            // Model details
            const model = element.model;
            return [
                new ModelItem(`ID: ${model.id}`, undefined, vscode.TreeItemCollapsibleState.None),
                new ModelItem(`Version: ${model.version || 'Unknown'}`, undefined, vscode.TreeItemCollapsibleState.None),
                new ModelItem(`Owner: ${this.citrateClient.formatAddress(model.owner || '')}`, undefined, vscode.TreeItemCollapsibleState.None),
                new ModelItem(`Price: ${model.price ? this.citrateClient.formatEther(model.price) : '0'} ETH`, undefined, vscode.TreeItemCollapsibleState.None),
                new ModelItem(`Inferences: ${model.totalInferences || 0}`, undefined, vscode.TreeItemCollapsibleState.None),
                new ModelItem(`Revenue: ${model.totalRevenue ? this.citrateClient.formatEther(model.totalRevenue) : '0'} ETH`, undefined, vscode.TreeItemCollapsibleState.None),
            ];
        }

        return [];
    }
}

class ModelItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly model?: LatticeModel,
        public readonly collapsibleState?: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);

        if (model) {
            this.tooltip = model.description || `Model ${model.id}`;
            this.description = model.status || 'active';
            this.contextValue = 'model';

            // Add commands
            this.command = {
                command: 'lattice.runInference',
                title: 'Run Inference',
                arguments: [model]
            };
        }
    }
}