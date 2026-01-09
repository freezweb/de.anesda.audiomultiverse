<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import ChannelStrip from './ChannelStrip.svelte';
	import Toolbar from './Toolbar.svelte';
	import { channels, meterData } from '$lib/stores/mixer';
	import { connectionState } from '$lib/stores/connection';
	import { 
		sendChannelFader, 
		sendChannelMute, 
		sendChannelSolo 
	} from '$lib/api/websocket';
	
	const dispatch = createEventDispatcher<{ disconnect: void }>();
	
	let viewMode: 'inputs' | 'outputs' | 'all' = 'inputs';
	
	// Kanäle nach Typ filtern
	$: inputChannels = $channels.filter(ch => ch.id < 1000);
	$: outputChannels = $channels.filter(ch => ch.id >= 1000);
	$: displayedChannels = viewMode === 'inputs' ? inputChannels : 
						   viewMode === 'outputs' ? outputChannels : 
						   $channels;
	
	function handleFaderChange(channelId: number, value: number) {
		sendChannelFader(channelId, value);
	}
	
	function handleMuteToggle(channelId: number) {
		const channel = $channels.find(ch => ch.id === channelId);
		if (channel) {
			sendChannelMute(channelId, !channel.mute);
		}
	}
	
	function handleSoloToggle(channelId: number) {
		const channel = $channels.find(ch => ch.id === channelId);
		if (channel) {
			sendChannelSolo(channelId, !channel.solo);
		}
	}
	
	function handleDisconnect() {
		dispatch('disconnect');
	}
</script>

<div class="h-full flex flex-col bg-mixer-bg">
	<!-- Toolbar -->
	<Toolbar 
		status={$connectionState.status}
		serverUrl={$connectionState.serverUrl}
		on:disconnect={handleDisconnect}
	/>
	
	<!-- View-Auswahl -->
	<div class="flex gap-2 p-2 bg-mixer-surface border-b border-gray-700">
		<button
			class="px-4 py-1 rounded text-sm transition-colors"
			class:bg-mixer-highlight={viewMode === 'inputs'}
			class:bg-gray-700={viewMode !== 'inputs'}
			on:click={() => viewMode = 'inputs'}
		>
			Eingänge ({inputChannels.length})
		</button>
		<button
			class="px-4 py-1 rounded text-sm transition-colors"
			class:bg-mixer-highlight={viewMode === 'outputs'}
			class:bg-gray-700={viewMode !== 'outputs'}
			on:click={() => viewMode = 'outputs'}
		>
			Ausgänge ({outputChannels.length})
		</button>
		<button
			class="px-4 py-1 rounded text-sm transition-colors"
			class:bg-mixer-highlight={viewMode === 'all'}
			class:bg-gray-700={viewMode !== 'all'}
			on:click={() => viewMode = 'all'}
		>
			Alle
		</button>
	</div>
	
	<!-- Channel-Strips -->
	<div class="flex-1 overflow-x-auto overflow-y-hidden">
		<div class="h-full flex gap-1 p-2">
			{#each displayedChannels as channel (channel.id)}
				{@const meter = $meterData.get(channel.id)}
				<ChannelStrip
					id={channel.id}
					name={channel.name}
					fader={channel.fader}
					mute={channel.mute}
					solo={channel.solo}
					pan={channel.pan}
					meterLevel={meter?.level ?? 0}
					meterPeak={meter?.peak ?? 0}
					on:faderChange={(e) => handleFaderChange(channel.id, e.detail)}
					on:muteToggle={() => handleMuteToggle(channel.id)}
					on:soloToggle={() => handleSoloToggle(channel.id)}
				/>
			{/each}
			
			{#if displayedChannels.length === 0}
				<div class="flex-1 flex items-center justify-center text-gray-500">
					Keine Kanäle verfügbar
				</div>
			{/if}
		</div>
	</div>
</div>
