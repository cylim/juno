<script lang="ts">
	import type { Canister } from '$lib/types/canister';
	import { nonNullish } from '@dfinity/utils';
	import { fade } from 'svelte/transition';

	export let canister: Canister | undefined = undefined;

	let enabled = false;
	$: enabled =
		nonNullish(canister) &&
		nonNullish(canister.data) &&
		nonNullish(canister.data.canister) &&
		canister.data.canister.status === 'running';
</script>

{#if enabled}
	<div in:fade>
		<button class="menu" on:click><slot /></button>
	</div>
{/if}
