import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Launchpad } from "../target/types/launchpad";
import { BN } from "bn.js";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import {
  getAssociatedTokenAddress, 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction
} from "@solana/spl-token";
import fs from "fs";
import { assert } from "chai";

describe("anchor-latest", async () => {
  // Configure the client to use the devnet cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Launchpad as Program<Launchpad>;
  console.log("programId:", program.programId.toString());

  // Token that the auction owner sells during Auction
  const auction_token = new PublicKey("8CSvK7xceqUeqRaPr91r5kgteXGcWmBL48aoUQCtdizq");
  console.log("auction_token:", auction_token.toString());
  // Token that the buyer uses to bid during Auction
  const bid_token = new PublicKey("6YMTJpgraqrd68mBfjkwG65FPuHiZWuifi4UP1WUoHjK");
  console.log("bid_token:", bid_token.toString());

  const sender = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync("./test_wallets/auction_owner_wallet.json", "utf-8")))
  );  // This sender is the auction owner
  console.log("sender:", sender.publicKey.toString());

  const sender_auctiontoken_ata = await getAssociatedTokenAddress(
    auction_token,
    sender.publicKey,
  );
  console.log("sender_auctiontoken_ata", sender_auctiontoken_ata.toString())

  const sender_bidtoken_ata = await getAssociatedTokenAddress(
    bid_token,
    sender.publicKey,
  );
  console.log("sender_bidtoken_ata", sender_bidtoken_ata.toString())

  const buyer = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync("./test_wallets/buyer_wallet.json", "utf-8")))
  );  // This is the buyer who buys/bids auction token
  // Bb3iXK1aCpSA6wYFsUULmTKXqpCoRMxztJ8XAqRbWMFx
  console.log("buyer:", buyer.publicKey.toString());
  const buyer_bidtoken_ata = new PublicKey("5Q3NSjAYBFNyWL6sJkiz7YidYpBqgxrYTK5nnrfkzFcR");
  console.log("buyer_bidtoken_ata:", buyer_bidtoken_ata.toString());

  // NOTE: Initially, buyer doesn't have buyer_auction_token_account.
  // We can check and create buyer_auction_token_account in SC if it doesn't exists.
  const buyer_auctiontoken_ata = await getAssociatedTokenAddress(
    auction_token,
    buyer.publicKey,
  );
  console.log("buyer_auctiontoken_ata", buyer_auctiontoken_ata.toString());

  const auction_pda_name = "lampbit-auction-contract";
  const [auction, _] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("auction")),
      Buffer.from(anchor.utils.bytes.utf8.encode(auction_pda_name)),
    ],
    program.programId
  );
  console.log("auction:", auction.toString());

  const auction_vault_ata = await getAssociatedTokenAddress(
    auction_token,
    auction,
    true
  );
  console.log("auction_vault_ata", auction_vault_ata.toString())

  const auction_bidtoken_ata = await getAssociatedTokenAddress(
    bid_token,
    auction,
    true
  );
  console.log("auction_bidtoken_ata", auction_bidtoken_ata.toString())

  // a function to set timeout or sleep
  const delay = ms => new Promise(res => setTimeout(res, ms));

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
        endTime: new BN(start_time + 7),
        unitPrice: new BN(unit_price),
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
    const tx = await program.methods.addToken()
    .accounts({
      owner: sender.publicKey,
      auction: auction,
      ownerAuctionTokenAccount: sender_auctiontoken_ata,
      auctionVaultTokenAccount: auction_vault_ata,
      auctionToken: auction_token,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
    }).signers([sender])
    .rpc();

    console.log("Your transaction signature", tx);
  });

  it("Buy Tokens using SPL!", async () => {
    const bidding_spl_amount = 1;

    const tx = await program.methods.buyTokenUsingSpl(
      new BN(bidding_spl_amount * LAMPORTS_PER_SOL)
      ).accounts({
        buyer: buyer.publicKey,
        buyerBidTokenAccount: buyer_bidtoken_ata,
        buyerAuctionTokenAccount: buyer_auctiontoken_ata,
        auction: auction,
        auctionVaultTokenAccount: auction_vault_ata,
        auctionVaultBidTokenAccount: auction_bidtoken_ata,
        auctionToken: auction_token,
        bidToken: bid_token,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
      }).signers([buyer])
      .rpc();

      console.log("Your transaction signature", tx);
  });

  it("Withdraw Funds!", async () => {
    console.log("Waiting for 7 secs...")
    await delay(8000);
    console.log("7 secs Over")

    const tx = await program.methods.withdrawFunds()
    .accounts({
        creator: sender.publicKey,
        auction: auction,
        auctionVaultTokenAccount: auction_vault_ata,
        auctionVaultBidTokenAccount: auction_bidtoken_ata,
        creatorAuctionTokenAccount: sender_auctiontoken_ata,
        creatorBidTokenAccount: sender_bidtoken_ata,
        auctionToken: auction_token,
        bidToken: bid_token,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
      }).signers([sender])
      .rpc();

      console.log("Your transaction signature", tx);
  });

});
