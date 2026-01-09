<script lang="ts">
	import { onMount } from 'svelte';
	import ServerDiscovery from '$lib/components/ServerDiscovery.svelte';
	import MixerView from '$lib/components/MixerView.svelte';
	import { connectionState, isConnected } from '$lib/stores/connection';
	import { connect, disconnect } from '$lib/api/websocket';
	
	let showDiscovery = true;
	
	function handleServerSelect(event: CustomEvent<{ url: string }>) {
		connect(event.detail.url);
		showDiscovery = false;
	}
	
	function handleDisconnect() {
		disconnect();
		showDiscovery = true;
	}
	
	// Bei Verbindungsverlust zurück zur Discovery
	$: if ($connectionState.status === 'error' && $connectionState.reconnectAttempts >= 5) {
		showDiscovery = true;
	}
</script>

<svelte:head>
	<title>AudioMultiverse Remote</title>
</svelte:head>

<div class="h-screen w-screen overflow-hidden">
	{#if showDiscovery || !$isConnected}
		<ServerDiscovery on:select={handleServerSelect} />
	{:else}
		<MixerView on:disconnect={handleDisconnect} />
	{/if}
</div>
