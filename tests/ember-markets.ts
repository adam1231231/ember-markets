import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EmberMarkets } from "../target/types/ember_markets";
import { BinaryOutcomeTokens } from "../target/types/binary_outcome_tokens";
import { TOKEN_PROGRAM_ID, createAccount, createMint, mintTo } from "@solana/spl-token";
import { BN } from "bn.js";
import { AddressLookupTableProgram, Connection, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";




const CONDITION_AUTH_PDA_SEED = Buffer.from("condition_auth_pda_seed");
const MARKET_AUTH_SEED = Buffer.from("market_auth_seed");

let ticketTokenMint: anchor.web3.PublicKey;
let yesToken: anchor.web3.PublicKey;
let noToken: anchor.web3.PublicKey;
let condition: anchor.web3.PublicKey;
let conditionAuthPda: anchor.web3.PublicKey;
let collateralToken: anchor.web3.PublicKey;
let collateralVault: anchor.web3.PublicKey;
let ticketTokenAta: anchor.web3.PublicKey;
let yesTokenAta: anchor.web3.PublicKey;
let noTokenAta: anchor.web3.PublicKey;
let collateralTokenAta: anchor.web3.PublicKey;
let payer = new anchor.web3.Keypair();

const OPTS: anchor.web3.ConfirmOptions = {
  skipPreflight: true,
};


describe("binary-outcome-tokens", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const EmberProgram = anchor.workspace.EmberMarkets as Program<EmberMarkets>;
  const BOTProgram = anchor.workspace.BinaryOutcomeTokens as Program<BinaryOutcomeTokens>;

  it("Initializing condition", async () => {

    await BOTProgram.provider.connection.confirmTransaction(await BOTProgram.provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000));
    await BOTProgram.provider.connection.confirmTransaction(await BOTProgram.provider.connection.requestAirdrop(BOTProgram.provider.publicKey, 1_000_000_00));

    let conditionKeypair = new anchor.web3.Keypair();

    condition = conditionKeypair.publicKey;

    const [authority, _] = anchor.web3.PublicKey.findProgramAddressSync(
      [CONDITION_AUTH_PDA_SEED, conditionKeypair.publicKey.toBuffer()],
      BOTProgram.programId
    );

    conditionAuthPda = authority;

    ticketTokenMint = await createMint(
      BOTProgram.provider.connection,
      payer,
      conditionAuthPda,
      null,
      0,
    );

    yesToken = await createMint(
      BOTProgram.provider.connection,
      payer,
      conditionAuthPda,
      null,
      0,
    );
    noToken = await createMint(
      BOTProgram.provider.connection,
      payer,
      conditionAuthPda,
      null,
      0,
    );

    // representing usdc with 6 decimals
    collateralToken = await createMint(
      BOTProgram.provider.connection,
      payer,
      payer.publicKey,
      null,
      6,
    )

    let vaultKeypair = new anchor.web3.Keypair();

    collateralVault = vaultKeypair.publicKey;


    collateralTokenAta = await createAccount(
      BOTProgram.provider.connection,
      payer,
      collateralToken,
      BOTProgram.provider.publicKey,
    );

    // minting 100 usdc to the wallet
    await mintTo(
      BOTProgram.provider.connection,
      payer,
      collateralToken,
      collateralTokenAta,
      payer,
      100_000_000
    );

        const tx = await BOTProgram.methods.initializeCondition("test",
            "a random token description",
            "yes",
            "no",
            new anchor.BN(100)).accounts({
            signer: BOTProgram.provider.publicKey,
            condition,
            conditionAuthPda,
            ticketTokenMint,
            outcomeToken1: yesToken,
            outcomeToken2: noToken,
            collateralToken,
            collateralVault,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
            .signers([conditionKeypair, vaultKeypair])
            .
            rpc(OPTS);

  });

  it("Minting tickets", async () => {

    ticketTokenAta = await createAccount(
      BOTProgram.provider.connection,
      payer,
      ticketTokenMint,
      BOTProgram.provider.publicKey,
    );

    const tx = await BOTProgram.methods.mintTicket(new anchor.BN(5)).accounts({
      signer: BOTProgram.provider.publicKey,
      condition,
      collateralVault,
      conditionAuthPda,
      ticketTokenMint,
      payer: collateralTokenAta,
      receiver: ticketTokenAta,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  });

  it("Splitting tickets", async () => {

    yesTokenAta = await createAccount(
      BOTProgram.provider.connection,
      payer,
      yesToken,
      BOTProgram.provider.publicKey,
    );

    noTokenAta = await createAccount(
      BOTProgram.provider.connection,
      payer,
      noToken,
      BOTProgram.provider.publicKey,
    );


    await BOTProgram.methods.splitTicket(new anchor.BN(5)).accounts({
      signer: BOTProgram.provider.publicKey,
      condition,
      conditionAuthPda,
      ticketTokenMint,
      outcome1Token: yesToken,
      outcome2Token: noToken,
      payer: ticketTokenAta,
      receiver1: yesTokenAta,
      receiver2: noTokenAta,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  });

  it("Initialize Market", async () => {
    let market = new anchor.web3.Keypair();

    let orderbook_1 = new anchor.web3.Keypair();
    const orderbook1Ix = await EmberProgram.account.orderBookState.createInstruction(orderbook_1)
    let orderbook_2 = new anchor.web3.Keypair();
    const orderbook2Ix = await EmberProgram.account.orderBookState.createInstruction(orderbook_2)
    let balances = new anchor.web3.Keypair();
    const balancesIx = await EmberProgram.account.usersBalances.createInstruction(balances)

    const [marketAuthPda, _] = anchor.web3.PublicKey.findProgramAddressSync([MARKET_AUTH_SEED, market.publicKey.toBuffer()], EmberProgram.programId);

    const baseVault1 = new anchor.web3.Keypair();
    const baseVault2 = new anchor.web3.Keypair();
    const quoteVault = new anchor.web3.Keypair();

    await EmberProgram.methods.initializeMarket().accounts({
      signer: EmberProgram.provider.publicKey,
      market: market.publicKey,
      orderbookState1: orderbook_1.publicKey,
      orderbookState2: orderbook_2.publicKey,
      balances: balances.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([orderbook_1, orderbook_2, balances, market])
      .preInstructions([orderbook1Ix, orderbook2Ix, balancesIx])
      .rpc();

    const tx = await EmberProgram.methods.initializeVaults().accounts({
      baseToken1: yesToken,
      baseToken2: noToken,
      quoteToken: collateralToken,
      baseVault1: baseVault1.publicKey,
      baseVault2: baseVault2.publicKey,
      quoteVault: quoteVault.publicKey,
      market: market.publicKey,
      signer: EmberProgram.provider.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      condition,
      marketAuthPda,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([baseVault1, baseVault2, quoteVault])
      .rpc(OPTS);
    console.log(tx);
  });

  it("Merging tickets", async () => {

    await BOTProgram.methods.mergeTicket(new anchor.BN(2)).accounts({
      signer: BOTProgram.provider.publicKey,
      condition,
      conditionAuthPda,
      ticketTokenMint,
      outcome1Token: yesToken,
      outcome2Token: noToken,
      payer1: yesTokenAta,
      payer2: noTokenAta,
      receiver: ticketTokenAta,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  }
  );

  it("Redeeming tickets", async () => {

    await BOTProgram.methods.redeemTicket(new anchor.BN(2)).accounts({
      signer: BOTProgram.provider.publicKey,
      condition,
      conditionAuthPda,
      ticketTokenMint,
      collateralVault,
      payer: ticketTokenAta,
      receiver: collateralTokenAta,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  });

  it("Announce result", async () => {
    await BOTProgram.methods.resolveCondition(new anchor.BN(0)).accounts({
      signer: BOTProgram.provider.publicKey,
      condition,
      conditionAuthPda,
    }).rpc(OPTS);
  });

  it("Claiming payout", async () => {
    await BOTProgram.methods.redeemPayout(new anchor.BN(3)).accounts({
      signer: BOTProgram.provider.publicKey,
      condition,
      conditionAuthPda,
      outcomeToken: yesToken,
      payer: yesTokenAta,
      collateralVault,
      receiver: collateralTokenAta,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  });
});

