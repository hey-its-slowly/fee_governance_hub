import * as anchor from "@coral-xyz/anchor";
import { MemcmpFilter } from "@solana/web3.js";
import { InstructionFeeConfig } from "./types/InstructionFeeConfig";
import { FeeGovernanceHub } from "../idl/fee_governance_hub";
import { CONFIG_TAG, getFeeGovernanceHubProgram } from "./constants";
import * as FeeGovernanceHubIdl from "../idl/fee_governance_hub.json";

export const parseInstructionFeeConfig = (
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  rawInstructionFeeConfig: any,
  key: anchor.web3.PublicKey
): InstructionFeeConfig => {
  const instructionFeeConfig = {
    key: key.toString(),
    bump: rawInstructionFeeConfig.bump,
    program: rawInstructionFeeConfig.program.toString(),
    feeInstructionIndex: rawInstructionFeeConfig.feeInstructionIndex,
    isUsingGlobalFeeWallets: rawInstructionFeeConfig.isUsingGlobalFeeWallets,
    feeWallets: rawInstructionFeeConfig.feeWallets.map((wallet: any) => ({
      address: wallet.address.toString(),
      feePercent: wallet.feePercent,
    })),
    feeAmount: rawInstructionFeeConfig.feeAmount,
    feeInstructionName: rawInstructionFeeConfig.feeInstructionName.toString(),
    createdAt: rawInstructionFeeConfig.createdAt,
  };

  return instructionFeeConfig;
};

export const getInstructionFeeConfigByKey = async (
  key: anchor.web3.PublicKey,
  program: anchor.Program<FeeGovernanceHub>
): Promise<InstructionFeeConfig | null> => {
  const rawInstructionFeeConfig = await program.account.config.fetchNullable(
    key
  );
  return parseInstructionFeeConfig(rawInstructionFeeConfig, key);
};

export const getFeeConfigByProgramAndAndInstructionIndex = async (
  programId: anchor.web3.PublicKey,
  instructionIndex: number,
  connection: anchor.web3.Connection
): Promise<InstructionFeeConfig | null> => {
  const program = getFeeGovernanceHubProgram(connection);
  if (!program) return null;

  const [configKey] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      CONFIG_TAG,
      programId.toBuffer(),
      new anchor.BN(instructionIndex).toArrayLike(Buffer, "le", 8),
    ],
    program.programId
  );

  const rawInstructionFeeConfig = await program.account.config.fetchNullable(
    configKey
  );

  if (!rawInstructionFeeConfig) return null;

  return parseInstructionFeeConfig(rawInstructionFeeConfig, configKey);
};

export const getFeeConfigsByProgram = async (
  programId: anchor.web3.PublicKey,
  connection: anchor.web3.Connection
): Promise<InstructionFeeConfig[]> => {
  const program = getFeeGovernanceHubProgram(connection);
  if (!program) return [];

  const coder = new anchor.BorshAccountsCoder(
    FeeGovernanceHubIdl as unknown as FeeGovernanceHub
  );
  const discriminator = coder.accountDiscriminator("Config");
  const filters: MemcmpFilter[] = [];
  filters.push({
    memcmp: {
      offset: discriminator.length + 1,
      bytes: programId.toBase58(),
    },
  });

  const configs = await program.account.config.all();

  return configs.map((config) =>
    parseInstructionFeeConfig(config.account, config.publicKey)
  );
};
