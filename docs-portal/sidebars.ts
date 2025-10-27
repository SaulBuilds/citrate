export default {
  docs: [
    'intro',
    {
      type: 'category', label: 'Overview', items: [
        'overview/vision', 'overview/architecture', 'overview/tokenomics'
      ]
    },
    { type: 'category', label: 'Users', items: [
      'users/what-is-citrate', 'users/getting-started-wallet', 'users/rewards'
    ]},
    { type: 'category', label: 'Developers', items: [
      'developers/quickstart', 'developers/contracts', 'developers/rpc', 'developers/sdk/javascript', 'developers/testing'
    ]},
    { type: 'category', label: 'Node Operators', items: [
      'operators/run-node', 'operators/multinode', 'operators/monitoring'
    ]},
    { type: 'category', label: 'AI Providers', items: [
      'providers/register-models', 'providers/pricing', 'providers/quality-proof'
    ]},
    { type: 'category', label: 'Security', items: [
      'security/audits', 'security/threat-model'
    ]},
    'faq'
  ]
};

