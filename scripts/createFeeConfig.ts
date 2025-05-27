import dotenv from "dotenv";

dotenv.config();

import * as anchor from "@coral-xyz/anchor";
import { getAdminKeypair, getConnection } from "./common";
import {
  createFeeConfig,
  getFeeGovernanceHubProgram,
} from "fee-governance-hub-ts";
import { sendAndConfirmTransaction } from "@solana/web3.js";

// TODO: change the parameters
const targetProgram = new anchor.web3.PublicKey(
  "E31gwCAUYSifpGMFwjpQxjJCny5v8mZTGj7zWXwZoMxX" // spl_fishing program id
);
const feeInstructionIndex = 0;
const isUsingGlobalFeeWallets = false;
const feeWallets = [
  {
    address: "ArpaDqpkJpKfxLP7WoFvYMbkj33C1PAHcy8tyrxFpgrc",
    feePercent: 0.8, // 80%
  },
  {
    address: "DAgVzrqoH1GtwawKnZf4Z1tWeAZKiXQZ6AxYuKBJZ5HQ",
    feePercent: 0.2, // 20%
  },
];
const feeAmount = 10_000_000; // 0.01 SOL
const feeInstructionName = "create_game";

(async () => {
  try {
    const tx = new anchor.web3.Transaction();
    const connection = getConnection();

    const adminKeypair = getAdminKeypair();

    const program = getFeeGovernanceHubProgram(connection);

    const createConfigIx = await createFeeConfig(
      adminKeypair.publicKey,
      targetProgram,
      feeInstructionIndex,
      isUsingGlobalFeeWallets,
      feeWallets,
      feeAmount,
      feeInstructionName,
      program
    );

    tx.add(createConfigIx);

    tx.feePayer = adminKeypair.publicKey;

    const txSignature = await sendAndConfirmTransaction(
      connection,
      tx,
      [adminKeypair],
      {
        commitment: "confirmed",
        skipPreflight: false,
      }
    );

    console.log("\nFee config created successfully!");
    console.log(txSignature);
  } catch (e) {
    console.error(e);
  }
})();
