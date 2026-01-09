<script lang="ts">
	import { channels, meterData } from '$lib/stores/mixer';
	
	// Meter-Ansicht: Alle Kanäle in einer Übersicht
	
	$: inputChannels = $channels.filter(ch => ch.id < 1000);
	$: outputChannels = $channels.filter(ch => ch.id >= 1000);
	
	function getMeterHeight(channelId: number): number {
		const meter = $meterData.get(channelId);
		return meter ? Math.min(100, meter.level * 100) : 0;
	}
	
	function getMeterColor(level: number): string {
		if (level > 0.9) return 'bg-red-500';
		if (level > 0.71) return 'bg-yellow-500';
		return 'bg-green-500';
	}
	
	function levelToDb(level: number): string {
		if (level < 0.001) return '-∞';
		const db = 20 * Math.log10(level);
		return db.toFixed(1);
	}
</script>

<div class="h-full flex flex-col bg-mixer-bg p-4">
	<!-- Eingänge -->
	<div class="mb-6">
		<h2 class="text-lg font-semibold mb-3">Eingänge</h2>
		<div class="flex gap-1 flex-wrap">
			{#each inputChannels as channel}
				{@const meter = $meterData.get(channel.id)}
				{@const level = meter?.level ?? 0}
				<div class="w-8 flex flex-col items-center">
					<!-- Meter Bar -->
					<div class="w-3 h-24 bg-gray-800 rounded relative overflow-hidden">
						<div 
							class="absolute bottom-0 left-0 right-0 transition-all duration-75 {getMeterColor(level)}"
							style="height: {getMeterHeight(channel.id)}%"
						></div>
						
						<!-- Peak Hold -->
						{#if meter?.peak}
							<div 
								class="absolute left-0 right-0 h-0.5 bg-white"
								style="bottom: {Math.min(100, meter.peak * 100)}%"
							></div>
						{/if}
					</div>
					
					<!-- dB Value -->
					<span class="text-[8px] text-gray-400 mt-1">
						{levelToDb(level)}
					</span>
					
					<!-- Channel Number -->
					<span class="text-[8px] text-gray-500">
						{channel.id + 1}
					</span>
					
					<!-- Mute/Solo Indicator -->
					<div class="flex gap-0.5 mt-0.5">
						{#if channel.mute}
							<span class="text-[6px] text-red-500">M</span>
						{/if}
						{#if channel.solo}
							<span class="text-[6px] text-yellow-500">S</span>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
	
	<!-- Ausgänge -->
	<div>
		<h2 class="text-lg font-semibold mb-3">Ausgänge</h2>
		<div class="flex gap-1 flex-wrap">
			{#each outputChannels as channel}
				{@const meter = $meterData.get(channel.id)}
				{@const level = meter?.level ?? 0}
				<div class="w-8 flex flex-col items-center">
					<!-- Meter Bar -->
					<div class="w-3 h-24 bg-gray-800 rounded relative overflow-hidden">
						<div 
							class="absolute bottom-0 left-0 right-0 transition-all duration-75 {getMeterColor(level)}"
							style="height: {getMeterHeight(channel.id)}%"
						></div>
						
						{#if meter?.peak}
							<div 
								class="absolute left-0 right-0 h-0.5 bg-white"
								style="bottom: {Math.min(100, meter.peak * 100)}%"
							></div>
						{/if}
					</div>
					
					<span class="text-[8px] text-gray-400 mt-1">
						{levelToDb(level)}
					</span>
					
					<span class="text-[8px] text-gray-500">
						O{channel.id - 999}
					</span>
					
					<div class="flex gap-0.5 mt-0.5">
						{#if channel.mute}
							<span class="text-[6px] text-red-500">M</span>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
	
	<!-- Legende -->
	<div class="mt-auto pt-4 flex items-center gap-4 text-xs text-gray-400 border-t border-gray-700">
		<div class="flex items-center gap-1">
			<div class="w-3 h-3 bg-green-500 rounded"></div>
			<span>&lt; -3 dB</span>
		</div>
		<div class="flex items-center gap-1">
			<div class="w-3 h-3 bg-yellow-500 rounded"></div>
			<span>-3 bis 0 dB</span>
		</div>
		<div class="flex items-center gap-1">
			<div class="w-3 h-3 bg-red-500 rounded"></div>
			<span>&gt; 0 dB (Clip)</span>
		</div>
	</div>
</div>
