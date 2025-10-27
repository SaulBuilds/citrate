export function Footer() {
  return (
    <footer className="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
      <div className="container mx-auto px-4 py-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          <div>
            <h3 className="font-bold text-gray-900 dark:text-white mb-4">
              Citrate Explorer
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Explore the AI-native Layer-1 BlockDAG with GhostDAG consensus
            </p>
          </div>
          
          <div>
            <h4 className="font-semibold text-gray-900 dark:text-white mb-3">
              Blockchain
            </h4>
            <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
              <li><a href="/blocks" className="hover:text-blue-500">Blocks</a></li>
              <li><a href="/transactions" className="hover:text-blue-500">Transactions</a></li>
              <li><a href="/accounts" className="hover:text-blue-500">Accounts</a></li>
            </ul>
          </div>
          
          <div>
            <h4 className="font-semibold text-gray-900 dark:text-white mb-3">
              AI/ML
            </h4>
            <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
              <li><a href="/models" className="hover:text-blue-500">Models</a></li>
              <li><a href="/inferences" className="hover:text-blue-500">Inferences</a></li>
              <li><a href="/proofs" className="hover:text-blue-500">Proofs</a></li>
            </ul>
          </div>
          
          <div>
            <h4 className="font-semibold text-gray-900 dark:text-white mb-3">
              Resources
            </h4>
            <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
              <li><a href="/api-docs" className="hover:text-blue-500">API Docs</a></li>
              <li><a href="https://github.com/lattice" className="hover:text-blue-500">GitHub</a></li>
              <li><a href="/status" className="hover:text-blue-500">Status</a></li>
            </ul>
          </div>
        </div>
        
        <div className="mt-8 pt-8 border-t border-gray-200 dark:border-gray-700">
          <p className="text-center text-sm text-gray-600 dark:text-gray-400">
            © 2024 Citrate v3. All rights reserved.
          </p>
        </div>
      </div>
    </footer>
  );
}