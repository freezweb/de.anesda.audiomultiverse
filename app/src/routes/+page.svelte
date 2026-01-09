<script lang="ts">
	import { onMount } from 'svelte';
	import ChannelStrip from '$lib/components/ChannelStrip.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import RoutingMatrix from '$lib/components/RoutingMatrix.svelte';
	import MeterBridge from '$lib/components/MeterBridge.svelte';
	import ScenesView from '$lib/components/ScenesView.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import ServerDiscovery from '$lib/components/ServerDiscovery.svelte';
	import Fader from '$lib/components/Fader.svelte';
	import { channels, mixerState, initializeChannels } from '$lib/stores/mixer';
	import { connectionState, isConnected } from '$lib/stores/connection';
	import { connect, disconnect, sendChannelFader, sendChannelMute, sendChannelSolo, sendRouting } from '$lib/api/websocket';

	type ViewMode = 'mixer' | 'matrix' | 'meters' | 'scenes' | 'settings';
	let viewMode: ViewMode = 'mixer';
	
	let masterFader = 0.75;
	let showDiscovery = false;

	onMount(() => {
		// Demo-Kanäle initialisieren
		initializeChannels(32, 32);
		
		// Verbindung zum Server herstellen (URL aus connectionState/localStorage)
		connect();

		return () => {
			disconnect();
		};
	});
	
	// Zeige Discovery wenn nicht verbunden
	$: if ($connectionState.status === 'error' || $connectionState.status === 'disconnected') {
		// Nach 2 Sekunden Discovery anzeigen wenn keine Verbindung
		setTimeout(() => {
			if (!$isConnected) {
				showDiscovery = true;
			}
		}, 2000);
	}
	
	function handleServerSelect(e: CustomEvent<{ url: string }>) {
		showDiscovery = false;
		connect(e.detail.url);
	}

	onMount(() => {
		// Demo-Kanäle initialisieren
		initializeChannels(32, 32);
		
		// Verbindung zum Server herstellen (URL aus connectionState/localStorage)
		connect();

		return () => {
			disconnect();
		};
	});
	
	// Input-Kanäle für Mixer-Ansicht
	$: inputChannels = $channels.filter(ch => ch.id < 1000);
	
	function handleFaderChange(channelId: number, value: number) {
		sendChannelFader(channelId, value);
	}
	
	function handleMuteToggle(channelId: number) {
		const channel = $channels.find(ch => ch.id === channelId);
		if (channel) sendChannelMute(channelId, !channel.mute);
	}
	
	function handleSoloToggle(channelId: number) {
		const channel = $channels.find(ch => ch.id === channelId);
		if (channel) sendChannelSolo(channelId, !channel.solo);
	}
	
	function handleRoutingChange(e: CustomEvent<{ input: number; output: number; enabled: boolean }>) {
		sendRouting(e.detail.input, e.detail.output, e.detail.enabled);
	}
</script>

<div class="h-screen flex flex-col bg-mixer-bg">
	<!-- Toolbar -->
	<header class="h-12 bg-mixer-surface border-b border-gray-700 flex items-center px-4 justify-between">
		<div class="flex items-center gap-3">
			<span class="text-xl font-bold">🎛️ AudioMultiverse</span>
			<span class="text-sm text-gray-400">v0.1.0</span>
		</div>

		<div class="flex items-center gap-4">
			<!-- Verbindungsstatus -->
			<button 
				class="flex items-center gap-2 px-3 py-1 rounded hover:bg-gray-700 transition-colors"
				on:click={() => showDiscovery = true}
				title="Server-Suche öffnen"
			>
				<div class="w-2 h-2 rounded-full" 
					class:bg-green-500={$isConnected}
					class:bg-yellow-500={$connectionState.status === 'connecting'}
					class:bg-red-500={$connectionState.status === 'error' || $connectionState.status === 'disconnected'}
				></div>
				<span class="text-sm text-gray-400">
					{$isConnected ? 'Verbunden' : $connectionState.status === 'connecting' ? 'Verbinde...' : 'Getrennt'}
				</span>
				<span class="text-gray-500">🔍</span>
			</button>

			<!-- View-Switcher -->
			<div class="flex gap-1">
				<button 
					class="px-3 py-1 text-sm rounded transition-colors"
					class:bg-blue-600={viewMode === 'mixer'}
					class:bg-gray-700={viewMode !== 'mixer'}
					on:click={() => viewMode = 'mixer'}
				>
					Mixer
				</button>
				<button 
					class="px-3 py-1 text-sm rounded transition-colors"
					class:bg-blue-600={viewMode === 'matrix'}
					class:bg-gray-700={viewMode !== 'matrix'}
					on:click={() => viewMode = 'matrix'}
				>
					Matrix
				</button>
				<button 
					class="px-3 py-1 text-sm rounded transition-colors"
					class:bg-blue-600={viewMode === 'meters'}
					class:bg-gray-700={viewMode !== 'meters'}
					on:click={() => viewMode = 'meters'}
				>
					Meter
				</button>
				<button 
					class="px-3 py-1 text-sm rounded transition-colors"
					class:bg-blue-600={viewMode === 'scenes'}
					class:bg-gray-700={viewMode !== 'scenes'}
					on:click={() => viewMode = 'scenes'}
				>
					Szenen
				</button>
			</div>

			<button 
				class="p-2 hover:bg-gray-700 rounded"
				class:bg-blue-600={viewMode === 'settings'}
				on:click={() => viewMode = 'settings'}
			>⚙️</button>
		</div>
	</header>

	<!-- Main Content -->
	{#if viewMode === 'mixer'}
		<div class="flex-1 flex overflow-hidden">
			<!-- Channel Strips -->
			<div class="flex-1 flex overflow-x-auto p-2 gap-1">
				{#each inputChannels as channel (channel.id)}
					<ChannelStrip 
						{channel}
						on:faderChange={(e) => handleFaderChange(channel.id, e.detail)}
						on:muteToggle={() => handleMuteToggle(channel.id)}
						on:soloToggle={() => handleSoloToggle(channel.id)}
					/>
				{/each}
				
				{#if inputChannels.length === 0}
					<div class="flex-1 flex items-center justify-center text-gray-500">
						<div class="text-center">
							<p class="text-xl mb-2">Verbinde mit Server...</p>
							<p class="text-sm">{$connectionState.serverUrl}</p>
						</div>
					</div>
				{/if}
			</div>

			<!-- Master Section -->
			<div class="w-28 bg-mixer-surface border-l border-gray-700 p-2 flex flex-col">
				<div class="text-center text-sm text-gray-400 mb-2">MASTER</div>
				<div class="flex-1">
					<Fader bind:value={masterFader} />
				</div>
				<button 
					class="mt-2 py-1 text-xs font-bold rounded bg-gray-600 hover:bg-red-600 transition-colors"
				>
					MUTE ALL
				</button>
			</div>
		</div>
	{:else if viewMode === 'matrix'}
		<RoutingMatrix on:routingChange={handleRoutingChange} />
	{:else if viewMode === 'meters'}
		<MeterBridge />
	{:else if viewMode === 'scenes'}
		<ScenesView />
	{:else if viewMode === 'settings'}
		<SettingsView />
	{/if}
</div>

<!-- Server Discovery Modal -->
{#if showDiscovery}
	<ServerDiscovery 
		on:select={handleServerSelect}
		on:close={() => showDiscovery = false}
	/>
{/if}
