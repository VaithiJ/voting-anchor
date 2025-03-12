import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VotingSystem } from "../target/types/voting_system";
import { PublicKey, Keypair, SystemProgram, Transaction } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";

describe("Voting System", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.VotingSystem as Program<VotingSystem>;
  const wallet = provider.wallet as anchor.Wallet;

  const voteMint = Keypair.generate();
  const voter = Keypair.generate();
  const votingState = Keypair.generate();
  let voterTokenAccount: PublicKey;
  let userCoinTokenAccount: PublicKey;
  let userPcTokenAccount: PublicKey;
  let userLpTokenAccount: PublicKey;

  const raydiumProgramId = new PublicKey("CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW");
  const amm = Keypair.generate();
  const ammOpenOrders = Keypair.generate();
  const ammTargetOrders = Keypair.generate();
  const poolLpMint = Keypair.generate();
  const poolTokenCoin = Keypair.generate();
  const poolTokenPc = Keypair.generate();
  const serumMarket = Keypair.generate();
  const coinMint = Keypair.generate();
  const pcMint = Keypair.generate();

  before(async () => {
    const connection = provider.connection;

    await createMint(connection, wallet.payer, wallet.publicKey, null, 6, voteMint);
    await createMint(connection, wallet.payer, wallet.publicKey, null, 6, coinMint);
    await createMint(connection, wallet.payer, wallet.publicKey, null, 6, pcMint);
    await createMint(connection, wallet.payer, wallet.publicKey, null, 6, poolLpMint);

    voterTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      voteMint.publicKey,
      voter.publicKey
    ).then((acc) => acc.address);

    userCoinTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      coinMint.publicKey,
      voter.publicKey
    ).then((acc) => acc.address);

    userPcTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      pcMint.publicKey,
      voter.publicKey
    ).then((acc) => acc.address);

    userLpTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      poolLpMint.publicKey,
      voter.publicKey
    ).then((acc) => acc.address);

    await mintTo(connection, wallet.payer, coinMint.publicKey, userCoinTokenAccount, wallet.publicKey, 10_000_000);
    await mintTo(connection, wallet.payer, pcMint.publicKey, userPcTokenAccount, wallet.publicKey, 10_000_000);

    const rent = await connection.getMinimumBalanceForRentExemption(100);
    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: amm.publicKey,
        lamports: rent,
        space: 100,
        programId: SystemProgram.programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: ammOpenOrders.publicKey,
        lamports: rent,
        space: 100,
        programId: SystemProgram.programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: ammTargetOrders.publicKey,
        lamports: rent,
        space: 100,
        programId: SystemProgram.programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: poolTokenCoin.publicKey,
        lamports: rent,
        space: 100,
        programId: SystemProgram.programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: poolTokenPc.publicKey,
        lamports: rent,
        space: 100,
        programId: SystemProgram.programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: serumMarket.publicKey,
        lamports: rent,
        space: 100,
        programId: SystemProgram.programId,
      })
    );
    await provider.sendAndConfirm(tx, [amm, ammOpenOrders, ammTargetOrders, poolTokenCoin, poolTokenPc, serumMarket]);
  });

  it("Initializes the voting session", async () => {
    const tx = await program.methods
      .initialize(["Alice", "Bob", "Charlie"])
      .accounts({
        votingState: votingState.publicKey,
        authority: wallet.publicKey,
        voteMint: voteMint.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([votingState])
      .rpc();

    console.log("Initialization tx:", tx);
    const votingAccount = await program.account.votingState.fetch(votingState.publicKey);
    assert.equal(votingAccount.candidates.length, 3, "Should have 3 candidates");
    assert.equal(votingAccount.isActive, true, "Voting should be active");
  });

  it("Casts a vote", async () => {
    const amountCoin = new anchor.BN(1_000_000);
    const amountPc = new anchor.BN(1_000_000);
  
    const tx = await program.methods
      .castVote(0, amountCoin, amountPc)
      .accounts({
        votingState: votingState.publicKey,
        voter: voter.publicKey,
        voteMint: voteMint.publicKey,
        voterTokenAccount: voterTokenAccount,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        raydiumProgram: raydiumProgramId,
        splTokenProgram: TOKEN_PROGRAM_ID,
        amm: amm.publicKey,
        ammOpenOrders: ammOpenOrders.publicKey,
        ammTargetOrders: ammTargetOrders.publicKey,
        poolLpMint: poolLpMint.publicKey,
        poolTokenCoin: poolTokenCoin.publicKey,
        poolTokenPc: poolTokenPc.publicKey,
        serumMarket: serumMarket.publicKey,
        userCoinTokenAccount: userCoinTokenAccount,
        userPcTokenAccount: userPcTokenAccount,
        userLpTokenAccount: userLpTokenAccount,
      })
      .signers([voter])
      .rpc();
  
    console.log("Vote tx:", tx);
  
    // Verify voting state
    const votingAccount = await program.account.votingState.fetch(votingState.publicKey);
    assert.ok(votingAccount.candidates[0].voteCount.eq(new anchor.BN(1)), "Alice should have 1 vote");
    assert.equal(votingAccount.voters.length, 1, "Voter should be recorded");
  
    // Verify voter token balance
    const voterToken = await provider.connection.getTokenAccountBalance(voterTokenAccount);
    assert.equal(voterToken.value.uiAmount, 1, "Voter should receive 1 VOTE token");
  });

  it("Prevents double voting", async () => {
    const amountCoin = new anchor.BN(1_000_000);
    const amountPc = new anchor.BN(1_000_000);

    try {
      await program.methods
        .castVote(0, amountCoin, amountPc)
        .accounts({
          votingState: votingState.publicKey,
          voter: voter.publicKey,
          voteMint: voteMint.publicKey,
          voterTokenAccount: voterTokenAccount,
          authority: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          raydiumProgram: raydiumProgramId,
          splTokenProgram: TOKEN_PROGRAM_ID,
          amm: amm.publicKey,
          ammOpenOrders: ammOpenOrders.publicKey,
          ammTargetOrders: ammTargetOrders.publicKey,
          poolLpMint: poolLpMint.publicKey,
          poolTokenCoin: poolTokenCoin.publicKey,
          poolTokenPc: poolTokenPc.publicKey,
          serumMarket: serumMarket.publicKey,
          userCoinTokenAccount: userCoinTokenAccount,
          userPcTokenAccount: userPcTokenAccount,
          userLpTokenAccount: userLpTokenAccount,
        })
        .signers([voter])
        .rpc();
      assert.fail("Should have failed due to double voting");
    } catch (err) {
      assert.include(err.toString(), "AlreadyVoted", "Expected AlreadyVoted error");
    }
  });

  it("Ends the voting session", async () => {
    const tx = await program.methods
      .endVoting()
      .accounts({
        votingState: votingState.publicKey,
        authority: wallet.publicKey,
      })
      .rpc();

    console.log("End voting tx:", tx);
    const votingAccount = await program.account.votingState.fetch(votingState.publicKey);
    assert.equal(votingAccount.isActive, false, "Voting should be ended");
  });

  it("Fetches the results", async () => {
    const results = await program.methods
      .getResults()
      .accounts({
        votingState: votingState.publicKey,
      })
      .view();

    console.log("Results:", results.map((c: any) => ({ name: c.name, votes: c.voteCount.toNumber() })));
    assert.equal(results.length, 3, "Should return 3 candidates");
    assert.ok(results[0].voteCount.eq(new anchor.BN(1)), "Alice should have 1 vote");
    assert.ok(results[1].voteCount.eq(new anchor.BN(0)), "Bob should have 0 votes");
    assert.ok(results[2].voteCount.eq(new anchor.BN(0)), "Charlie should have 0 votes");
  });
});