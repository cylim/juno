<script lang="ts">
	import { i18n } from '$lib/stores/i18n.store';
	import IconSort from '$lib/components/icons/IconSort.svelte';
	import Popover from '$lib/components/ui/Popover.svelte';
	import type { ListOrderField } from '$lib/types/list';
	import { listParamsStore } from '$lib/stores/data.store';

	let desc = $listParamsStore.order.desc;
	let field: ListOrderField = $listParamsStore.order.field;

	let button: HTMLButtonElement | undefined;
	let visible: boolean | undefined;

	const apply = () => {
		listParamsStore.setOrder({
			desc,
			field
		});

		visible = false;
	};

	$: visible,
		(() => {
			if (visible) {
				return;
			}

			// Avoid glitch
			setTimeout(() => {
				desc = $listParamsStore.order.desc;
				field = $listParamsStore.order.field;
			}, 250);
		})();
</script>

<button
	class="icon"
	aria-label={$i18n.sort.title}
	type="button"
	on:click={() => (visible = true)}
	bind:this={button}><IconSort size="20px" /></button
>

<Popover bind:visible anchor={button} direction="rtl">
	<div class="container">
		<p class="category">{$i18n.sort.sort_by_field}</p>

		<label>
			<input type="radio" bind:group={field} name="field" value="keys" />
			<span>{$i18n.sort.keys}</span>
		</label>

		<label>
			<input type="radio" bind:group={field} name="field" value="created_at" />
			<span>{$i18n.sort.created_at}</span>
		</label>

		<label>
			<input type="radio" bind:group={field} name="field" value="updated_at" />
			<span>{$i18n.sort.updated_at}</span>
		</label>

		<p class="category sort">{$i18n.sort.sort_results}</p>

		<label>
			<input type="radio" bind:group={desc} name="desc" value={false} />
			<span>{$i18n.sort.ascending}</span>
		</label>

		<label>
			<input type="radio" bind:group={desc} name="desc" value={true} />
			<span>{$i18n.sort.descending}</span>
		</label>

		<button class="apply" type="button" on:click|stopPropagation={apply}>
			{$i18n.core.apply}
		</button>
	</div>
</Popover>

<style lang="scss">
	button.icon {
		padding: 0;
	}

	.container {
		padding: var(--padding);
	}

	label {
		display: flex;
		gap: var(--padding-2x);
		align-items: center;

		cursor: pointer;
	}

	.category {
		font-weight: var(--font-weight-bold);
		margin: 0 0 var(--padding);

		&.sort {
			margin-top: var(--padding-1_5x);
		}
	}

	label span {
		color: var(--value-color);
	}

	.apply {
		margin: var(--padding-1_5x) 0 var(--padding);
	}
</style>
