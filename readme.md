## How to run

Environment setup and deploy:

1. Install Solana CLI(1.18.23): https://docs.solana.com/de/cli/install-solana-cli-tools. Or `agave-install init 1.18.23`. Or `sh -c "$(curl -sSfL https://release.anza.xyz/v1.18.23/install)"`
2. Install Anchor(0.30.1): https://www.anchor-lang.com/docs/installation
3. In the terminal run: `yarn install` to install the node packages needed to run the tests.
4. Then `anchor build`
5. Copy the deployed program id from the terminal(`anchor keys list` will show the program id) and paste it into the lib.rs and the anchor.toml file. Change admin wallets in constant.rs
6. Then: `anchor build` and `anchor deploy`. or `solana program deploy target/deploy/fee_governance_hub.so`
7. For devnet build, set `default = ["devnet"]` in `programs/fee_governance_hub/Cargo.toml`

Contract configuration with admin wallet:

1. Change .env.example to .env. Change wallets and network in .env.
2. in `programs/fee_governance_hub/src/constant.rs`, change the admin wallet address.

## Create fee config for an instruction of a consuming program

Note: Fee config is required per instruction that needs to collect fees.
For example, we need 2 fee configs for spl_fishing program: `create_game`(origination fee) and `flip`(transaction fee).

1. Change the parameters in `scripts/createFeeConfig.ts`. `feeInstructionIndex` is the constant(Number type) of the instruction declared in the consuming program.

2. Run `ts-node scripts/createFeeConfig.ts`

## Building and Deploy programs

1. `anchor build --program-name nft_auction`
2. `anchor deploy --program-name nft_auction`