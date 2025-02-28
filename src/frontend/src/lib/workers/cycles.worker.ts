import type { MemorySize } from '$declarations/satellite/satellite.did';
import { icpXdrConversionRate } from '$lib/api/cmc.api';
import { canisterStatus } from '$lib/api/ic.api';
import { memorySize as memorySizeOrbiter } from '$lib/api/orbiter.worker.api';
import { memorySize as memorySizeSatellite } from '$lib/api/satellites.worker.api';
import {
	CYCLES_WARNING,
	MEMORY_HEAP_WARNING,
	SYNC_CYCLES_TIMER_INTERVAL
} from '$lib/constants/constants';
import type { Canister, CanisterInfo, CanisterSegment } from '$lib/types/canister';
import type { PostMessage, PostMessageDataRequest } from '$lib/types/post-message';
import { cyclesToICP } from '$lib/utils/cycles.utils';
import { loadIdentity } from '$lib/utils/worker.utils';
import type { Identity } from '@dfinity/agent';
import { isNullish, nonNullish } from '@dfinity/utils';
import { createStore, getMany, set } from 'idb-keyval';

onmessage = async ({ data: dataMsg }: MessageEvent<PostMessage<PostMessageDataRequest>>) => {
	const { msg, data } = dataMsg;

	switch (msg) {
		case 'stopCyclesTimer':
			stopCyclesTimer();
			return;
		case 'startCyclesTimer':
			await startCyclesTimer({ data });
			return;
		case 'restartCyclesTimer':
			stopCyclesTimer();
			await startCyclesTimer({ data });
			return;
	}
};

let timer: NodeJS.Timeout | undefined = undefined;

const startCyclesTimer = async ({ data: { segments } }: { data: PostMessageDataRequest }) => {
	const identity = await loadIdentity();

	if (isNullish(identity)) {
		// We do nothing if no identity
		return;
	}

	const sync = async () => await syncCanisters({ identity, segments: segments ?? [] });

	// We sync the cycles now but also schedule the update afterwards
	await sync();

	timer = setInterval(sync, SYNC_CYCLES_TIMER_INTERVAL);
};

const stopCyclesTimer = () => {
	if (!timer) {
		return;
	}

	clearInterval(timer);
	timer = undefined;
};

const customStore = createStore('juno-db', 'juno-cycles-store');

let syncing = false;

const syncCanisters = async ({
	identity,
	segments
}: {
	identity: Identity;
	segments: CanisterSegment[];
}) => {
	if (segments.length === 0) {
		// No canister to sync
		return;
	}

	// We avoid to relaunch a sync while previous sync is not finished
	if (syncing) {
		return;
	}

	await emitSavedCanisters({ canisterIds: segments.map(({ canisterId }) => canisterId) });

	try {
		const trillionRatio: bigint = await icpXdrConversionRate();

		await syncNnsCanisters({ identity, segments, trillionRatio });
	} finally {
		syncing = false;
	}
};

const syncNnsCanisters = async ({
	identity,
	segments,
	trillionRatio
}: {
	identity: Identity;
	segments: CanisterSegment[];
	trillionRatio: bigint;
}) => {
	await Promise.allSettled(
		segments.map(async ({ canisterId, segment }) => {
			try {
				const [canisterResult, memorySizeResult] = await Promise.allSettled([
					canisterStatus({ canisterId, identity }),
					...(segment === 'satellite'
						? [memorySizeSatellite({ satelliteId: canisterId, identity })]
						: segment === 'orbiter'
							? [memorySizeOrbiter({ orbiterId: canisterId, identity })]
							: [])
				]);

				if (canisterResult.status === 'rejected') {
					throw canisterResult.reason;
				}

				const { value: canisterInfo } = canisterResult;

				// We silence those error because we managed the canister status.
				// Satellites and orbiters which were not migrated for example will throw an error because the end point won't be exposed.
				if (memorySizeResult?.status === 'rejected') {
					console.error(`Error fetching memory size information: `, memorySizeResult.reason);
				}

				await syncCanister({
					canisterInfo,
					trillionRatio,
					canisterId: canisterInfo.canisterId,
					memory: memorySizeResult?.status === 'fulfilled' ? memorySizeResult.value : undefined
				});
			} catch (err: unknown) {
				console.error(err);

				emitCanister({
					id: canisterId,
					sync: 'error'
				});

				throw err;
			}
		})
	);
};

// Update ui with one canister information
const emitCanister = (canister: Canister) =>
	postMessage({
		msg: 'syncCanister',
		data: {
			canister
		}
	});

const syncCanister = async ({
	canisterId,
	trillionRatio,
	canisterInfo: { cycles, status, memory_size, idle_cycles_burned_per_day },
	memory
}: {
	canisterId: string;
	trillionRatio: bigint;
	canisterInfo: CanisterInfo;
	memory: MemorySize | undefined;
}) => {
	const canister: Canister = {
		id: canisterId,
		sync: 'synced',
		data: {
			icp: cyclesToICP({ cycles, trillionRatio }),
			warning: {
				cycles: cycles < CYCLES_WARNING,
				heap: (memory?.heap ?? 0n) >= MEMORY_HEAP_WARNING
			},
			canister: {
				status,
				memory_size,
				idle_cycles_burned_per_day,
				cycles
			},
			...(nonNullish(memory) && { memory })
		}
	};

	// Save information in indexed-db as well to load previous values on navigation and refresh
	await set(canisterId, canister, customStore);

	emitCanister(canister);
};

const emitSavedCanisters = async ({ canisterIds }: { canisterIds: string[] }) => {
	const canisters = (await getMany(canisterIds, customStore))
		.filter((canister: Canister | undefined) => canister !== undefined)
		.map((canister) => ({
			...canister,
			sync: 'syncing'
		}));

	const canistersNeverSynced: Canister[] = canisterIds
		.filter((id) => canisters.find(({ syncedId }) => id === syncedId) === undefined)
		.map((id) => ({
			id,
			sync: 'loading'
		}));

	await Promise.all(canistersNeverSynced.map(emitCanister));

	await Promise.all(canisters.map(emitCanister));
};
