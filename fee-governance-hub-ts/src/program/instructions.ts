import * as anchor from "@coral-xyz/anchor";
import { FeeGovernanceHub } from "../idl/fee_governance_hub";
import { FeeWallet } from "./types";
import {
  MAX_FEE_INSTRUCTION_NAME_LEN,
  MAX_FEE_WALLETS_LEN,
  PERCENT_DENOMINATOR,
} from "./constants";

export const createFeeConfig = async (
  authority: anchor.web3.PublicKey,
  targetProgram: anchor.web3.PublicKey,
  feeInstructionIndex: number,
  isUsingGlobalFeeWallets: boolean,
  feeWallets: FeeWallet[],
  feeAmount: number,
  feeInstructionName: string,
  program: anchor.Program<FeeGovernanceHub>
) => {
  if (feeWallets.length > MAX_FEE_WALLETS_LEN) {
    throw new Error(
      "Fee wallets length must be less than or equal to " + MAX_FEE_WALLETS_LEN
    );
  }

  if (feeInstructionName.length > MAX_FEE_INSTRUCTION_NAME_LEN) {
    throw new Error(
      "Fee instruction name must be less than or equal to " +
        MAX_FEE_INSTRUCTION_NAME_LEN
    );
  }

  const ix = await program.methods
    .createConfig({
      feeInstructionIndex: new anchor.BN(feeInstructionIndex),
      isUsingGlobalFeeWallets,
      feeWallets: feeWallets.map((wallet) => ({
        address: new anchor.web3.PublicKey(wallet.address),
        feePercent: new anchor.BN(wallet.feePercent * PERCENT_DENOMINATOR),
      })),
      feeAmount: new anchor.BN(feeAmount),
      feeInstructionName,
    })
    .accounts({
      authority,
      targetProgram,
    })
    .instruction();

  return ix;
};

export const updateFeeConfig = async (
  authority: anchor.web3.PublicKey,
  targetProgram: anchor.web3.PublicKey,
  feeInstructionIndex: number,
  isUsingGlobalFeeWallets: boolean,
  feeWallets: FeeWallet[],
  feeAmount: number,
  feeInstructionName: string,
  program: anchor.Program<FeeGovernanceHub>
) => {
  if (feeWallets.length > MAX_FEE_WALLETS_LEN) {
    throw new Error(
      "Fee wallets length must be less than or equal to " + MAX_FEE_WALLETS_LEN
    );
  }

  if (feeInstructionName.length > MAX_FEE_INSTRUCTION_NAME_LEN) {
    throw new Error(
      "Fee instruction name must be less than or equal to " +
        MAX_FEE_INSTRUCTION_NAME_LEN
    );
  }

  const ix = await program.methods
    .updateConfig({
      feeInstructionIndex: new anchor.BN(feeInstructionIndex),
      isUsingGlobalFeeWallets,
      feeWallets: feeWallets.map((wallet) => ({
        address: new anchor.web3.PublicKey(wallet.address),
        feePercent: new anchor.BN(wallet.feePercent * PERCENT_DENOMINATOR),
      })),
      feeAmount: new anchor.BN(feeAmount),
      feeInstructionName,
    })
    .accounts({
      authority,
      targetProgram,
    })
    .instruction();

  return ix;
};
