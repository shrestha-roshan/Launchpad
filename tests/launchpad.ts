import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Launchpad } from "../target/types/launchpad";
import { BN } from "bn.js";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import fs from "fs";

describe("anchor-latest", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Launchpad as Program<Launchpad>;
  console.log("programId:", program.programId.toString());

  const sender = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync("./test-wallet.json", "utf-8")))
  );
  console.log("sender:", sender.publicKey.toString());

  const receiver_acc = Keypair.generate();
  console.log("receiver_acc:", receiver_acc.publicKey.toString());

  const auction_pda_name = "lampbit-auction";
  const [auction, __] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("auction")),
      Buffer.from(anchor.utils.bytes.utf8.encode(auction_pda_name)),
    ],
    program.programId
  );
  console.log("auction:", auction.toString());

  it("Init Auction!", async () => {
    // get the timestamp at the time of stream
    const start_time = Math.floor(Date.now() / 1000);
    console.log("start_time:", start_time);

    // Add your test here.
    const tx = await program.methods
      .initAuction({
        enabled: true,
        fixedAmount: true,
        name: auction_pda_name,
        startTime: new BN(start_time),
        endTime: new BN(start_time + 1000),
        unitPrice: new BN(10),
        tokenCap: new BN(1000),
      })
      .accounts({
        owner: sender.publicKey,
        auction: auction,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .signers([sender])
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
