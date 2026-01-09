<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Fader from './Fader.svelte';
	import Meter from './Meter.svelte';
	
	export let id: number;
	export let name: string;
	export let fader = 0.75;
	export let mute = false;
	export let solo = false;
	export let pan = 0;
	export let meterLevel = 0;
	export let meterPeak = 0;
	
	const dispatch = createEventDispatcher<{
		faderChange: number;
		muteToggle: void;
		soloToggle: void;
		panChange: number;
	}>();
	
	function handleFaderChange(e: CustomEvent<number>) {
		dispatch('faderChange', e.detail);
	}
	
	function handlePanChange(e: Event) {
		const target = e.target as HTMLInputElement;
		dispatch('panChange', parseInt(target.value) / 100);
	}
</script>

<div class="w-20 min-w-[80px] bg-mixer-surface rounded-lg flex flex-col p-2 gap-1 h-full">
	<!-- Kanalname -->
	<div class="text-center text-xs font-medium truncate" title={name}>
		{name}
	</div>

	<!-- Meter & Fader Area -->
	<div class="flex-1 flex gap-1 min-h-0">
		<!-- Meter -->
		<div class="w-3">
			<Meter value={meterLevel} peak={meterPeak} />
		</div>
		
		<!-- Fader -->
		<div class="flex-1">
			<Fader 
				value={fader} 
				on:change={handleFaderChange}
			/>
		</div>
	</div>

	<!-- Pan -->
	<div class="flex items-center gap-0.5">
		<span class="text-[8px] text-gray-500">L</span>
		<input 
			type="range" 
			min="-100" 
			max="100" 
			value={pan * 100}
			on:input={handlePanChange}
			class="flex-1 h-1 bg-gray-700 rounded-lg appearance-none cursor-pointer"
		/>
		<span class="text-[8px] text-gray-500">R</span>
	</div>

	<!-- Mute & Solo Buttons -->
	<div class="flex gap-0.5">
		<button 
			class="flex-1 py-0.5 text-[10px] font-bold rounded transition-colors"
			class:bg-red-600={mute}
			class:bg-gray-600={!mute}
			class:hover:bg-gray-500={!mute}
			on:click={() => dispatch('muteToggle')}
		>
			M
		</button>
		<button 
			class="flex-1 py-0.5 text-[10px] font-bold rounded transition-colors"
			class:bg-yellow-500={solo}
			class:text-black={solo}
			class:bg-gray-600={!solo}
			class:hover:bg-gray-500={!solo}
			on:click={() => dispatch('soloToggle')}
		>
			S
		</button>
	</div>

	<!-- Kanal-Nummer -->
	<div class="text-center text-[10px] text-gray-500">
		{id >= 1000 ? `O${id - 1000}` : id}
	</div>
</div>
