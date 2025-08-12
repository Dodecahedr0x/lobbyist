use {
    crate::common::PERMISSIONLESS_ACCOUNT,
    borsh::{BorshDeserialize, BorshSerialize},
    litesvm::LiteSVM,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program::system_program,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::{
        io::{Read, Write},
        marker::PhantomData,
    },
};

pub const SQUADS_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");

pub const SQUADS_PROGRAM_CONFIG: Pubkey =
    Pubkey::from_str_const("BSTq9w3kZwNwpBXJEvTZz2G9ZTNyKBvoSeXMvwb4cNZr");

pub const SQUADS_PROGRAM_CONFIG_TREASURY: Pubkey =
    Pubkey::from_str_const("5DH2e3cJmFpyi6mk65EGFediunm4ui6BiKNUNrhWtD1b");

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProposalCreateArgs {
    /// Index of the multisig transaction this proposal is associated with.
    pub transaction_index: u64,
    /// Whether the proposal should be initialized with status `Draft`.
    pub draft: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultTransactionCreateArgs {
    /// Index of the vault this transaction belongs to.
    pub vault_index: u8,
    /// Number of ephemeral signing PDAs required by the transaction.
    pub ephemeral_signers: u8,
    pub transaction_message: Vec<u8>,
    pub memo: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct TransactionMessage {
    /// The number of signer pubkeys in the account_keys vec.
    pub num_signers: u8,
    /// The number of writable signer pubkeys in the account_keys vec.
    pub num_writable_signers: u8,
    /// The number of writable non-signer pubkeys in the account_keys vec.
    pub num_writable_non_signers: u8,
    /// The list of unique account public keys (including program IDs) that will be used in the provided instructions.
    pub account_keys: SmallVec<u8, Pubkey>,
    /// The list of instructions to execute.
    pub instructions: SmallVec<u8, CompiledInstruction>,
    /// List of address table lookups used to load additional accounts
    /// for this transaction.
    pub address_table_lookups: SmallVec<u8, MessageAddressTableLookup>,
}

// Concise serialization schema for instructions that make up transaction.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    /// Indices into the tx's `account_keys` list indicating which accounts to pass to the instruction.
    pub account_indexes: SmallVec<u8, u8>,
    /// Instruction data.
    pub data: SmallVec<u16, u8>,
}

/// Address table lookups describe an on-chain address lookup table to use
/// for loading more readonly and writable accounts in a single tx.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct MessageAddressTableLookup {
    /// Address lookup table account key
    pub account_key: Pubkey,
    /// List of indexes used to load writable account addresses
    pub writable_indexes: SmallVec<u8, u8>,
    /// List of indexes used to load readonly account addresses
    pub readonly_indexes: SmallVec<u8, u8>,
}

pub fn squads_multisig_pda(creator_key: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"multisig", b"multisig", creator_key.as_ref()],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn squads_vault_pda(multisig_pda: Pubkey, index: u8) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"multisig",
            multisig_pda.to_bytes().as_ref(),
            b"vault",
            &[index],
        ],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn squads_spending_limit_pda(multisig_pda: &Pubkey, create_key: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"multisig",
            multisig_pda.to_bytes().as_ref(),
            b"spending_limit",
            create_key.to_bytes().as_ref(),
        ],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn squads_proposal_pda(multisig_pda: Pubkey, transaction_index: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"multisig",
            multisig_pda.to_bytes().as_ref(),
            b"transaction",
            &transaction_index.to_le_bytes(),
            b"proposal",
        ],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn squads_transaction_pda(multisig_pda: Pubkey, transaction_index: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"multisig",
            multisig_pda.to_bytes().as_ref(),
            b"transaction",
            &transaction_index.to_le_bytes(),
        ],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

/// Concise serialization schema for vectors where the length can be represented
/// by any type `L` (typically unsigned integer like `u8` or `u16`)
/// that implements AnchorDeserialize and can be converted to `u32`.
#[derive(Clone, Debug, Default)]
pub struct SmallVec<L, T>(Vec<T>, PhantomData<L>);

impl<L, T> SmallVec<L, T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<L, T> From<SmallVec<L, T>> for Vec<T> {
    fn from(val: SmallVec<L, T>) -> Self {
        val.0
    }
}

impl<L, T> From<Vec<T>> for SmallVec<L, T> {
    fn from(val: Vec<T>) -> Self {
        Self(val, PhantomData)
    }
}

