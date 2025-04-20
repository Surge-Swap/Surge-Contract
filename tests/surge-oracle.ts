import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SurgeOracle } from "../target/types/surge_oracle";

describe("surge-oracle", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.surgeOracle as Program<SurgeOracle>;
  const provider = anchor.getProvider();

  let volatilityStats = anchor.web3.Keypair.generate();
  let authority = (provider.wallet as anchor.Wallet).payer;

  it("Is initialized!", async () => {
    await program.methods
      .initializeVolatilityStats()
      .accounts({
        volatilityStats: volatilityStats.publicKey,
        authority: authority.publicKey,
      })
      .signers([volatilityStats])
      .rpc();
  });
});
