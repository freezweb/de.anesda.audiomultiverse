<script lang="ts">
	/**
	 * Erweiterte Meter-Komponente mit RMS, LUFS und Korrelation
	 * Für professionelles Monitoring
	 */
	
	export let peakLeft = 0;
	export let peakRight = 0;
	export let rmsLeft = 0;
	export let rmsRight = 0;
	export let lufs = -70; // LUFS Wert
	export let correlation = 1; // -1 bis +1
	export let showLufs = true;
	export let showCorrelation = true;
	
	// Peak in dB
	$: peakLeftDb = peakLeft > 0.001 ? (20 * Math.log10(peakLeft)).toFixed(1) : '-∞';
	$: peakRightDb = peakRight > 0.001 ? (20 * Math.log10(peakRight)).toFixed(1) : '-∞';
	
	// RMS in dB
	$: rmsLeftDb = rmsLeft > 0.001 ? (20 * Math.log10(rmsLeft)).toFixed(1) : '-∞';
	$: rmsRightDb = rmsRight > 0.001 ? (20 * Math.log10(rmsRight)).toFixed(1) : '-∞';
	
	// Meter-Höhen (0-100%)
	$: peakLeftHeight = Math.max(0, Math.min(100, dbToPercent(20 * Math.log10(peakLeft || 0.00001))));
	$: peakRightHeight = Math.max(0, Math.min(100, dbToPercent(20 * Math.log10(peakRight || 0.00001))));
	$: rmsLeftHeight = Math.max(0, Math.min(100, dbToPercent(20 * Math.log10(rmsLeft || 0.00001))));
	$: rmsRightHeight = Math.max(0, Math.min(100, dbToPercent(20 * Math.log10(rmsRight || 0.00001))));
	
	// LUFS-Meter (Bereich: -70 bis 0)
	$: lufsHeight = Math.max(0, Math.min(100, (lufs + 70) / 70 * 100));
	$: lufsDisplay = lufs > -70 ? lufs.toFixed(1) : '-∞';
	
	// Korrelations-Farbe (-1 = rot/out of phase, 0 = gelb/mono, 1 = grün/stereo)
	$: correlationColor = correlation < 0 ? 'red' : correlation < 0.5 ? 'yellow' : 'green';
	$: correlationPosition = ((correlation + 1) / 2) * 100; // -1 bis +1 auf 0-100%
	
	// Clipping-Erkennung
	$: clippingLeft = peakLeft >= 1.0;
	$: clippingRight = peakRight >= 1.0;
	
	// dB zu Prozent (für Meter-Anzeige, -60dB = 0%, 0dB = 100%)
	function dbToPercent(db: number): number {
		if (db <= -60) return 0;
		if (db >= 0) return 100;
		return ((db + 60) / 60) * 100;
	}
	
	// Meter-Farbe basierend auf dB
	function getMeterColor(db: number): string {
		if (db >= -3) return '#ef4444'; // rot
		if (db >= -12) return '#eab308'; // gelb
		return '#22c55e'; // grün
	}
	
	$: leftColor = getMeterColor(20 * Math.log10(peakLeft || 0.00001));
	$: rightColor = getMeterColor(20 * Math.log10(peakRight || 0.00001));
</script>

