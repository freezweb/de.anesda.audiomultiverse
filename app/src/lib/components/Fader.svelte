<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	
	export let value = 0.75; // 0-1.25 (0dB = 0.75, +10dB = 1.25)
	export let min = 0;
	export let max = 1.25;
	
	const dispatch = createEventDispatcher<{ change: number }>();
	
	let trackElement: HTMLDivElement;
	let isDragging = false;
	
	// Fader-Position in Prozent (invertiert: 0% = oben)
	$: position = Math.max(0, Math.min(100, ((max - value) / (max - min)) * 100));
	
	// dB-Wert Anzeige
	$: dbValue = valueToDb(value);
	
	function valueToDb(val: number): string {
		if (val < 0.001) return '-∞';
		if (val <= 0.75) {
			// 0 bis 0.75 = -inf bis 0dB
			const db = 20 * Math.log10(val / 0.75);
			return db.toFixed(1);
		} else {
			// 0.75 bis 1.25 = 0 bis +10dB
			const db = ((val - 0.75) / 0.25) * 10;
			return '+' + db.toFixed(1);
		}
	}
	
	function handleStart(e: MouseEvent | TouchEvent) {
		isDragging = true;
		handleMove(e);
		
		window.addEventListener('mousemove', handleMove);
		window.addEventListener('mouseup', handleEnd);
		window.addEventListener('touchmove', handleMove);
		window.addEventListener('touchend', handleEnd);
	}
	
	function handleMove(e: MouseEvent | TouchEvent) {
		if (!isDragging || !trackElement) return;
		
		const rect = trackElement.getBoundingClientRect();
		const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
		
		// Position berechnen (invertiert)
		let percent = ((clientY - rect.top) / rect.height) * 100;
		percent = Math.max(0, Math.min(100, percent));
		
		// Zu Wert konvertieren
		const newValue = max - (percent / 100) * (max - min);
		
		dispatch('change', newValue);
	}
	
	function handleEnd() {
		isDragging = false;
		
		window.removeEventListener('mousemove', handleMove);
		window.removeEventListener('mouseup', handleEnd);
		window.removeEventListener('touchmove', handleMove);
		window.removeEventListener('touchend', handleEnd);
	}
</script>

<div 
	class="relative w-full h-full select-none touch-none"
	bind:this={trackElement}
>
	<!-- Fader Track -->
	<div class="absolute inset-x-0 inset-y-4 bg-gray-800 rounded fader-track">
		<!-- Markierungen -->
		<div class="absolute inset-0 flex flex-col justify-between py-2 text-[8px] text-gray-500 pointer-events-none">
			<span class="text-center">+10</span>
			<span class="text-center">0</span>
			<span class="text-center">-10</span>
			<span class="text-center">-20</span>
			<span class="text-center">-∞</span>
		</div>
	</div>
	
	<!-- Fader Thumb -->
	<div 
		class="absolute left-0 right-0 h-8 bg-gradient-to-b from-gray-400 to-gray-600 rounded shadow-lg cursor-grab active:cursor-grabbing border border-gray-500"
		style="top: calc({position}% - 16px)"
		on:mousedown={handleStart}
		on:touchstart={handleStart}
		role="slider"
		aria-valuenow={value}
		aria-valuemin={min}
		aria-valuemax={max}
		tabindex="0"
	>
		<div class="absolute inset-x-1 top-1/2 -translate-y-1/2 h-0.5 bg-gray-800"></div>
	</div>
	
	<!-- dB Wert -->
	<div class="absolute bottom-0 left-0 right-0 text-center text-xs text-gray-400 font-mono">
		{dbValue} dB
	</div>
</div>
