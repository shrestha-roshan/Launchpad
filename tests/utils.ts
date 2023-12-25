import { PublicKey, Keypair, Connection } from "@solana/web3.js"
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { web3 } from "@coral-xyz/anchor";

const connection = new Connection("https://api.devnet.solana.com")

export async function createATA(
    payer: Keypair,
    ata_address: PublicKey,
    ata_owner: PublicKey,
    token_mint: PublicKey
) {
    const ata_inx = createAssociatedTokenAccountInstruction(
        payer.publicKey,
        ata_address,
        ata_owner,
        token_mint
    );
    const tnx = new web3.Transaction().add(ata_inx);
    const tnx_sig = await web3.sendAndConfirmTransaction(
        connection,
        tnx,
        [payer]
    );
    console.log("ATA Creation Tnx Sig:", tnx_sig)
}