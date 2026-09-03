import { Contract, rpc, scValToNative } from '@stellar/stellar-sdk';

export interface MultisigProposalData {
  id: bigint;
  proposer: string;
  target: string;
  amount: bigint;
  approvals: string[];
  threshold: number;
  executed: boolean;
}

export class MultisigClient {
  private contract: Contract;
  private server: rpc.Server;

  constructor(private contractId: string, private rpcUrl: string) {
    this.contract = new Contract(contractId);
    this.server = new rpc.Server(rpcUrl);
  }

  /**
   * Query proposal details by ID
   */
  async getProposal(proposalId: number): Promise<MultisigProposalData | null> {
    const tx = this.contract.call('get_proposal', scValToNative(proposalId));
    const result = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result.retval) as MultisigProposalData;
    }
    return null;
  }
}
