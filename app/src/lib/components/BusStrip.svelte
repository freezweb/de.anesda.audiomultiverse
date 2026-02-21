<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Fader from './Fader.svelte';
	import Meter from './Meter.svelte';
	
	export let bus: {
		id: number;
		name: string;
		level: number;
		mute: boolean;
		pan: number;
		stereoLinked: boolean;
		type: 'aux' | 'group';
		color?: string;
	};
	
	export let meterLevel = 0;
	export let meterPeak = 0;
	
	// Touch-Optimierung
	export let touchOptimized = false;
	
	const dispatch = createEventDispatcher<{
		levelChange: number;
		muteToggle: void;
		panChange: number;
		stereoLinkToggle: void;
		rename: string;
	}>();
	
	$: buttonClass = touchOptimized 
		? 'py-3 text-sm font-bold rounded touch-manipulation'
		: 'py-1 text-xs font-bold rounded';
	
	// Bus-Type Farbe
	$: typeColor = bus.type === 'aux' ? '#22c55e' : '#3b82f6'; // Grün für Aux, Blau für Group
	$: displayColor = bus.color ?? typeColor;
	
	function onLevelChange(e: CustomEvent<number>) {
		dispatch('levelChange', e.detail);
	}
	
	function onMuteToggle() {
		dispatch('muteToggle');
	}
	
	function onPanChange(e: Event) {
		const target = e.target as HTMLInputElement;
		dispatch('panChange', parseInt(target.value) / 100);
	}
	
	function onStereoLinkToggle() {
		dispatch('stereoLinkToggle');
	}
	
	// Level in dB
	$: levelDb = bus.level > 0 ? (20 * Math.log10(bus.level)).toFixed(1) : '-∞';
</script>

<div 
	class="w-24 min-w-[96px] bg-mixer-surface rounded-lg flex flex-col p-2 gap-2 h-full"
	style="border-top: 3px solid {displayColor}"
>
	<!-- Bus-Type Badge -->
	<div class="flex items-center justify-between">
		<span 
			class="text-[10px] px-1 py-0.5 rounded"
			style="background-color: {displayColor}; color: black"
		>
			{bus.type === 'aux' ? 'AUX' : 'GRP'}
		</span>
		<span class="text-[10px] text-gray-400">{bus.id + 1}</span>
	</div>
	
	<!-- Bus-Name -->
	<div class="text-center text-xs font-medium truncate" title={bus.name}>
		{bus.name}
	</div>

	<!-- Meter & Fader Area -->
	<div class="flex-1 flex gap-1 min-h-[200px]">
		<!-- Meter -->
		<div class="w-4">
			<Meter value={meterLevel} peak={meterPeak} />
		</div>
		
		<!-- Fader -->
		<div class="flex-1">
			<Fader 
				value={bus.level} 
				on:change={onLevelChange}
			/>
		</div>
	</div>
	
	<!-- Level Display -->
	<div class="text-center text-xs text-gray-400">
		{levelDb} dB
	</div>

	<!-- Pan -->
	<div class="flex items-center justify-center gap-1">
		<span class="text-xs text-gray-500">L</span>
		<input 
			type="range" 
			min="-100" 
			max="100" 
			value={(bus.pan - 0.5) * 200}
			on:input={onPanChange}
			class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
		/>
		<span class="text-xs text-gray-500">R</span>
	</div>

	<!-- Mute & Stereo Link Buttons -->
	<div class="flex gap-1">
		<button 
			class="{buttonClass} flex-1 {bus.mute ? 'bg-red-600' : 'bg-gray-600 hover:bg-gray-500'}"
			on:click={onMuteToggle}
			title="Mute"
		>
			M
		</button>
		<button 
			class="{buttonClass} flex-1 {bus.stereoLinked ? 'bg-purple-600' : 'bg-gray-600 hover:bg-gray-500'}"
			on:click={onStereoLinkToggle}
			title="Stereo Link"
		>
			⟷
		</button>
	</div>
</div>
