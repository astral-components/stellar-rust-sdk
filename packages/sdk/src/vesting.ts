import { Contract, nativeToScVal, rpc, scValToNative } from '@stellar/stellar-sdk';

export interface VestingScheduleData {
  sender: string;
  beneficiary: string;
  token: string;
  totalAmount: bigint;
  startTime: bigint;
  cliffTime: bigint;
  endTime: bigint;
  claimedAmount: bigint;
}

export class VestingClient {
  private contract: Contract;
  private server: rpc.Server;

  constructor(private contractId: string, private rpcUrl: string) {
    this.contract = new Contract(contractId);
    this.server = new rpc.Server(rpcUrl);
  }

  /**
   * Fetch currently vested token balance for a schedule ID
   */
  async getVestedAmount(scheduleId: number): Promise<bigint> {
    const tx = this.contract.call(
      'get_vested_amount',
      nativeToScVal(scheduleId, { type: 'u32' }),
    );
    const result = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationSuccess(result) && result.result) {
      return BigInt(scValToNative(result.result.retval));
    }
    return 0n;
  }

  /**
   * Query details of a specific vesting schedule
   */
  async getSchedule(scheduleId: number): Promise<VestingScheduleData | null> {
    const tx = this.contract.call(
      'get_schedule',
      nativeToScVal(scheduleId, { type: 'u32' }),
    );
    const result = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationSuccess(result) && result.result) {
      return scValToNative(result.result.retval) as VestingScheduleData;
    }
    return null;
  }
}
