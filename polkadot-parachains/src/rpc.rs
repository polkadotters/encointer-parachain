// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Parachain-specific RPCs implementation.

#![warn(missing_docs)]

use std::sync::Arc;

use parachain_runtime::{AssetBalance, AssetId, Moment};
use parachains_common::{AccountId, Balance, Block, BlockNumber, Nonce};
use sc_client_api::AuxStore;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Full client dependencies
pub struct FullDeps<C, P, Backend> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// backend instance used to access offchain storage (added by encointer).
	pub backend: Arc<Backend>,
	/// whether offchain-indexing is enabled (added by encointer).
	pub offchain_indexing_enabled: bool,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, TBackend>(
	deps: FullDeps<C, P, TBackend>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + Sync + Send + 'static,
	C::Api: pallet_encointer_ceremonies_rpc_runtime_api::CeremoniesApi<Block, AccountId, Moment>,
	C::Api:
		pallet_encointer_communities_rpc_runtime_api::CommunitiesApi<Block, AccountId, BlockNumber>,
	C::Api: pallet_encointer_bazaar_rpc_runtime_api::BazaarApi<Block, AccountId>,
	C::Api: encointer_balances_tx_payment_rpc_runtime_api::BalancesTxPaymentApi<
		Block,
		Balance,
		AssetId,
		AssetBalance,
	>,
	TBackend: sc_client_api::Backend<Block>, // added by encointer
	<TBackend as sc_client_api::Backend<Block>>::OffchainStorage: 'static, // added by encointer
{
	use frame_rpc_system::{System, SystemApiServer};
	use pallet_encointer_bazaar_rpc::{BazaarApiServer, BazaarRpc};
	use pallet_encointer_ceremonies_rpc::{CeremoniesApiServer, CeremoniesRpc};
	use pallet_encointer_communities_rpc::{CommunitiesApiServer, CommunitiesRpc};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};

	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, backend, offchain_indexing_enabled, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(BazaarRpc::new(client.clone()).into_rpc())?;

	match backend.offchain_storage() {
		Some(storage) => {
			module.merge(
				CommunitiesRpc::new(client.clone(), storage.clone(), offchain_indexing_enabled)
					.into_rpc(),
			)?;

			module
				.merge(CeremoniesRpc::new(client, storage, offchain_indexing_enabled).into_rpc())?;
		},
		None => log::warn!(
			"Offchain caching disabled, due to lack of offchain storage support in backend. \n
			Will not initialize custom RPCs for 'CommunitiesApi' and 'CeremoniesApi'"
		),
	};

	Ok(module)
}

/// Instantiate reduced RPC extensions for launch runtime
pub fn create_launch_ext<C, P, TBackend>(
	deps: FullDeps<C, P, TBackend>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + Sync + Send + 'static,
{
	use frame_rpc_system::{System, SystemApiServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};

	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, backend: _, offchain_indexing_enabled: _, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client).into_rpc())?;

	Ok(module)
}
