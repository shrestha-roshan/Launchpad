import { AnchorProvider, Program, getProvider } from "@coral-xyz/anchor";
import { Launchpad } from "../target/types/launchpad";
import {
  Keypair,
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
} from "@solana/web3.js";
import { BN } from "bn.js";
import fs from "fs";
import { assert } from "chai";
import * as anchor from "@coral-xyz/anchor";

describe("multi whitelist", async () => {
  const connection = new anchor.web3.Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  );
  const sender = Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(
        fs.readFileSync(
          "/Users/chou/lampbit-contracts/tests/test_wallets/test-wallet.json",
          "utf-8"
        )
      )
    )
  ); // This sender is the auction owner
  console.log("sender:", sender.publicKey.toString());
  // Configure the client to use the devnet cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Launchpad as Program<Launchpad>;

  const programId = program.programId;
  let addresses = [];
  let whitelist_pdas = [];
  const auction_pda_name = "lampbit-auction-edge2";
  const [auction, _] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("auction")),
      Buffer.from(anchor.utils.bytes.utf8.encode(auction_pda_name)),
    ],
    program.programId
  );
  console.log("auction:", auction.toString());

  const [auction_vault, __] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode("auction_vault")),
      auction.toBuffer(),
    ],
    program.programId
  );

  // Test Data
  const test_data = {
    funding_demand: 1782,
    token_amount: 360000000,
    unit_ticket_amount: 400000,
  };

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
        fundingDemand: new BN(test_data.funding_demand),
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
    console.log("tokensInPool", (await auction_data).tokensInPool.toNumber());
    console.log(
      "tokenQuantityPerTicket",
      (await auction_data).tokenQuantityPerTicket.toNumber()
    );
    console.log("fundingDemand", (await auction_data).fundingDemand.toNumber());
  });

  it("should be able to whitelist multiple addresses", async () => {
    for (let i = 0; i < 11; i++) {
      const address = anchor.web3.Keypair.generate().publicKey;

      const [whitelist_pda, ___] = PublicKey.findProgramAddressSync(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("whitelist")),
          address.toBuffer(),
          auction.toBuffer(),
        ],
        program.programId
      );
      addresses.push(address);
      whitelist_pdas.push(whitelist_pda);
    }

    let ixns = [];
    // enumerate through all the addresses and whitelist them
    for (let i = 0; i < addresses.length; i++) {
      let ixn = await program.methods
        .whitelist({
          whitelisted: true,
        })
        .accounts({
          creator: sender.publicKey,
          whitelistPda: whitelist_pdas[i],
          auction: auction,
          whitelistUser: addresses[i],
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
        })
        .signers([sender])
        .instruction();
      ixns.push(ixn);
    }

    const tx = new anchor.web3.Transaction().add(...ixns);
    tx.recentBlockhash = (
      await provider.connection.getRecentBlockhash()
    ).blockhash;
    tx.feePayer = sender.publicKey;
    tx.sign(sender);
    const tx_ser = tx.serialize();
    const txid = await connection.sendRawTransaction(tx_ser);
    console.log("txid", txid);

    // wait for the transaction to be confirmed
    await provider.connection.confirmTransaction(txid);

    // check if all the addresses are whitelisted
    for (let i = 0; i < addresses.length; i++) {
      const whitelist_data = await program.account.whitelist.fetch(
        whitelist_pdas[i]
      );
      assert.ok(whitelist_data.whitelisted);
    }
  });
});
