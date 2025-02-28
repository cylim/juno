<script lang="ts">
	import Menu from '$lib/components/ui/Menu.svelte';
	import IconSatellite from '$lib/components/icons/IconSatellite.svelte';
	import IconDatastore from '$lib/components/icons/IconDatastore.svelte';
	import IconStorage from '$lib/components/icons/IconStorage.svelte';
	import IconAuthentication from '$lib/components/icons/IconAuthentication.svelte';
	import IconHosting from '$lib/components/icons/IconHosting.svelte';
	import { satelliteIdStore } from '$lib/stores/satellite.store';
	import { page } from '$app/stores';
	import { i18n } from '$lib/stores/i18n.store';
	import Theme from '$lib/components/ui/Theme.svelte';

	let routeId: string | null;
	$: routeId = $page.route.id;

	let satelliteId: string;
	$: satelliteId = $satelliteIdStore ?? '';

	const isSelected = ({ routeId, path }: { routeId: string | null; path: string }): boolean =>
		routeId?.includes(path) ?? false;
</script>

<Menu>
	<nav>
		<a
			class="link"
			href={`/satellite/?s=${satelliteId}`}
			class:selected={isSelected({ routeId, path: 'satellite' })}
		>
			<IconSatellite size="24px" />
			<span>{$i18n.satellites.satellite}</span>
		</a>

		<a
			class="link"
			href={`/authentication/?s=${satelliteId}`}
			class:selected={isSelected({ routeId, path: 'authentication' })}
		>
			<IconAuthentication size="24px" />
			<span>{$i18n.authentication.title}</span>
		</a>

		<a
			class="link"
			href={`/datastore/?s=${satelliteId}`}
			class:selected={isSelected({ routeId, path: 'datastore' })}
		>
			<IconDatastore size="24px" />
			<span>{$i18n.datastore.title}</span>
		</a>

		<a
			class="link"
			href={`/storage/?s=${satelliteId}`}
			class:selected={isSelected({ routeId, path: 'storage' })}
		>
			<IconStorage size="24px" />
			<span>{$i18n.storage.title}</span>
		</a>

		<a
			class="link"
			href={`/hosting/?s=${satelliteId}`}
			class:selected={isSelected({ routeId, path: 'hosting' })}
		>
			<IconHosting size="24px" />
			<span>{$i18n.hosting.title}</span>
		</a>
	</nav>

	<div class="toggle">
		<Theme />
	</div>
</Menu>

<style lang="scss">
	@use '../../styles/mixins/fonts';
	@use '../../styles/mixins/media';

	.selected {
		background: var(--color-background);
		color: var(--color-background-contrast);

		&:hover:not(:disabled),
		&:active:not(:disabled) {
			background: var(--color-background);
		}
	}

	a.link {
		display: inline-flex;
		align-items: center;

		gap: var(--padding);

		padding: var(--padding-1_5x) var(--padding-6x);

		width: var(--menu-width);
		border-radius: 0;

		transition: background var(--animation-time) ease-out;

		&:hover:not(:disabled),
		&:active:not(:disabled) {
			color: var(--color-primary-contrast);
			font-weight: var(--font-weight-bold);
		}

		span {
			font-size: var(--font-size-ultra-small);
		}

		text-decoration: none;
	}

	@include media.dark-theme {
		a.link {
			&:hover:not(:disabled),
			&:active:not(:disabled) {
				color: var(--color-card-contrast);
			}
		}
	}

	nav {
		flex: 1;
	}

	.toggle {
		padding: 0 var(--padding-6x) var(--padding-4x);
		width: var(--menu-width);
		overflow: hidden;

		@include media.min-width(xlarge) {
			padding: 0 var(--padding-6x) var(--padding-8x);
		}
	}
</style>
