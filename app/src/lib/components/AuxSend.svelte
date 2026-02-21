<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	
	export let channelId: number;
	export let auxBusId: number;
	export let auxBusName: string;
	export let level = 0;
	export let enabled = false;
	export let mode: 'pre' | 'post' = 'post';
	export let pan = 0.5;
	
	// Touch-Optimierung
	export let touchOptimized = false;
	
	const dispatch = createEventDispatcher<{
		levelChange: { auxBusId: number; level: number };
		enabledChange: { auxBusId: number; enabled: boolean };
		modeChange: { auxBusId: number; mode: 'pre' | 'post' };
		panChange: { auxBusId: number; pan: number };
	}>();
	
	$: buttonClass = touchOptimized ? 'py-2 px-3' : 'py-1 px-2';
	
	function onLevelChange(e: Event) {
		const target = e.target as HTMLInputElement;
		const value = parseInt(target.value) / 100;
		dispatch('levelChange', { auxBusId, level: value });
	}
	
	function onEnabledToggle() {
		dispatch('enabledChange', { auxBusId, enabled: !enabled });
	}
	
	function onModeToggle() {
		const newMode = mode === 'pre' ? 'post' : 'pre';
		dispatch('modeChange', { auxBusId, mode: newMode });
	}
	
	function onPanChange(e: Event) {
		const target = e.target as HTMLInputElement;
		dispatch('panChange', { auxBusId, pan: parseInt(target.value) / 100 });
	}
	
	// Level in dB anzeigen
	$: levelDb = level > 0 ? (20 * Math.log10(level)).toFixed(1) : '-∞';
</script>

<div class="bg-gray-800 rounded-lg p-2 flex flex-col gap-1">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<span class="text-xs font-medium text-gray-300 truncate" title={auxBusName}>
			{auxBusName}
		</span>
		<button 
			class="{buttonClass} text-xs rounded {enabled ? 'bg-green-600' : 'bg-gray-600 hover:bg-gray-500'}"
			on:click={onEnabledToggle}
			title={enabled ? 'Send deaktivieren' : 'Send aktivieren'}
		>
			{enabled ? 'ON' : 'OFF'}
		</button>
	</div>
	
	<!-- Level Fader (horizontal) -->
	<div class="flex items-center gap-2">
		<input 
			type="range" 
			min="0" 
			max="100" 
			value={level * 100}
			on:input={onLevelChange}
			class="flex-1 h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer 
				   {enabled ? 'accent-green-500' : 'accent-gray-500'}"
			disabled={!enabled}
		/>
		<span class="text-xs w-12 text-right {enabled ? 'text-white' : 'text-gray-500'}">
			{levelDb} dB
		</span>
	</div>
	
	<!-- Pre/Post & Pan -->
	<div class="flex items-center gap-2">
		<!-- Pre/Post Toggle -->
		<button 
			class="{buttonClass} text-xs rounded {mode === 'pre' ? 'bg-blue-600' : 'bg-gray-600'}"
			on:click={onModeToggle}
			title={mode === 'pre' ? 'Pre-Fader (vor dem Kanal-Fader)' : 'Post-Fader (nach dem Kanal-Fader)'}
		>
			{mode === 'pre' ? 'PRE' : 'POST'}
		</button>
		
		<!-- Pan -->
		<div class="flex-1 flex items-center gap-1">
			<span class="text-xs text-gray-500">L</span>
			<input 
				type="range" 
				min="0" 
				max="100" 
				value={pan * 100}
				on:input={onPanChange}
				class="flex-1 h-1 bg-gray-700 rounded-lg appearance-none cursor-pointer"
				disabled={!enabled}
			/>
			<span class="text-xs text-gray-500">R</span>
		</div>
	</div>
</div>

<style>
	/* Custom Slider Styling */
	input[type="range"]::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: currentColor;
		cursor: pointer;
	}
	
	input[type="range"]::-moz-range-thumb {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: currentColor;
		cursor: pointer;
		border: none;
	}
</style>
