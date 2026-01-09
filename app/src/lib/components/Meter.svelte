<script lang="ts">
	export let value = 0; // 0-1 (Amplitude)
	export let peak = 0; // Peak-Hold Wert
	export let clipTimeout = 2000; // Peak-Hold Dauer in ms
	
	// Höhe in Prozent
	$: meterHeight = Math.max(0, Math.min(100, value * 100));
	$: peakPosition = Math.max(0, Math.min(100, peak * 100));
	
	// dB Anzeige
	$: dbValue = value > 0.001 ? (20 * Math.log10(value)).toFixed(1) : '-∞';
	
	// Farbzonen: Grün bis -12dB, Gelb bis -3dB, Rot darüber
	// -12dB = 10^(-12/20) ≈ 0.25
	// -3dB = 10^(-3/20) ≈ 0.71
	$: isYellow = value > 0.25;
	$: isRed = value > 0.71;
	$: isClipping = value >= 1.0;
</script>

<div class="relative w-full h-full bg-gray-900 rounded overflow-hidden">
	<!-- Meter-Hintergrund mit Skala -->
	<div class="absolute inset-0 flex flex-col justify-between py-1 text-[6px] text-gray-600 z-10 pointer-events-none">
		<span class="text-center">0</span>
		<span class="text-center">-3</span>
		<span class="text-center">-6</span>
		<span class="text-center">-12</span>
		<span class="text-center">-20</span>
		<span class="text-center">-∞</span>
	</div>
	
	<!-- Meter-Balken -->
	<div 
		class="absolute bottom-0 left-0 right-0 transition-all duration-75"
		style="height: {meterHeight}%"
	>
		<!-- Gradient: Grün -> Gelb -> Rot -->
		<div class="absolute inset-0 bg-gradient-to-t from-green-500 via-yellow-400 to-red-500"></div>
		
		<!-- Masken für Farbzonen -->
		{#if !isRed}
			<div 
				class="absolute top-0 left-0 right-0 bg-gradient-to-t from-yellow-500 to-yellow-500"
				style="height: {isYellow ? 100 - (0.71 / Math.max(value, 0.001)) * 100 : 100}%"
			></div>
		{/if}
		
		{#if !isYellow}
			<div 
				class="absolute top-0 left-0 right-0 bg-green-500"
				style="height: 100%"
			></div>
		{/if}
	</div>
	
	<!-- Einfacher farbiger Balken ohne komplexe Masken -->
	<div 
		class="absolute bottom-0 left-0.5 right-0.5"
		class:bg-green-500={!isYellow}
		class:bg-yellow-500={isYellow && !isRed}
		class:bg-red-500={isRed}
		style="height: {meterHeight}%"
	></div>
	
	<!-- Peak-Indikator -->
	{#if peak > 0.01}
		<div 
			class="absolute left-0.5 right-0.5 h-0.5 bg-white"
			style="bottom: {peakPosition}%"
		></div>
	{/if}
	
	<!-- Clip-Indikator -->
	{#if isClipping}
		<div class="absolute top-0.5 left-0.5 right-0.5 h-2 bg-red-600 animate-pulse rounded-sm"></div>
	{/if}
</div>
