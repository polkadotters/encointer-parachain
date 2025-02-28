// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

use cumulus_primitives_core::ParaId;
use parachain_runtime::{BalanceType, CeremonyPhaseType};
use parachains_common::{AccountId, AuraId};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};

pub use crate::chain_spec_helpers::{
	public_from_ss58, rococo_properties, EncointerKeys, GenesisKeys, RelayChain, WellKnownKeys,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type EncointerChainSpec = GenericChainSpec<parachain_runtime::RuntimeGenesisConfig, Extensions>;

/// Specialized `ChainSpec` for the launch parachain runtime.
pub type LaunchChainSpec = GenericChainSpec<launch_runtime::RuntimeGenesisConfig, Extensions>;

pub const ENDOWED_FUNDING: u128 = 1 << 60;

/// Configure `endowed_accounts` with initial balance of `ENDOWED_FUNDING`.
pub fn allocate_endowance(endowed_accounts: Vec<AccountId>) -> Vec<(AccountId, u128)> {
	endowed_accounts.into_iter().map(|k| (k, ENDOWED_FUNDING)).collect()
}

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Chain-spec for the encointer runtime
pub fn encointer_spec(
	id: ParaId,
	genesis_keys: GenesisKeys,
	relay_chain: RelayChain,
) -> EncointerChainSpec {
	let (council, endowed, authorities) = match genesis_keys {
		GenesisKeys::Encointer =>
			(EncointerKeys::council(), [].to_vec(), EncointerKeys::authorities()),
		GenesisKeys::EncointerWithCouncilEndowed =>
			(EncointerKeys::council(), EncointerKeys::council(), EncointerKeys::authorities()),
		GenesisKeys::WellKnown =>
			(WellKnownKeys::council(), WellKnownKeys::endowed(), WellKnownKeys::authorities()),
	};

	chain_spec(
		"Encointer Network",
		move || {
			encointer_genesis(
				council.clone(),
				authorities.clone(),
				allocate_endowance(endowed.clone()),
				id,
			)
		},
		relay_chain.chain_type(),
		id,
		&relay_chain,
	)
}

/// Chain-spec for the launch runtime
pub fn launch_spec(
	id: ParaId,
	genesis_keys: GenesisKeys,
	relay_chain: RelayChain,
) -> LaunchChainSpec {
	let (council, endowed, authorities) = match genesis_keys {
		GenesisKeys::Encointer =>
			(EncointerKeys::council(), [].to_vec(), EncointerKeys::authorities()),
		GenesisKeys::EncointerWithCouncilEndowed =>
			(EncointerKeys::council(), EncointerKeys::council(), EncointerKeys::authorities()),
		GenesisKeys::WellKnown =>
			(WellKnownKeys::council(), WellKnownKeys::endowed(), WellKnownKeys::authorities()),
	};

	chain_spec(
		"Encointer Launch",
		move || {
			launch_genesis(
				council.clone(),
				authorities.clone(),
				allocate_endowance(endowed.clone()),
				id,
			)
		},
		relay_chain.chain_type(),
		id,
		&relay_chain,
	)
}

/// decorates the given `testnet_constructor` with metadata.
///
/// Intended to remove redundant code when defining encointer-launch-runtime and
/// encointer-parachain-runtime chain-specs.
fn chain_spec<F: Fn() -> RuntimeGenesisConfig + 'static + Send + Sync, RuntimeGenesisConfig>(
	chain_name: &str,
	testnet_constructor: F,
	chain_type: ChainType,
	para_id: ParaId,
	relay_chain: &RelayChain,
) -> GenericChainSpec<RuntimeGenesisConfig, Extensions> {
	GenericChainSpec::<RuntimeGenesisConfig, Extensions>::from_genesis(
		chain_name,
		&format!("encointer-{}", relay_chain.to_string()),
		chain_type,
		testnet_constructor,
		Vec::new(),
		// telemetry endpoints
		None,
		// protocol id
		Some(&format!("nctr-{}", relay_chain.to_string().chars().next().unwrap())),
		// properties
		None,
		Some(relay_chain.properties()),
		Extensions { relay_chain: relay_chain.to_string(), para_id: para_id.into() },
	)
}

pub fn sybil_dummy_spec(id: ParaId, relay_chain: RelayChain) -> EncointerChainSpec {
	let (council, endowed, authorities) =
		(WellKnownKeys::council(), WellKnownKeys::endowed(), WellKnownKeys::authorities());

	EncointerChainSpec::from_genesis(
		"Sybil Dummy",
		"sybil-dummy-rococo-v1",
		relay_chain.chain_type(),
		move || {
			encointer_genesis(
				council.clone(),
				authorities.clone(),
				allocate_endowance(endowed.clone()),
				id,
			)
		},
		Vec::new(),
		// telemetry endpoints
		None,
		// protocol id
		None,
		None,
		// properties
		Some(
			serde_json::from_str(
				r#"{
			"ss58Format": 2,
			"tokenDecimals": 12,
			"tokenSymbol": "DUM"
		  }"#,
			)
			.unwrap(),
		),
		Extensions { relay_chain: relay_chain.to_string(), para_id: id.into() },
	)
}

