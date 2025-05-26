export interface FeeWallet {
  address: string;
  feePercent: number;
}

export interface InstructionFeeConfig {
  key: string;
  bump: number;
  program: string;
  feeInstructionIndex: number;
  isUsingGlobalFeeWallets: boolean;
  feeWallets: FeeWallet[];
  feeAmount: number;
  feeInstructionName: string;
  createdAt: number;
}
