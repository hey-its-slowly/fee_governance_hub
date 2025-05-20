## How to run 

Environment setup and deploy:

1. Install Solana CLI(1.18.23): https://docs.solana.com/de/cli/install-solana-cli-tools. Or `solana-install init 1.18.23`. Or `sh -c "$(curl -sSfL https://release.anza.xyz/v1.18.23/install)"`
2. Install Anchor(0.30.1): https://www.anchor-lang.com/docs/installation
3. In the terminal run: `yarn install` to install the node packages needed to run the tests.
4. Then `anchor build`
5. Copy the deployed program id from the terminal(`anchor keys list` will show the program id) and paste it into the lib.rs and the anchor.toml file. Change admin wallets in constant.rs
6. Then: `anchor build` and `anchor deploy`. or `solana program deploy target/deploy/fee_governance_hub.so`


Contract configuration with admin wallet:

1. Change .env.example to .env. Change wallets and network in .env.
2. in `programs/fee_governance_hub/src/constant.rs`, change the admin wallet address.
