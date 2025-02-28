import type {
	Controller,
	Orbiter,
	Satellite
} from '$declarations/mission_control/mission_control.did';
import type { SetControllerParams } from '$lib/types/controllers';
import type { OptionIdentity } from '$lib/types/itentity';
import type { Metadata } from '$lib/types/metadata';
import { getMissionControlActor } from '$lib/utils/actor.juno.utils';
import { toSetController } from '$lib/utils/controllers.utils';
import { Principal } from '@dfinity/principal';
import { toNullable } from '@dfinity/utils';

export const setSatellitesController = async ({
	identity,
	missionControlId,
	satelliteIds,
	controllerId,
	...rest
}: {
	missionControlId: Principal;
	satelliteIds: Principal[];
	identity: OptionIdentity;
} & SetControllerParams) => {
	try {
		const actor = await getMissionControlActor({ missionControlId, identity });
		await actor.set_satellites_controllers(
			satelliteIds,
			[Principal.fromText(controllerId)],
			toSetController(rest)
		);
	} catch (err: unknown) {
		console.error(
			'setSatellitesController:',
			missionControlId.toText(),
			satelliteIds.map((id) => id.toText()).join(',')
		);
		throw err;
	}
};

export const deleteSatellitesController = async ({
	missionControlId,
	satelliteIds,
	controller,
	identity
}: {
	missionControlId: Principal;
	satelliteIds: Principal[];
	controller: Principal;
	identity: OptionIdentity;
}) => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	await actor.del_satellites_controllers(satelliteIds, [controller]);
};

export const setMissionControlController = async ({
	identity,
	missionControlId,
	controllerId,
	...rest
}: {
	missionControlId: Principal;
	identity: OptionIdentity;
} & SetControllerParams) => {
	try {
		const actor = await getMissionControlActor({ missionControlId, identity });
		await actor.set_mission_control_controllers(
			[Principal.fromText(controllerId)],
			toSetController(rest)
		);
	} catch (err: unknown) {
		console.error('setMissionControlController:', missionControlId.toText());
		throw err;
	}
};

export const deleteMissionControlController = async ({
	missionControlId,
	controller,
	identity
}: {
	missionControlId: Principal;
	controller: Principal;
	identity: OptionIdentity;
}) => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	await actor.del_mission_control_controllers([controller]);
};

/**
 * @deprecated use setSatellitesController
 */
export const addSatellitesController = async ({
	missionControlId,
	satelliteIds,
	controllerId,
	identity
}: {
	missionControlId: Principal;
	satelliteIds: Principal[];
	identity: OptionIdentity;
} & SetControllerParams) => {
	try {
		const actor = await getMissionControlActor({ missionControlId, identity });
		await actor.add_satellites_controllers(satelliteIds, [Principal.fromText(controllerId)]);
	} catch (err: unknown) {
		console.error(
			'addSatellitesController:',
			missionControlId.toText(),
			satelliteIds.map((id) => id.toText()).join(',')
		);
		throw err;
	}
};

/**
 * @deprecated use deleteSatellitesController
 */
export const removeSatellitesController = async ({
	missionControlId,
	satelliteIds,
	controller,
	identity
}: {
	missionControlId: Principal;
	satelliteIds: Principal[];
	controller: Principal;
	identity: OptionIdentity;
}) => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	await actor.remove_satellites_controllers(satelliteIds, [controller]);
};

/**
 * @deprecated use setMissionControlController
 */
export const addMissionControlController = async ({
	missionControlId,
	controllerId,
	identity
}: {
	missionControlId: Principal;
	identity: OptionIdentity;
} & SetControllerParams) => {
	try {
		const actor = await getMissionControlActor({ missionControlId, identity });
		await actor.add_mission_control_controllers([Principal.fromText(controllerId)]);
	} catch (err: unknown) {
		console.error('addMissionControlController:', missionControlId.toText());
		throw err;
	}
};

/**
 * @deprecated use deleteMissionControlController
 */
