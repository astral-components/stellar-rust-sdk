import { Contract, rpc, scValToNative } from '@stellar/stellar-sdk';

export interface RecipientShare {
  recipient: string;
  bps: number; // Basis points out of 10,000
}

export class SplitterClient {
  private contract: Contract;
  private server: rpc.Server;

  constructor(private contractId: string, private rpcUrl: string) {
    this.contract = new Contract(contractId);
    this.server = new rpc.Server(rpcUrl);
  }

  /**
   * Fetch current recipient allocation shares
   */
  async getShares(): Promise<RecipientShare[]> {
    const tx = this.contract.call('get_shares');
    const result = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result.retval) as RecipientShare[];
    }
    return [];
  }
}
