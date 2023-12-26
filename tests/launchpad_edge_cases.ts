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
  Connection
} from "@solana/web3.js";
import {
  getAssociatedTokenAddress, 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { createATA } from "./utils";
import fs from "fs";
import { assert } from "chai";

describe("anchor-latest", async () => {
  // Configure the client to use the devnet cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const con = new Connection("https://api.devnet.solana.com")
  const program = anchor.workspace.Launchpad as Program<Launchpad>;
  console.log("programId:", program.programId.toString());

  // a function to set timeout or sleep
  const delay = ms => new Promise(res => setTimeout(res, ms));

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
  console.log("buyer:", buyer.publicKey.toString());

  const buyer_bidtoken_ata = new PublicKey("5Q3NSjAYBFNyWL6sJkiz7YidYpBqgxrYTK5nnrfkzFcR");
  console.log("buyer_bidtoken_ata:", buyer_bidtoken_ata.toString());

  const buyer_auctiontoken_ata = await getAssociatedTokenAddress(
    auction_token,
    buyer.publicKey,
  );
  console.log("buyer_auctiontoken_ata", buyer_auctiontoken_ata.toString());

  const auction_pda_name = "lampbit-auction-edge2";
  const [auction, _] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("auction")),
      Buffer.from(anchor.utils.bytes.utf8.encode(auction_pda_name)),
    ],
    program.programId
  );
  console.log("auction:", auction.toString());

  const [buyer_pda, ____] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("buyer")),
      buyer.publicKey.toBuffer(),
      auction.toBuffer()
    ],
    program.programId
  )
  console.log("buyer_pda:", buyer_pda.toString());

  const [auction_vault, __] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("auction_vault")),
      auction.toBuffer()
    ],
    program.programId
  )
  console.log("auction_vault:", auction_vault.toString());

  const auction_vault_ata = await getAssociatedTokenAddress(
    auction_token,
    auction_vault,
    true
  );
  console.log("auction_vault_ata", auction_vault_ata.toString())

  const auction_vault_bidtoken_ata = await getAssociatedTokenAddress(
    bid_token,
    auction_vault,
    true
  );
  console.log("auction_vault_bidtoken_ata", auction_vault_bidtoken_ata.toString())

  const [whitelist_pda, ___] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("whitelist")),
      buyer.publicKey.toBuffer(),
      auction.toBuffer(),
    ],
    program.programId
  );
  console.log("whitelist_pda:", whitelist_pda.toString());

  // Reference Data
  const actual_data = {
    ticker: "$SOBB",
    funding_demand: 1782, // in SOL
    presale_start_time: "Dec 28th, 23:30 [UTC]", // should be in unix 
    presale_end_time: "Dec 29th, 11:30 [UTC]",
    auction_start_time: "Dec 29th, 12:30 [UTC]", 
    auction_end_time: "Dec 29th, 13:30 [UTC]",
    token_amount: 360000000, // total quantity of token inside pool (Constant) 
    token_price: 0.00000495, //in SOL,  ≈ $0.0005
    unit_ticket_amount: 400000, // $SOBB (Constant)
    number_of_tickets: 900,  // [calc: token_amount / unit_ticket_amount]
    ticket_price:  1.98, // 1.98 SOL ≈ $216 [calc: funding_demand / number_of_tickets]
    // This ticket_price is the SOL needed to buy one ticket
  }

  // Test Data
  const test_data = {
    funding_demand: 1782,
    token_amount: 360000000,
    unit_ticket_amount: 400000,
  }
  
  // const auction_data = await program.account.auction.fetch(auction);
  // console.log("fundingDemand", auction_data.fundingDemand.toNumber())
  // console.log("tokens_in_pool", auction_data.tokensInPool.toNumber())
  // console.log("tokenQuantityPerTicket", auction_data.tokenQuantityPerTicket.toNumber())
  // const ticket_price =  auction_data.fundingDemand.toNumber() / (
  //   auction_data.tokensInPool.toNumber() / auction_data.tokenQuantityPerTicket.toNumber()
  // )
  // console.log("ticket_price:", ticket_price)
  // console.log("remainingTokens", auction_data.remainingTokens.toNumber())
  // assert(false)

  describe("ATA SanityCheck!", async () => {
    it("Check whether all required ATAs exists or not!", async () => {
      if (! await con.getAccountInfo(sender_auctiontoken_ata)) {
        console.log("Token Acc doesn't exist!! Creating one...")
        createATA(sender, sender_auctiontoken_ata, sender.publicKey, auction_token)
        console.log(sender_auctiontoken_ata.toString(), "created!")
      } else {
        console.log(sender_auctiontoken_ata.toString(), "exists!")
      }

      if (! await con.getAccountInfo(sender_bidtoken_ata)) {
        console.log("Token Acc doesn't exist!! Creating one...")
        createATA(sender, sender_bidtoken_ata, sender.publicKey, bid_token)
        console.log(sender_bidtoken_ata.toString(), "created!")
      } else {
        console.log(sender_bidtoken_ata.toString(), "exists!")
      }

      if (! await con.getAccountInfo(buyer_auctiontoken_ata)) {
        console.log("Token Acc doesn't exist!! Creating one...")
        createATA(buyer, buyer_auctiontoken_ata, buyer.publicKey, auction_token)
        console.log(buyer_auctiontoken_ata.toString(), "created!")
      } else {
        console.log(buyer_auctiontoken_ata.toString(), "exists!")
      }

      if (! await con.getAccountInfo(auction_vault_ata)) {
        console.log("Token Acc doesn't exist!! Creating one...")
        createATA(sender, auction_vault_ata, auction_vault, auction_token)
        console.log(auction_vault_ata.toString(), "created!")
      } else {
        console.log(auction_vault_ata.toString(), "exists!")
      }

      if (! await con.getAccountInfo(auction_vault_bidtoken_ata)) {
        console.log("Token Acc doesn't exist!! Creating one...")
        createATA(sender, auction_vault_bidtoken_ata, auction_vault, bid_token)
        console.log(auction_vault_bidtoken_ata.toString(), "created!")
      } else {
        console.log(auction_vault_bidtoken_ata.toString(), "exists!")
      }
    })
  });

  describe("Case 1: Init Auction, Add Token, Whitelist, Pre-Sale Buy!", async () => {
    it("Init Auction!", async () => {
        // get the timestamp for auction to go LIVE
        const start_time = Math.floor(Date.now() / 1000);
        console.log("start_time:", start_time);
    
        const init_auc_tx = await program.methods
          .initAuction({
            name: auction_pda_name,
            enabled: true,
            fixedAmount: true,
            startTime: new BN(start_time + 15),
            endTime: new BN(start_time + 25),
            payWithNative: true,
            preSale: true,
            preSaleStartTime: new BN(start_time),
            preSaleEndTime: new BN(start_time + 10),
            tokensInPool: new BN(test_data.token_amount),
            tokenQuantityPerTicket: new BN(test_data.unit_ticket_amount),
            fundingDemand: new BN(test_data.funding_demand)
          })
          .accounts({
            owner: sender.publicKey,
            auction: auction,
            auctionVault: auction_vault,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
          })
          .signers([sender])
          .rpc();
        console.log("init_auc_tx", init_auc_tx);

        const auction_data = program.account.auction.fetch(auction);
        console.log("tokensInPool", (await auction_data).tokensInPool.toNumber())
        console.log("tokenQuantityPerTicket", (await auction_data).tokenQuantityPerTicket.toNumber())
        console.log("fundingDemand", (await auction_data).fundingDemand.toNumber())
    });

    it("Add Token!", async () => {
        const add_token_tx = await program.methods.addToken()
        .accounts({
        owner: sender.publicKey,
        auction: auction,
        auctionVault: auction_vault,
        ownerAuctionTokenAccount: sender_auctiontoken_ata,
        auctionVaultTokenAccount: auction_vault_ata,
        auctionToken: auction_token,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
        }).signers([sender])
        .rpc();
        console.log("add_token_tx", add_token_tx);
    });

    it("Whitelist!", async () => {
        const whitelist_tx = await program.methods.whitelist(
            {
              whitelisted: true,
            }
          )
          .accounts({
              creator: sender.publicKey,
              whitelistPda: whitelist_pda,
              auction: auction,
              whitelistUser: buyer.publicKey,
              rent: SYSVAR_RENT_PUBKEY,
              systemProgram: SystemProgram.programId,
            }).signers([sender])
            .rpc();
        console.log("whitelist_tx", whitelist_tx);
    });

    it("PreSale Buy using SOL!", async () => {
        const presale_buy_tx = await program.methods.preSaleBuyUsingSol()
        .accounts({
            buyer: buyer.publicKey,
            buyerPda: buyer_pda,
            buyerAuctionTokenAccount: buyer_auctiontoken_ata,
            auction: auction,
            auctionVault: auction_vault,
            auctionVaultTokenAccount: auction_vault_ata,
            auctionToken: auction_token,
            whitelistPda: whitelist_pda,
            clock: SYSVAR_CLOCK_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
        }).signers([buyer])
        .rpc();
        console.log("presale_buy_tx", presale_buy_tx);
    });
  });
  // assert(false)
  describe("Case 2: Init Auction(paywithSol), Add Token, Buy Token using Sol, Withdraw Funds!", async () => {
    it("Init Auction!", async () => {
        // get the timestamp when auction goes LIVE
        const start_time = Math.floor(Date.now() / 1000);
        console.log("start_time:", start_time);

        const init_auc_tx = await program.methods
          .initAuction({
            name: auction_pda_name,
            enabled: true,
            fixedAmount: true,
            startTime: new BN(start_time + 7),
            endTime: new BN(start_time + 14),
            payWithNative: true,
            preSale: true,
            preSaleStartTime: new BN(start_time),
            preSaleEndTime: new BN(start_time + 5),
            tokensInPool: new BN(test_data.token_amount),
            tokenQuantityPerTicket: new BN(test_data.unit_ticket_amount),
            fundingDemand: new BN(test_data.funding_demand)
          })
          .accounts({
            owner: sender.publicKey,
            auction: auction,
            auctionVault: auction_vault,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
          })
          .signers([sender])
          .rpc();
        console.log("init_auc_tx", init_auc_tx);
    });

    it("Add Token!", async () => {
        const add_token_tx = await program.methods.addToken()
        .accounts({
        owner: sender.publicKey,
        auction: auction,
        auctionVault: auction_vault,
        ownerAuctionTokenAccount: sender_auctiontoken_ata,
        auctionVaultTokenAccount: auction_vault_ata,
        auctionToken: auction_token,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
        }).signers([sender])
        .rpc();
        console.log("add_token_tx", add_token_tx);
    });

    it("Buy Tokens using Sol!", async () => {
        console.log("Lets wait for Auction to go LIVE...")
        await delay(7000);

        const buy_token_using_spl_tx = await program.methods.buyTokenUsingSol()
        .accounts({
            buyer: buyer.publicKey,
            auction: auction,
            auctionVault: auction_vault,
            buyerPda: buyer_pda,
            auctionVaultTokenAccount: auction_vault_ata,
            buyerAuctionTokenAccount: buyer_auctiontoken_ata,
            auctionToken: auction_token,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            clock: SYSVAR_CLOCK_PUBKEY,
            systemProgram: SystemProgram.programId
          }).signers([buyer])
          .rpc();
          console.log("buy_token_using_spl_tx", buy_token_using_spl_tx);
    });

    it("Withdraw Funds!", async () => {
    console.log("Waiting for Auction to End...")
    await delay(7000);

    const withdraw_funds_tx = await program.methods.withdrawFunds()
    .accounts({
        creator: sender.publicKey,
        auction: auction,
        auctionVault: auction_vault,
        auctionVaultTokenAccount: auction_vault_ata,
        creatorAuctionTokenAccount: sender_auctiontoken_ata,
        auctionToken: auction_token,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId
        }).signers([sender])
        .rpc();
        console.log("withdraw_funds_tx", withdraw_funds_tx);
    });

  });
});
