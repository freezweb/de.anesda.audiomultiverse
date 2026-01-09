import { writable, derived } from 'svelte/store';

export interface ChannelState {
	id: number;
	name: string;
	fader: number;
	mute: boolean;
	solo: boolean;
	pan: number;
	gain: number;
}

export interface MeterData {
	channelId: number;
	level: number;
	peak: number;
}

export interface RoutingPoint {
	inputChannel: number;
	outputChannel: number;
	enabled: boolean;
}

export interface MixerState {
	channels: ChannelState[];
	routing: RoutingPoint[];
	masterFader: number;
	masterMute: boolean;
}

const initialState: MixerState = {
	channels: [],
	routing: [],
	masterFader: 0.75,
	masterMute: false
};

export const mixerState = writable<MixerState>(initialState);
export const meterData = writable<Map<number, MeterData>>(new Map());

export const channels = derived(mixerState, ($state) => $state.channels);
export const routing = derived(mixerState, ($state) => $state.routing);
export const hasSolo = derived(mixerState, ($state) => 
	$state.channels.some(ch => ch.solo)
);

export function updateChannel(channelId: number, updates: Partial<ChannelState>) {
	mixerState.update(state => ({
		...state,
		channels: state.channels.map(ch => 
			ch.id === channelId ? { ...ch, ...updates } : ch
		)
	}));
}

export function updateMeterData(data: MeterData[]) {
	meterData.update(map => {
		const newMap = new Map(map);
		for (const meter of data) {
			newMap.set(meter.channelId, meter);
		}
		return newMap;
	});
}

export function setFullState(state: MixerState) {
	mixerState.set(state);
}