export const removeMissionControlController = async ({
	missionControlId,
	controller,
	identity
}: {
	missionControlId: Principal;
	controller: Principal;
	identity: OptionIdentity;
}) => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	await actor.remove_mission_control_controllers([controller]);
};

export const listMissionControlControllers = async ({
	missionControlId,
	identity
}: {
	missionControlId: Principal;
	identity: OptionIdentity;
}): Promise<[Principal, Controller][]> => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	return actor.list_mission_control_controllers();
};

export const topUp = async ({
	missionControlId,
	canisterId,
	e8s,
	identity
}: {
	missionControlId: Principal;
	canisterId: Principal;
	e8s: bigint;
	identity: OptionIdentity;
}) => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	await actor.top_up(canisterId, { e8s });
};

export const missionControlVersion = async ({
	missionControlId,
	identity
}: {
	missionControlId: Principal;
	identity: OptionIdentity;
}): Promise<string> => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	return actor.version();
};

export const setSatelliteMetadata = async ({
	missionControlId,
	satelliteId,
	metadata,
	identity
}: {
	missionControlId: Principal;
	satelliteId: Principal;
	metadata: Metadata;
	identity: OptionIdentity;
}): Promise<Satellite> => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	return actor.set_satellite_metadata(satelliteId, metadata);
};

export const setOrbitersController = async ({
	missionControlId,
	orbiterIds,
	controllerId,
	identity,
	...rest
}: {
	missionControlId: Principal;
	orbiterIds: Principal[];
	identity: OptionIdentity;
} & SetControllerParams) => {
	try {
		const actor = await getMissionControlActor({ missionControlId, identity });
		await actor.set_orbiters_controllers(
			orbiterIds,
			[Principal.fromText(controllerId)],
			toSetController(rest)
		);
	} catch (err: unknown) {
		console.error(
			'setOrbitersController:',
			missionControlId.toText(),
			orbiterIds.map((id) => id.toText()).join(',')
		);
		throw err;
	}
};

export const deleteOrbitersController = async ({
	missionControlId,
	orbiterIds,
	controller,
	identity
}: {
	missionControlId: Principal;
	orbiterIds: Principal[];
	controller: Principal;
	identity: OptionIdentity;
}) => {
	const actor = await getMissionControlActor({ missionControlId, identity });
	await actor.del_orbiters_controllers(orbiterIds, [controller]);
};

export const deleteSatellite = async ({
	missionControlId,
	satelliteId,
	cyclesToDeposit,
	identity
}: {
	missionControlId: Principal;
	satelliteId: Principal;
	cyclesToDeposit: bigint;
	identity: OptionIdentity;
}) => {
	const { del_satellite } = await getMissionControlActor({ missionControlId, identity });
	await del_satellite(satelliteId, cyclesToDeposit);
};

export const deleteOrbiter = async ({
	missionControlId,
	orbiterId,
	cyclesToDeposit,
	identity
}: {
	missionControlId: Principal;
	orbiterId: Principal;
	cyclesToDeposit: bigint;
	identity: OptionIdentity;
}) => {
	const { del_orbiter } = await getMissionControlActor({ missionControlId, identity });
	await del_orbiter(orbiterId, cyclesToDeposit);
};

export const depositCycles = async ({
	missionControlId,
	cycles,
	destinationId: destination_id,
	identity
}: {
	missionControlId: Principal;
	cycles: bigint;
	destinationId: Principal;
	identity: OptionIdentity;
}) => {
	const { deposit_cycles } = await getMissionControlActor({ missionControlId, identity });
	return deposit_cycles({
		cycles,
		destination_id
	});
};

export const setOrbiter = async ({
	missionControlId,
	orbiterId,
	orbiterName,
	identity
}: {
	missionControlId: Principal;
	orbiterId: Principal;
	orbiterName?: string;
	identity: OptionIdentity;
}): Promise<Orbiter> => {
	const { set_orbiter } = await getMissionControlActor({ missionControlId, identity });
	return set_orbiter(orbiterId, toNullable(orbiterName));
};
