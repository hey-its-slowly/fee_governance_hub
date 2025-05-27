import { PublicKey } from "@solana/web3.js";

export interface FeeWallet {
  address: string;
  feePercent: number;
}

export class InstructionFeeConfig {
  key: string;
  bump: number;
  program: string;
  feeInstructionIndex: number;
  isUsingGlobalFeeWallets: boolean;
  feeWallets: FeeWallet[];
  feeAmount: number;
  feeInstructionName: string;
  createdAt: number;

  constructor(
    key: string,
    bump: number,
    program: string,
    feeInstructionIndex: number,
    isUsingGlobalFeeWallets: boolean,
    feeWallets: FeeWallet[],
    feeAmount: number,
    feeInstructionName: string,
    createdAt: number
  ) {
    this.key = key;
    this.bump = bump;
    this.program = program;
    this.feeInstructionIndex = feeInstructionIndex;
    this.isUsingGlobalFeeWallets = isUsingGlobalFeeWallets;
    this.feeWallets = feeWallets;
    this.feeAmount = feeAmount;
    this.feeInstructionName = feeInstructionName;
    this.createdAt = createdAt;
  }

  public getRemainingAccounts(): {
    pubkey: PublicKey;
    isWritable: boolean;
    isSigner: boolean;
  }[] {
    return this.feeWallets.map((wallet) => ({
      pubkey: new PublicKey(wallet.address),
      isWritable: true,
      isSigner: false,
    }));
  }
}
