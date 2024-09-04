import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {DispenserProgram} from "../target/types/dispenser_program"
import { Keypair } from "@solana/web3.js"
import { randomBytes } from "crypto";

describe("dispense", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider); 
  const program = anchor.workspace.DispenserProgram as Program<DispenserProgram>;

  it("Initializes the escrow", async () => {
    const host = anchor.web3.Keypair.generate();
    const airdropSignature = await provider.connection.requestAirdrop(
      host.publicKey,
      4 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);

    const hostBal = await provider.connection.getBalance(host.publicKey)
    console.log(parseInt(hostBal.toString()))

    const winner1 = new Keypair();
    const winner2 = new Keypair();
    const winners = [winner1.publicKey, winner2.publicKey];
    const prizes: anchor.BN[] = [
      new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL),
      new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL),
    ];

    const escrow_id1 = new anchor.BN(1);
    const [escrowPda, escrowBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow"), host.publicKey.toBuffer(), escrow_id1.toArrayLike(Buffer, "le", 8)], 
      program.programId
    );
    const [vaultPda, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow_vault"), host.publicKey.toBuffer(), escrow_id1.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // Logging derived PDA and bumps
    console.log("Escrow PDA:", escrowPda.toString());
    console.log("Vault PDA:", vaultPda.toString());
    console.log("Escrow Bump:", escrowBump);
    console.log("Vault Bump:", vaultBump);

    await program.methods.initializeEscrow(escrow_id1,winners,prizes)
      .accountsStrict({
        host: host.publicKey,
        escrow: escrowPda,
        escrowVault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([host])
      .rpc();

    const afterEscrowBalance = await provider.connection.getBalance(vaultPda);
    console.log("Initial Escrow balance: ", afterEscrowBalance / anchor.web3.LAMPORTS_PER_SOL);

    await program.methods.withdrawPrize(escrow_id1, winner1.publicKey)
      .accountsStrict({
        escrow: escrowPda,
        escrowVault: vaultPda,
        winner: winner1.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([winner1])
      .rpc();

    const afterWinner1Balance = await provider.connection.getBalance(winner1.publicKey);
    console.log("After winner balance: ", afterWinner1Balance / anchor.web3.LAMPORTS_PER_SOL);
  });

  it("Initializes the escrow two times", async () => {
    const host = anchor.web3.Keypair.generate();
    const airdropSignature = await provider.connection.requestAirdrop(
      host.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);

    const hostBal = await provider.connection.getBalance(host.publicKey)
    console.log(parseInt(hostBal.toString()))

    const winner1 = new Keypair();
    const winner2 = new Keypair();
    const winners = [winner1.publicKey, winner2.publicKey];
    const prizes: anchor.BN[] = [
      new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL),
      new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL),
    ];

    const escrow_id1 = new anchor.BN(1);
    const [escrowPda1, escrowBump1] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow"), host.publicKey.toBuffer(), escrow_id1.toArrayLike(Buffer, "le", 8)], 
      program.programId
    );
    const [vaultPda1, vaultBump1] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow_vault"), host.publicKey.toBuffer(), escrow_id1.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const escrow_id2 = new anchor.BN(2);
    const [escrowPda2, escrowBump2] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow"), host.publicKey.toBuffer(), escrow_id2.toArrayLike(Buffer, "le", 8)], 
      program.programId
    );
    const [vaultPda2, vaultBump2] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow_vault"), host.publicKey.toBuffer(), escrow_id2.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // Logging derived PDA and bumps
    console.log("Escrow PDA:", escrowPda1.toString());
    console.log("Vault PDA:", vaultPda1.toString());
    console.log("Escrow Bump:", escrowBump1);
    console.log("Vault Bump:", vaultBump1);

    await program.methods.initializeEscrow(escrow_id1,winners,prizes)
      .accountsStrict({
        host: host.publicKey,
        escrow: escrowPda1,
        escrowVault: vaultPda1,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([host])
      .rpc();

      await program.methods.initializeEscrow(escrow_id2,winners,prizes)
      .accountsStrict({
        host: host.publicKey,
        escrow: escrowPda2,
        escrowVault: vaultPda2,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([host])
      .rpc();


    const afterEscrowBalance = await provider.connection.getBalance(vaultPda1);
    console.log("Initial Escrow balance: ", afterEscrowBalance / anchor.web3.LAMPORTS_PER_SOL);

    const afterWinner1Balance = await provider.connection.getBalance(winner1.publicKey);
    console.log("After winner balance: ", afterWinner1Balance / anchor.web3.LAMPORTS_PER_SOL);
  });
});


