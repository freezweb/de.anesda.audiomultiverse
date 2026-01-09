<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	
	const dispatch = createEventDispatcher<{ 
		select: { url: string };
		close: void;
	}>();
	
	interface DiscoveredServer {
		name: string;
		host: string;
		port: number;
		version: string;
		addresses: string[];
	}
	
	let servers: DiscoveredServer[] = [];
	let manualHost = '';
	let manualPort = '8080';
	let isScanning = false;
	let errorMessage = '';

	onMount(() => {
		startDiscovery();
	});
	
	async function startDiscovery() {
		isScanning = true;
		errorMessage = '';
		servers = [];
		
		try {
			// Tauri-Command aufrufen
			const discovered = await invoke<DiscoveredServer[]>('discover_servers');
			servers = discovered;
			
			if (servers.length === 0) {
				errorMessage = 'Keine Server gefunden. Prüfe ob der Server läuft.';
			}
		} catch (error) {
			console.error('Discovery Fehler:', error);
			errorMessage = `Suche fehlgeschlagen: ${error}`;
		} finally {
			isScanning = false;
		}
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
	
	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			connectManual();
		}
	}
</script>

<div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
	<div class="bg-mixer-surface rounded-xl shadow-2xl max-w-lg w-full overflow-hidden">
		<!-- Header -->
		<div class="bg-gradient-to-r from-blue-600 to-purple-600 p-6 text-white">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-2xl font-bold flex items-center gap-2">
						🔍 Server-Suche
					</h1>
					<p class="text-blue-100 mt-1">AudioMultiverse Server im Netzwerk finden</p>
				</div>
				<button 
					class="text-white/70 hover:text-white text-2xl transition-colors"
					on:click={() => dispatch('close')}
				>
					✕
				</button>
			</div>
		</div>
		
		<div class="p-6 space-y-6">
			<!-- Gefundene Server -->
			<div>
				<div class="flex items-center justify-between mb-3">
					<h2 class="text-lg font-semibold text-white">Verfügbare Server</h2>
					<button 
						class="px-3 py-1 bg-mixer-accent/20 text-mixer-accent rounded-lg 
						       hover:bg-mixer-accent/30 transition-colors flex items-center gap-2"
						on:click={startDiscovery}
						disabled={isScanning}
					>
						{#if isScanning}
							<span class="animate-spin">⟳</span>
							<span>Suche...</span>
						{:else}
							<span>⟳</span>
							<span>Aktualisieren</span>
						{/if}
					</button>
				</div>
				
				<div class="bg-mixer-bg rounded-lg overflow-hidden">
					{#if isScanning}
						<div class="p-8 text-center">
							<div class="animate-pulse text-4xl mb-3">📡</div>
							<p class="text-gray-400">Suche nach Servern im Netzwerk...</p>
						</div>
					{:else if servers.length === 0}
						<div class="p-8 text-center">
							<div class="text-4xl mb-3">🔇</div>
							<p class="text-gray-400">{errorMessage || 'Keine Server gefunden'}</p>
							<p class="text-gray-500 text-sm mt-2">
								Stelle sicher, dass der AudioMultiverse Server läuft
							</p>
						</div>
					{:else}
						<div class="divide-y divide-gray-700">
							{#each servers as server}
								<button
									class="w-full p-4 text-left hover:bg-white/5 transition-colors
									       flex items-center justify-between group"
									on:click={() => selectServer(server)}
								>
									<div class="flex items-center gap-4">
										<div class="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center">
											<span class="text-2xl">🎛️</span>
										</div>
										<div>
											<div class="font-medium text-white group-hover:text-mixer-accent transition-colors">
												{server.name}
											</div>
											<div class="text-sm text-gray-400">
												{server.host}:{server.port} • v{server.version}
											</div>
										</div>
									</div>
									<div class="text-gray-400 group-hover:text-mixer-accent transition-colors">
										→
									</div>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			</div>
			
			<!-- Manuelle Verbindung -->
			<div>
				<h2 class="text-lg font-semibold text-white mb-3">Manuelle Verbindung</h2>
				<div class="flex gap-2">
					<input
						type="text"
						bind:value={manualHost}
						on:keydown={handleKeydown}
						placeholder="IP-Adresse oder Hostname"
						class="flex-1 px-4 py-3 bg-mixer-bg border border-gray-600 rounded-lg
						       focus:border-mixer-accent focus:outline-none text-white"
					/>
					<input
						type="number"
						bind:value={manualPort}
						on:keydown={handleKeydown}
						placeholder="Port"
						class="w-24 px-4 py-3 bg-mixer-bg border border-gray-600 rounded-lg
						       focus:border-mixer-accent focus:outline-none text-white text-center"
					/>
					<button
						class="px-6 py-3 bg-mixer-accent text-white rounded-lg font-medium
						       hover:bg-blue-500 transition-colors disabled:opacity-50"
						on:click={connectManual}
						disabled={!manualHost}
					>
						Verbinden
					</button>
				</div>
			</div>
		</div>
	</div>
</div>
