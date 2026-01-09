<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { ConnectionStatus } from '$lib/stores/connection';
	
	export let status: ConnectionStatus = 'disconnected';
	export let serverUrl = '';
	
	const dispatch = createEventDispatcher<{ disconnect: void }>();
	
	$: statusText = {
		disconnected: 'Getrennt',
		connecting: 'Verbinde...',
		connected: 'Verbunden',
		error: 'Fehler'
	}[status];
	
	$: statusColor = {
		disconnected: 'bg-gray-500',
		connecting: 'bg-yellow-500',
		connected: 'bg-green-500',
		error: 'bg-red-500'
	}[status];
</script>

<header class="h-10 bg-mixer-surface border-b border-gray-700 flex items-center px-3 justify-between">
	<!-- Logo & Titel -->
	<div class="flex items-center gap-2">
		<span class="text-lg font-bold">🎛️ AudioMultiverse</span>
		<span class="text-xs text-gray-500">Remote</span>
	</div>

	<!-- Status & Disconnect -->
	<div class="flex items-center gap-3">
		<!-- Verbindungsstatus -->
		<div class="flex items-center gap-2">
			<div class="w-2 h-2 rounded-full {statusColor}"></div>
			<span class="text-xs text-gray-400">
				{statusText}
			</span>
		</div>
		
		{#if status === 'connected'}
			<button 
				class="px-2 py-0.5 text-xs bg-gray-700 hover:bg-red-600 rounded transition-colors"
				on:click={() => dispatch('disconnect')}
			>
				Trennen
			</button>
		{/if}
	</div>
</header>
