<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { channels, routing } from '$lib/stores/mixer';
	
	const dispatch = createEventDispatcher<{
		routingChange: { input: number; output: number; enabled: boolean }
	}>();
	
	export let inputCount = 32;
	export let outputCount = 32;
	
	// Routing-State als Set für schnellen Zugriff
	$: routingSet = new Set(
		$routing
			.filter(r => r.enabled)
			.map(r => `${r.inputChannel}-${r.outputChannel}`)
	);
	
	// Input-Kanäle (IDs < 1000)
	$: inputs = $channels.filter(ch => ch.id < 1000);
	
	// Output-Kanäle (IDs >= 1000)
	$: outputs = $channels.filter(ch => ch.id >= 1000);
	
	function isRouted(input: number, output: number): boolean {
		return routingSet.has(`${input}-${output}`);
	}
	
	function toggleRouting(input: number, output: number) {
		const enabled = !isRouted(input, output);
		dispatch('routingChange', { input, output, enabled });
	}
	
	// Zoom/Scroll state
	let scale = 1;
	let scrollX = 0;
	let scrollY = 0;
</script>

<div class="h-full flex flex-col bg-mixer-bg">
	<!-- Header -->
	<div class="flex items-center justify-between p-2 bg-mixer-surface border-b border-gray-700">
		<h2 class="text-lg font-semibold">Routing-Matrix</h2>
		<div class="flex gap-2">
			<button 
				class="px-2 py-1 text-sm bg-gray-700 rounded hover:bg-gray-600"
				on:click={() => scale = Math.min(2, scale + 0.25)}
			>
				+
			</button>
			<span class="text-sm text-gray-400">{Math.round(scale * 100)}%</span>
			<button 
				class="px-2 py-1 text-sm bg-gray-700 rounded hover:bg-gray-600"
				on:click={() => scale = Math.max(0.5, scale - 0.25)}
			>
				−
			</button>
		</div>
	</div>
	
	<!-- Matrix -->
	<div class="flex-1 overflow-auto p-2">
		<div 
			class="inline-block"
			style="transform: scale({scale}); transform-origin: top left;"
		>
			<table class="border-collapse">
				<!-- Header (Outputs) -->
				<thead>
					<tr>
						<th class="w-16 h-6"></th>
						{#each outputs as output, i}
							<th class="w-6 h-6 text-[8px] text-gray-400 font-normal">
								<div class="transform -rotate-45 origin-bottom-left whitespace-nowrap">
									{output.name}
								</div>
							</th>
						{/each}
					</tr>
				</thead>
				
				<!-- Body (Inputs × Outputs) -->
				<tbody>
					{#each inputs as input, inputIdx}
						<tr>
							<!-- Input Label -->
							<td class="text-[10px] text-gray-400 text-right pr-1 truncate max-w-[60px]">
								{input.name}
							</td>
							
							<!-- Routing Points -->
							{#each outputs as output, outputIdx}
								{@const routed = isRouted(input.id, output.id)}
								<td class="p-0">
									<button
										class="w-5 h-5 m-0.5 rounded-sm transition-all duration-75"
										class:bg-green-500={routed}
										class:bg-gray-700={!routed}
										class:hover:bg-green-400={routed}
										class:hover:bg-gray-600={!routed}
										on:click={() => toggleRouting(input.id, output.id)}
										title="{input.name} → {output.name}"
									>
										{#if routed}
											<span class="text-[8px]">✓</span>
										{/if}
									</button>
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
	
	<!-- Legend -->
	<div class="flex items-center gap-4 p-2 bg-mixer-surface border-t border-gray-700 text-xs text-gray-400">
		<div class="flex items-center gap-1">
			<div class="w-3 h-3 bg-green-500 rounded-sm"></div>
			<span>Verbunden</span>
		</div>
		<div class="flex items-center gap-1">
			<div class="w-3 h-3 bg-gray-700 rounded-sm"></div>
			<span>Nicht verbunden</span>
		</div>
		<span class="ml-auto">
			{inputs.length} Eingänge × {outputs.length} Ausgänge
		</span>
	</div>
</div>
