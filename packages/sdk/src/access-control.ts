import { Contract, rpc, scValToNative, xdr } from '@stellar/stellar-sdk';

export interface AccessControlConfig {
  contractId: string;
  rpcUrl: string;
  networkPassphrase: string;
}

export class AccessControlClient {
  private contract: Contract;
  private server: rpc.Server;

  constructor(private config: AccessControlConfig) {
    this.contract = new Contract(config.contractId);
    this.server = new rpc.Server(config.rpcUrl);
  }

  /**
   * Check if a specific account holds a given role
   */
  async hasRole(account: string, role: string): Promise<boolean> {
    const tx = this.contract.call(
      'has_role',
      xdr.ScVal.scvVec([
        new Address(account).toScVal(),
        xdr.ScVal.scvSymbol(role),
      ])
    );
    const result = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result.retval) as boolean;
    }
    return false;
  }

  /**
   * Get contract admin address
   */
  async getAdmin(): Promise<string> {
    const tx = this.contract.call('get_admin');
    const result = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result.retval) as string;
    }
    throw new Error('Failed to fetch admin');
  }
}