fn encointer_genesis(
	encointer_council: Vec<AccountId>,
	initial_authorities: Vec<AuraId>,
	endowance_allocation: Vec<(AccountId, u128)>,
	id: ParaId,
) -> parachain_runtime::RuntimeGenesisConfig {
	parachain_runtime::RuntimeGenesisConfig {
		system: parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			_config: Default::default(),
		},
		parachain_system: Default::default(),
		balances: parachain_runtime::BalancesConfig { balances: endowance_allocation },
		parachain_info: parachain_runtime::ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
		aura: parachain_runtime::AuraConfig {
			authorities: initial_authorities,
			..Default::default()
		},
		aura_ext: Default::default(),
		polkadot_xcm: parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			_config: Default::default(),
		},
		treasury: Default::default(),
		collective: Default::default(),
		membership: parachain_runtime::MembershipConfig {
			members: encointer_council.try_into().expect("Council below council max members; qed."),
			phantom: Default::default(),
		},
		encointer_scheduler: parachain_runtime::EncointerSchedulerConfig {
			current_phase: CeremonyPhaseType::Registering,
			current_ceremony_index: 1,
			phase_durations: vec![
				(CeremonyPhaseType::Registering, 604800000), // 7d
				(CeremonyPhaseType::Assigning, 86400000),    // 1d
				(CeremonyPhaseType::Attesting, 172800000),   // 2d
			],
			_config: Default::default(),
		},
		encointer_ceremonies: parachain_runtime::EncointerCeremoniesConfig {
			ceremony_reward: BalanceType::from_num(1),
			time_tolerance: 600_000,   // +-10min
			location_tolerance: 1_000, // [m]
			endorsement_tickets_per_bootstrapper: 10,
			endorsement_tickets_per_reputable: 5,
			reputation_lifetime: 5,
			inactivity_timeout: 5, // idle ceremonies before purging community
			meetup_time_offset: 0,
			_config: Default::default(),
		},
		encointer_communities: parachain_runtime::EncointerCommunitiesConfig {
			min_solar_trip_time_s: 1, // [s]
			max_speed_mps: 1,         // [m/s] suggested would be 83m/s for security,
			_config: Default::default(),
		},
		encointer_balances: parachain_runtime::EncointerBalancesConfig {
			// for relative adjustment.
			// 100_000 translates 5uKSM to 0.01 CC if ceremony reward is 20 CC
			// lower values lead to lower fees in CC proportionally
			fee_conversion_factor: 100_000,
			_config: Default::default(),
		},
		encointer_faucet: parachain_runtime::EncointerFaucetConfig {
			reserve_amount: 10_000_000_000_000,
			_config: Default::default(),
		},
	}
}

fn launch_genesis(
	encointer_council: Vec<AccountId>,
	initial_authorities: Vec<AuraId>,
	endowance_allocation: Vec<(AccountId, u128)>,
	id: ParaId,
) -> launch_runtime::RuntimeGenesisConfig {
	launch_runtime::RuntimeGenesisConfig {
		system: launch_runtime::SystemConfig {
			code: launch_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			_config: Default::default(),
		},
		parachain_system: Default::default(),
		balances: launch_runtime::BalancesConfig { balances: endowance_allocation },
		parachain_info: launch_runtime::ParachainInfoConfig {
			parachain_id: id,
			_config: Default::default(),
		},
		aura: launch_runtime::AuraConfig { authorities: initial_authorities },
		aura_ext: Default::default(),
		polkadot_xcm: launch_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			_config: Default::default(),
		},
		treasury: Default::default(),
		collective: Default::default(),
		membership: launch_runtime::MembershipConfig {
			members: encointer_council.try_into().expect("Council below council max members; qed."),
			phantom: Default::default(),
		},
	}
}

/// hard-coded launch-runtime config for rococo
pub fn launch_rococo() -> Result<LaunchChainSpec, String> {
	LaunchChainSpec::from_json_bytes(&include_bytes!("../res/encointer-rococo.json")[..])
}

/// hard-coded launch-runtime config for kusama
pub fn launch_kusama() -> Result<LaunchChainSpec, String> {
	LaunchChainSpec::from_json_bytes(&include_bytes!("../res/encointer-kusama.json")[..])
}

/// hard-coded launch-runtime config for westend
pub fn launch_westend() -> Result<LaunchChainSpec, String> {
	LaunchChainSpec::from_json_bytes(&include_bytes!("../res/encointer-westend.json")[..])
}
