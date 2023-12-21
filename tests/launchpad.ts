import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Launchpad } from "../target/types/launchpad";
import { BN } from "bn.js";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  getAssociatedTokenAddress, 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import fs from "fs";

describe("anchor-latest", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Launchpad as Program<Launchpad>;
  console.log("programId:", program.programId.toString());

  const sender = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync("./test_wallets/wallet_1.json", "utf-8")))
  );
  console.log("sender:", sender.publicKey.toString());

  const receiver_acc = Keypair.generate();
  console.log("receiver_acc:", receiver_acc.publicKey.toString());

  const auction_pda_name = "lampbit-auction-contract";
  const [auction, _] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("auction")),
      Buffer.from(anchor.utils.bytes.utf8.encode(auction_pda_name)),
    ],
    program.programId
  );
  console.log("auction:", auction.toString());

  const auction_token = new PublicKey("8CSvK7xceqUeqRaPr91r5kgteXGcWmBL48aoUQCtdizq");
  console.log("auction_token", auction_token) 

  it("Init Auction!", async () => {
    // get the timestamp when auction goes LIVE
    const start_time = Math.floor(Date.now() / 1000);
    console.log("start_time:", start_time);

    const unit_price = 1;
    const tokenCap = 2;

    const tx = await program.methods
      .initAuction({
        name: auction_pda_name,
        enabled: true,
        fixedAmount: true,
        startTime: new BN(start_time),
        endTime: new BN(start_time + 1000),
        unitPrice: new BN(unit_price * LAMPORTS_PER_SOL),
        tokenCap: new BN(tokenCap * LAMPORTS_PER_SOL),
        payWithNative: false,
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

  it("Add Token!", async () => {
    const sender_ata_address = await getAssociatedTokenAddress(
        auction_token,
        sender.publicKey,
      );
      console.log("sender_ata_address", sender_ata_address.toString())

      const auction_ata_address = await getAssociatedTokenAddress(
        auction_token,
        auction,
        true
      );
      console.log("auction_ata_address", auction_ata_address.toString())

    const tx = await program.methods.addToken()
    .accounts({
      owner: sender.publicKey,
      auction: auction,
      ownerAuctionTokenAccount: sender_ata_address,
      auctionVaultTokenAccount: auction_ata_address,
      auctionToken: auction_token,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
    }).signers([sender]).rpc();

    console.log("Your transaction signature", tx);

  });

});