impl<T: BorshSerialize> BorshSerialize for SmallVec<u8, T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = u8::try_from(self.len()).map_err(|_| std::io::ErrorKind::InvalidInput)?;
        // Write the length of the vector as u8.
        writer.write_all(&len.to_le_bytes())?;

        // Write the vector elements.
        serialize_slice(&self.0, writer)
    }
}

impl<T: BorshSerialize> BorshSerialize for SmallVec<u16, T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = u16::try_from(self.len()).map_err(|_| std::io::ErrorKind::InvalidInput)?;
        // Write the length of the vector as u16.
        writer.write_all(&len.to_le_bytes())?;

        // Write the vector elements.
        serialize_slice(&self.0, writer)
    }
}

impl<L, T> BorshDeserialize for SmallVec<L, T>
where
    L: BorshDeserialize + Into<u32>,
    T: BorshDeserialize,
{
    /// This implementation almost exactly matches standard implementation of
    /// `Vec<T>::deserialize` except that it uses `L` instead of `u32` for the length,
    /// and doesn't include `unsafe` code.
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let len: u32 = L::deserialize_reader(reader)?.into();

        let vec = if len == 0 {
            Vec::new()
        } else if let Some(vec_bytes) = T::vec_from_reader(len, reader)? {
            vec_bytes
        } else {
            let mut result = Vec::with_capacity(hint::cautious::<T>(len));
            for _ in 0..len {
                result.push(T::deserialize_reader(reader)?);
            }
            result
        };

        Ok(SmallVec(vec, PhantomData))
    }
}

// This is copy-pasted from borsh::de::hint;
mod hint {
    #[inline]
    pub fn cautious<T>(hint: u32) -> usize {
        let el_size = core::mem::size_of::<T>() as u32;
        core::cmp::max(core::cmp::min(hint, 4096 / el_size), 1) as usize
    }
}

/// Helper method that is used to serialize a slice of data (without the length marker).
/// Copied from borsh::ser::serialize_slice.
#[inline]
fn serialize_slice<T: BorshSerialize, W: Write>(data: &[T], writer: &mut W) -> std::io::Result<()> {
    if let Some(u8_slice) = T::u8_slice(data) {
        writer.write_all(u8_slice)?;
    } else {
        for item in data {
            item.serialize(writer)?;
        }
    }
    Ok(())
}

pub fn create_squads_proposal(svm: &mut LiteSVM, signer: &Keypair, multisig_pda: Pubkey) -> Pubkey {
    let transaction_pda = squads_transaction_pda(multisig_pda, 1);
    let mut create_vault_transaction_ix_data = vec![48, 250, 78, 168, 208, 226, 218, 211];
    borsh::to_writer(
        &mut create_vault_transaction_ix_data,
        &VaultTransactionCreateArgs {
            vault_index: 0,
            ephemeral_signers: 1,
            transaction_message: borsh::to_vec(&TransactionMessage {
                num_signers: 0,
                num_writable_signers: 0,
                num_writable_non_signers: 0,
                account_keys: vec![signer.pubkey()].into(),
                instructions: vec![].into(),
                address_table_lookups: vec![].into(),
            })
            .unwrap(),
            memo: None,
        },
    )
    .unwrap();
    let create_vault_transaction_ix = Instruction::new_with_bytes(
        SQUADS_PROGRAM_ID,
        &create_vault_transaction_ix_data,
        vec![
            AccountMeta::new(multisig_pda, false),
            AccountMeta::new(transaction_pda, false),
            AccountMeta::new_readonly(PERMISSIONLESS_ACCOUNT.pubkey(), true),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );

    let squads_proposal_pda = squads_proposal_pda(multisig_pda, 1);
    let mut create_squads_proposal_ix_data = vec![220, 60, 73, 224, 30, 108, 79, 159];
    borsh::to_writer(
        &mut create_squads_proposal_ix_data,
        &ProposalCreateArgs {
            transaction_index: 1,
            draft: false,
        },
    )
    .unwrap();
    let create_squads_proposal_ix = Instruction::new_with_bytes(
        SQUADS_PROGRAM_ID,
        &create_squads_proposal_ix_data,
        vec![
            AccountMeta::new_readonly(multisig_pda, false),
            AccountMeta::new(squads_proposal_pda, false),
            AccountMeta::new_readonly(PERMISSIONLESS_ACCOUNT.pubkey(), true),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_vault_transaction_ix, create_squads_proposal_ix],
        Some(&signer.pubkey()),
        &[&signer, &PERMISSIONLESS_ACCOUNT.insecure_clone()],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    squads_proposal_pda
}
