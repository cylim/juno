<script lang="ts">
	import type { Satellite } from '$declarations/mission_control/mission_control.did';
	import LoaderWarnings from '$lib/components/warning/LoaderWarnings.svelte';
	import VersionWarning from '$lib/components/warning/VersionWarnings.svelte';
	import CanisterWarnings from '$lib/components/canister/CanisterWarnings.svelte';
	import { nonNullish } from '@dfinity/utils';
	import { missionControlStore } from '$lib/stores/mission-control.store';
	import { i18n } from '$lib/stores/i18n.store';
	import { orbiterStore } from '$lib/stores/orbiter.store';

	export let satellite: Satellite | undefined = undefined;
</script>

<VersionWarning {satellite} />

{#if nonNullish($missionControlStore)}
	<LoaderWarnings canisterId={$missionControlStore}>
		<svelte:fragment slot="cycles"
			>{$i18n.canisters.warning_mission_control_low_cycles}</svelte:fragment
		>
	</LoaderWarnings>
{/if}

{#if nonNullish(satellite)}
	<CanisterWarnings canisterId={satellite.satellite_id}>
		<svelte:fragment slot="cycles">{$i18n.canisters.warning_satellite_low_cycles}</svelte:fragment>
		<svelte:fragment slot="heap">{$i18n.canisters.warning_satellite_heap_memory}</svelte:fragment>
	</CanisterWarnings>
{/if}

{#if nonNullish($orbiterStore)}
	<LoaderWarnings canisterId={$orbiterStore.orbiter_id}>
		<svelte:fragment slot="cycles">{$i18n.canisters.warning_orbiter_low_cycles}</svelte:fragment>
		<svelte:fragment slot="heap">{$i18n.canisters.warning_orbiter_heap_memory}</svelte:fragment>
	</LoaderWarnings>
{/if}
