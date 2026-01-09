import { writable, derived, type Writable } from 'svelte/store';

// Typen für den Mixer-State
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

// Initiales State
const initialState: MixerState = {
	channels: [],
	routing: [],
	masterFader: 0.75,
	masterMute: false
};

// Haupt-Store
export const mixerState = writable<MixerState>(initialState);

// Meter-Daten (separat für Performance)
export const meterData = writable<Map<number, MeterData>>(new Map());

// Derived Stores
export const channels = derived(mixerState, ($state) => $state.channels);
export const routing = derived(mixerState, ($state) => $state.routing);

// Solo-Status (ob irgendein Kanal solo ist)
export const hasSolo = derived(mixerState, ($state) => 
	$state.channels.some(ch => ch.solo)
);

// Aktionen
export function updateChannel(channelId: number, updates: Partial<ChannelState>) {
	mixerState.update(state => ({
		...state,
		channels: state.channels.map(ch => 
			ch.id === channelId ? { ...ch, ...updates } : ch
		)
	}));
}

export function setChannelFader(channelId: number, value: number) {
	updateChannel(channelId, { fader: value });
}

export function toggleChannelMute(channelId: number) {
	mixerState.update(state => ({
		...state,
		channels: state.channels.map(ch => 
			ch.id === channelId ? { ...ch, mute: !ch.mute } : ch
		)
	}));
}

export function toggleChannelSolo(channelId: number) {
	mixerState.update(state => ({
		...state,
		channels: state.channels.map(ch => 
			ch.id === channelId ? { ...ch, solo: !ch.solo } : ch
		)
	}));
}

export function setMasterFader(value: number) {
	mixerState.update(state => ({ ...state, masterFader: value }));
}

export function toggleMasterMute() {
	mixerState.update(state => ({ ...state, masterMute: !state.masterMute }));
}

export function setRouting(input: number, output: number, enabled: boolean) {
	mixerState.update(state => {
		const existingIndex = state.routing.findIndex(
			r => r.inputChannel === input && r.outputChannel === output
		);
		
		if (existingIndex >= 0) {
			const newRouting = [...state.routing];
			newRouting[existingIndex] = { inputChannel: input, outputChannel: output, enabled };
			return { ...state, routing: newRouting };
		} else {
			return { 
				...state, 
				routing: [...state.routing, { inputChannel: input, outputChannel: output, enabled }]
			};
		}
	});
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

// State komplett ersetzen (vom Server)
export function setFullState(state: MixerState) {
	mixerState.set(state);
}

// Initialisiere Kanäle (32 Eingänge + 32 Ausgänge)
export function initializeChannels(inputCount: number = 32, outputCount: number = 32) {
	const channels: ChannelState[] = [];
	
	// Input-Kanäle
	for (let i = 1; i <= inputCount; i++) {
		channels.push({
			id: i,
			name: `Input ${i}`,
			fader: 0.75,
			mute: false,
			solo: false,
			pan: 0,
			gain: 0
		});
	}
	
	// Output-Kanäle (IDs 1001+)
	for (let i = 1; i <= outputCount; i++) {
		channels.push({
			id: 1000 + i,
			name: `Output ${i}`,
			fader: 0.75,
			mute: false,
			solo: false,
			pan: 0,
			gain: 0
		});
	}
	
	mixerState.update(state => ({ ...state, channels }));
}
