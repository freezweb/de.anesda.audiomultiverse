<script lang="ts">
	export let value = 0;
	export let peak = 0;
	
	$: meterHeight = Math.max(0, Math.min(100, value * 100));
	$: peakPosition = Math.max(0, Math.min(100, peak * 100));
	
	$: isYellow = value > 0.25;
	$: isRed = value > 0.71;
	$: isClipping = value >= 1.0;
</script>

<div class="relative w-full h-full bg-gray-900 rounded overflow-hidden">
	<!-- Meter-Balken -->
	<div 
		class="absolute bottom-0 left-0 right-0"
		class:bg-green-500={!isYellow}
		class:bg-yellow-500={isYellow && !isRed}
		class:bg-red-500={isRed}
		style="height: {meterHeight}%"
	></div>
	
	<!-- Peak-Indikator -->
	{#if peak > 0.01}
		<div 
			class="absolute left-0 right-0 h-px bg-white"
			style="bottom: {peakPosition}%"
		></div>
	{/if}
	
	<!-- Clip-Indikator -->
	{#if isClipping}
		<div class="absolute top-0 left-0 right-0 h-1 bg-red-600 animate-pulse"></div>
	{/if}
</div>
