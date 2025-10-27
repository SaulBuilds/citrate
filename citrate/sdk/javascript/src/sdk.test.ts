import axios from 'axios';
import { CitrateSDK } from './sdk';

jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('CitrateSDK', () => {
  beforeEach(() => {
    mockedAxios.create.mockReturnValue({
      post: jest.fn().mockResolvedValue({ data: { jsonrpc: '2.0', result: '0x1', id: 1 } }),
    } as any);
  });

  it('merges config defaults', () => {
    const sdk = new CitrateSDK({ rpcEndpoint: 'http://127.0.0.1:8545', chainId: 999 } as any);
    // @ts-ignore
    expect(sdk['config'].rpcEndpoint).toBe('http://127.0.0.1:8545');
    // @ts-ignore
    expect(sdk['config'].chainId).toBe(999);
  });

  it('calls RPC with array params', async () => {
    const sdk = new CitrateSDK({ rpcEndpoint: 'http://127.0.0.1:8545', chainId: 1337 } as any);
    await sdk.getNetworkInfo();
    expect(mockedAxios.create).toHaveBeenCalled();
  });
});

