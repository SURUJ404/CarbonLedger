import { Injectable } from '@nestjs/common';
import { SorobanRpc, Contract, TransactionBuilder, Networks, BASE_FEE } from '@stellar/stellar-sdk';

// Thin wrapper around the Soroban RPC server used by every module that
// needs to read or submit a transaction against one of the four
// CarbonLedger contracts (registry / credit / marketplace / oracle).
@Injectable()
export class SorobanService {
  private server: SorobanRpc.Server;
  private network: string;

  constructor() {
    const rpcUrl = process.env.SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org';
    this.server = new SorobanRpc.Server(rpcUrl, { allowHttp: rpcUrl.startsWith('http://') });
    this.network = process.env.STELLAR_NETWORK === 'public' ? Networks.PUBLIC : Networks.TESTNET;
  }

  getContract(contractId: string): Contract {
    return new Contract(contractId);
  }

  async buildTransaction(sourcePubKey: string) {
    const account = await this.server.getAccount(sourcePubKey);
    return new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.network,
    });
  }

  async simulate(tx: any) {
    return this.server.simulateTransaction(tx);
  }

  async submit(signedTx: any) {
    return this.server.sendTransaction(signedTx);
  }

  async getTransactionStatus(hash: string) {
    return this.server.getTransaction(hash);
  }
}
