<script lang="ts">
	import { connectionState, isConnected } from '$stores/connection';

	$: connected = $isConnected;
</script>

<header class="h-12 bg-mixer-surface border-b border-gray-700 flex items-center px-4 justify-between">
	<!-- Logo & Titel -->
	<div class="flex items-center gap-3">
		<span class="text-xl font-bold">🎛️ AudioMultiverse</span>
		<span class="text-sm text-gray-400">v0.1.0</span>
	</div>

	<!-- Status -->
	<div class="flex items-center gap-4">
		<!-- Verbindungsstatus -->
		<div class="flex items-center gap-2">
			<div class="w-2 h-2 rounded-full {connected ? 'bg-green-500' : 'bg-red-500'}"></div>
			<span class="text-sm text-gray-400">
				{connected ? 'Verbunden' : 'Nicht verbunden'}
			</span>
		</div>

		<!-- Views -->
		<div class="flex gap-1">
			<button class="px-3 py-1 text-sm bg-blue-600 rounded">Mixer</button>
			<button class="px-3 py-1 text-sm bg-gray-700 rounded hover:bg-gray-600">Matrix</button>
			<button class="px-3 py-1 text-sm bg-gray-700 rounded hover:bg-gray-600">Meter</button>
		</div>

		<!-- Settings -->
		<button class="p-2 hover:bg-gray-700 rounded">
			⚙️
		</button>
	</div>
</header>