<div class="flex flex-col gap-2 bg-gray-900 rounded-lg p-2 h-full">
	<!-- Hauptmeter (Peak + RMS) -->
	<div class="flex-1 flex gap-1">
		<!-- Skala -->
		<div class="flex flex-col justify-between text-[8px] text-gray-500 py-1 w-6 text-right pr-1">
			<span>0</span>
			<span>-6</span>
			<span>-12</span>
			<span>-24</span>
			<span>-48</span>
			<span>-60</span>
		</div>
		
		<!-- Left Channel -->
		<div class="relative w-4 bg-gray-800 rounded overflow-hidden">
			<!-- Peak Bar -->
			<div 
				class="absolute bottom-0 left-0 right-0 transition-all duration-75"
				style="height: {peakLeftHeight}%; background-color: {leftColor}"
			></div>
			<!-- RMS Bar (dunkler overlay) -->
			<div 
				class="absolute bottom-0 left-0.5 right-0.5 bg-white/30 transition-all duration-150"
				style="height: {rmsLeftHeight}%"
			></div>
			<!-- Clip Indicator -->
			{#if clippingLeft}
				<div class="absolute top-0 left-0 right-0 h-2 bg-red-600 animate-pulse"></div>
			{/if}
		</div>
		
		<!-- Right Channel -->
		<div class="relative w-4 bg-gray-800 rounded overflow-hidden">
			<!-- Peak Bar -->
			<div 
				class="absolute bottom-0 left-0 right-0 transition-all duration-75"
				style="height: {peakRightHeight}%; background-color: {rightColor}"
			></div>
			<!-- RMS Bar -->
			<div 
				class="absolute bottom-0 left-0.5 right-0.5 bg-white/30 transition-all duration-150"
				style="height: {rmsRightHeight}%"
			></div>
			<!-- Clip Indicator -->
			{#if clippingRight}
				<div class="absolute top-0 left-0 right-0 h-2 bg-red-600 animate-pulse"></div>
			{/if}
		</div>
		
		<!-- LUFS Meter -->
		{#if showLufs}
			<div class="relative w-4 bg-gray-800 rounded overflow-hidden ml-1">
				<div 
					class="absolute bottom-0 left-0 right-0 bg-blue-500 transition-all duration-300"
					style="height: {lufsHeight}%"
				></div>
				<div class="absolute inset-0 flex items-center justify-center">
					<span class="text-[6px] text-white font-bold rotate-90 whitespace-nowrap">LUFS</span>
				</div>
			</div>
		{/if}
	</div>
	
	<!-- Numerische Anzeige -->
	<div class="grid grid-cols-2 gap-1 text-[9px]">
		<div class="text-center">
			<div class="text-gray-500">Peak L</div>
			<div class="font-mono {clippingLeft ? 'text-red-500' : 'text-white'}">{peakLeftDb}</div>
		</div>
		<div class="text-center">
			<div class="text-gray-500">Peak R</div>
			<div class="font-mono {clippingRight ? 'text-red-500' : 'text-white'}">{peakRightDb}</div>
		</div>
		<div class="text-center">
			<div class="text-gray-500">RMS L</div>
			<div class="font-mono text-gray-300">{rmsLeftDb}</div>
		</div>
		<div class="text-center">
			<div class="text-gray-500">RMS R</div>
			<div class="font-mono text-gray-300">{rmsRightDb}</div>
		</div>
	</div>
	
	<!-- LUFS Anzeige -->
	{#if showLufs}
		<div class="text-center">
			<div class="text-[8px] text-gray-500">LUFS</div>
			<div class="text-sm font-mono text-blue-400">{lufsDisplay}</div>
		</div>
	{/if}
	
	<!-- Korrelations-Meter -->
	{#if showCorrelation}
		<div class="mt-1">
			<div class="text-[8px] text-gray-500 text-center mb-0.5">Correlation</div>
			<div class="relative h-3 bg-gray-800 rounded overflow-hidden">
				<!-- Hintergrund-Gradient -->
				<div class="absolute inset-0 flex">
					<div class="flex-1 bg-gradient-to-r from-red-500 via-yellow-500 to-green-500 opacity-30"></div>
				</div>
				<!-- Center-Markierung -->
				<div class="absolute left-1/2 top-0 bottom-0 w-px bg-gray-600"></div>
				<!-- Korrelations-Indikator -->
				<div 
					class="absolute top-0.5 bottom-0.5 w-2 rounded transition-all duration-100"
					style="left: calc({correlationPosition}% - 4px); background-color: {correlationColor === 'red' ? '#ef4444' : correlationColor === 'yellow' ? '#eab308' : '#22c55e'}"
				></div>
			</div>
			<div class="flex justify-between text-[7px] text-gray-500">
				<span>-1</span>
				<span>0</span>
				<span>+1</span>
			</div>
		</div>
	{/if}
</div>
