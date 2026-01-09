<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Fader from './Fader.svelte';
	import Meter from './Meter.svelte';
	import { meterData } from '$lib/stores/mixer';

	export let channel: {
		id: number;
		name: string;
		fader: number;
		mute: boolean;
		solo: boolean;
		pan: number;
		color?: string;
	};
	
	const dispatch = createEventDispatcher<{
		faderChange: number;
		muteToggle: void;
		soloToggle: void;
		panChange: number;
	}>();
	
	// Meter-Daten für diesen Kanal
	$: meter = $meterData.get(channel.id);

	function onFaderChange(e: CustomEvent<number>) {
		dispatch('faderChange', e.detail);
	}

	function onMuteToggle() {
		dispatch('muteToggle');
	}

	function onSoloToggle() {
		dispatch('soloToggle');
	}
	
	function onPanChange(e: Event) {
		const target = e.target as HTMLInputElement;
		dispatch('panChange', parseInt(target.value) / 100);
	}
</script>

<div 
	class="w-24 min-w-[96px] bg-mixer-surface rounded-lg flex flex-col p-2 gap-2 h-full"
	style="border-top: 3px solid {channel.color ?? '#666'}"
>
	<!-- Kanalname -->
	<div class="text-center text-xs font-medium truncate" title={channel.name}>
		{channel.name}
	</div>

	<!-- Meter & Fader Area -->
	<div class="flex-1 flex gap-1 min-h-[200px]">
		<!-- Meter -->
		<div class="w-4">
			<Meter value={meter?.level ?? 0} peak={meter?.peak ?? 0} />
		</div>
		
		<!-- Fader -->
		<div class="flex-1">
			<Fader 
				value={channel.fader} 
				on:change={onFaderChange}
			/>
		</div>
	</div>

	<!-- Pan -->
	<div class="flex items-center justify-center gap-1">
		<span class="text-xs text-gray-500">L</span>
		<input 
			type="range" 
			min="-100" 
			max="100" 
			value={channel.pan * 100}
			on:input={onPanChange}
			class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
		/>
		<span class="text-xs text-gray-500">R</span>
	</div>

	<!-- Mute & Solo Buttons -->
	<div class="flex gap-1">
		<button 
			class="flex-1 py-1 text-xs font-bold rounded {channel.mute ? 'bg-red-600' : 'bg-gray-600 hover:bg-gray-500'}"
			on:click={onMuteToggle}
		>
			M
		</button>
		<button 
			class="flex-1 py-1 text-xs font-bold rounded {channel.solo ? 'bg-yellow-500 text-black' : 'bg-gray-600 hover:bg-gray-500'}"
			on:click={onSoloToggle}
		>
			S
		</button>
	</div>

	<!-- Kanal-Nummer -->
	<div class="text-center text-xs text-gray-500">
		{channel.id + 1}
	</div>
</div>
