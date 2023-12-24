import { PublicKey, Keypair } from "@solana/web3.js"
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";

export async function createATA(
    payer: Keypair,
    ata_address: PublicKey,
    ata_owner: PublicKey,
    token_mint: PublicKey
) {
    //  instruction for creating auction vault ATA
    const ata_inx = createAssociatedTokenAccountInstruction(
        payer.publicKey,
        ata_address,
        ata_owner,
        token_mint
    );
    const tnx = new anchor.web3.Transaction().add(ata_inx);
    const tnx_sig = await anchor.AnchorProvider.env().sendAndConfirm(tnx, [payer]);
    console.log("ATA Creation Tnx Sig:", tnx_sig)
    }