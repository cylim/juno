<script lang="ts">
	import { i18n } from '$lib/stores/i18n.store';
	import SpinnerParagraph from '$lib/components/ui/SpinnerParagraph.svelte';
	import { toasts } from '$lib/stores/toasts.store';
	import type { AnalyticsTrackEvents } from '$declarations/orbiter/orbiter.did';
	import AnalyticsChart from '$lib/components/analytics/AnalyticsChart.svelte';
	import { isNullish, nonNullish } from '@dfinity/utils';
	import { orbiterStore } from '$lib/stores/orbiter.store';
	import { satelliteStore } from '$lib/stores/satellite.store';
	import AnalyticsNew from '$lib/components/analytics/AnalyticsNew.svelte';
	import AnalyticsFilter from '$lib/components/analytics/AnalyticsFilter.svelte';
	import type {
		AnalyticsPageViews as AnalyticsPageViewsType,
		PageViewsParams,
		PageViewsPeriod
	} from '$lib/types/ortbiter';
	import { debounce } from '@dfinity/utils';
	import AnalyticsEvents from '$lib/components/analytics/AnalyticsEvents.svelte';
	import AnalyticsEventsExport from '$lib/components/analytics/AnalyticsEventsExport.svelte';
	import AnalyticsPageViews from '$lib/components/analytics/AnalyticsPageViews.svelte';
	import AnalyticsMetrics from '$lib/components/analytics/AnalyticsMetrics.svelte';
	import NoAnalytics from '$lib/components/analytics/NoAnalytics.svelte';
	import { authStore } from '$lib/stores/auth.store';
	import { versionStore } from '$lib/stores/version.store';
	import { getAnalyticsPageViews, getAnalyticsTrackEvents } from '$lib/services/orbiters.services';

	let loading = true;

	let pageViews: AnalyticsPageViewsType | undefined = undefined;
	let trackEvents: AnalyticsTrackEvents | undefined = undefined;

	let period: PageViewsPeriod = {};

	const loadAnalytics = async () => {
		if (isNullish($orbiterStore)) {
			loading = false;
			return;
		}

		if (isNullish($versionStore.orbiter) || isNullish($versionStore.orbiter?.current)) {
			return;
		}

		try {
			const params: PageViewsParams = {
				satelliteId: $satelliteStore?.satellite_id,
				orbiterId: $orbiterStore.orbiter_id,
				identity: $authStore.identity,
				...period
			};

			const [views, events] = await Promise.all([
				getAnalyticsPageViews({ params, orbiterVersion: $versionStore.orbiter.current }),
				getAnalyticsTrackEvents({ params, orbiterVersion: $versionStore.orbiter.current })
			]);

			pageViews = views;
			trackEvents = events;

			loading = false;
		} catch (err: unknown) {
			toasts.error({
				text: $i18n.errors.analytics_load_error,
				detail: err
			});
		}
	};

	const debouncePageViews = debounce(loadAnalytics);

	$: $orbiterStore, $satelliteStore, $versionStore, period, debouncePageViews();

	const selectPeriod = ({ detail }: CustomEvent<PageViewsPeriod>) => (period = detail);
</script>

{#if loading}
	<SpinnerParagraph>{$i18n.analytics.loading}</SpinnerParagraph>
{:else}
	{#if isNullish($orbiterStore) || isNullish(pageViews)}
		<NoAnalytics />
	{:else}
		<AnalyticsFilter on:junoPeriod={selectPeriod} />

		<AnalyticsChart data={pageViews} />

		<AnalyticsMetrics {pageViews} />

		<AnalyticsPageViews {pageViews} />

		{#if nonNullish(trackEvents) && trackEvents.total.length > 0}
			<hr />

			<AnalyticsEvents {trackEvents} />

			<AnalyticsEventsExport orbiter={$orbiterStore} {period} />
		{/if}
	{/if}

	{#if isNullish($orbiterStore)}
		<AnalyticsNew />
	{/if}
{/if}
