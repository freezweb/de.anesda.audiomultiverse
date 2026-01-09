<script lang="ts">
	import { createEventDispatcher, onMount, onDestroy } from 'svelte';
	
	const dispatch = createEventDispatcher<{ select: { url: string } }>();
	
	interface DiscoveredServer {
		name: string;
		host: string;
		port: number;
		version: string;
		lastSeen: number;
	}
	
	let servers: DiscoveredServer[] = [];
	let manualHost = '';
	let manualPort = '3000';
	let isScanning = false;
	let scanInterval: ReturnType<typeof setInterval> | null = null;
	
	// Demo-Server für Entwicklung
	const demoServers: DiscoveredServer[] = [
		{
			name: 'AudioMultiverse Local',
			host: 'localhost',
			port: 3000,
			version: '0.1.0',
			lastSeen: Date.now()
		}
	];
	
	onMount(() => {
		// Starte Server-Discovery
		startDiscovery();
	});
	
	onDestroy(() => {
		if (scanInterval) {
			clearInterval(scanInterval);
		}
	});
	
	async function startDiscovery() {
		isScanning = true;
		
		// TODO: Echte mDNS/UDP Discovery implementieren
		// Für jetzt: Demo-Server anzeigen
		servers = [...demoServers];
		
		// Simuliere Scan
		setTimeout(() => {
			isScanning = false;
		}, 2000);
	}
	
	function selectServer(server: DiscoveredServer) {
		const url = `ws://${server.host}:${server.port}/ws`;
		dispatch('select', { url });
	}
	
	function connectManual() {
		if (manualHost) {
			const url = `ws://${manualHost}:${manualPort}/ws`;
			dispatch('select', { url });
		}
	}
	
	function refresh() {
		servers = [];
		startDiscovery();
	}
</script>

<div class="h-full flex flex-col items-center justify-center p-8 bg-mixer-bg">
	<div class="max-w-md w-full">
		<!-- Header -->
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold mb-2">🎛️ AudioMultiverse</h1>
			<p class="text-gray-400">Remote Control</p>
		</div>
		
		<!-- Gefundene Server -->
		<div class="bg-mixer-surface rounded-lg p-4 mb-6">
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-lg font-semibold">Verfügbare Server</h2>
				<button 
					class="text-gray-400 hover:text-white transition-colors"
					on:click={refresh}
					disabled={isScanning}
				>
					{#if isScanning}
						<span class="animate-spin inline-block">⟳</span>
					{:else}
						⟳
					{/if}
				</button>
			</div>
			
			{#if servers.length === 0}
				<p class="text-gray-500 text-center py-4">
					{#if isScanning}
						Suche nach Servern...
					{:else}
						Keine Server gefunden
					{/if}
				</p>
			{:else}
				<div class="space-y-2">
					{#each servers as server}
						<button
							class="w-full p-3 bg-mixer-accent rounded-lg hover:bg-mixer-highlight transition-colors text-left"
							on:click={() => selectServer(server)}
						>
							<div class="font-medium">{server.name}</div>
							<div class="text-sm text-gray-400">
								{server.host}:{server.port} • v{server.version}
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</div>
		
		<!-- Manuelle Verbindung -->
		<div class="bg-mixer-surface rounded-lg p-4">
			<h2 class="text-lg font-semibold mb-4">Manuelle Verbindung</h2>
			
			<div class="flex gap-2 mb-3">
				<input
					type="text"
					bind:value={manualHost}
					placeholder="IP-Adresse oder Hostname"
					class="flex-1 bg-gray-800 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-mixer-highlight"
				/>
				<input
					type="number"
					bind:value={manualPort}
					placeholder="Port"
					class="w-20 bg-gray-800 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-mixer-highlight"
				/>
			</div>
			
			<button
				class="w-full py-2 bg-mixer-highlight rounded font-medium hover:bg-opacity-80 transition-colors disabled:opacity-50"
				on:click={connectManual}
				disabled={!manualHost}
			>
				Verbinden
			</button>
		</div>
	</div>
</div>
