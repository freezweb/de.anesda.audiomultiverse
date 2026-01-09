import { writable, derived } from 'svelte/store';

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface ConnectionState {
	status: ConnectionStatus;
	serverUrl: string;
	lastError: string | null;
	reconnectAttempts: number;
}

const initialState: ConnectionState = {
	status: 'disconnected',
	serverUrl: 'ws://localhost:3000/ws',
	lastError: null,
	reconnectAttempts: 0
};

export const connectionState = writable<ConnectionState>(initialState);

export const isConnected = derived(connectionState, ($state) => $state.status === 'connected');
export const connectionStatus = derived(connectionState, ($state) => $state.status);

export function setConnecting(url: string) {
	connectionState.update(state => ({
		...state,
		status: 'connecting',
		serverUrl: url,
		lastError: null
	}));
}

export function setConnected() {
	connectionState.update(state => ({
		...state,
		status: 'connected',
		lastError: null,
		reconnectAttempts: 0
	}));
}

export function setDisconnected() {
	connectionState.update(state => ({
		...state,
		status: 'disconnected'
	}));
}

export function setError(error: string) {
	connectionState.update(state => ({
		...state,
		status: 'error',
		lastError: error,
		reconnectAttempts: state.reconnectAttempts + 1
	}));
}

export function updateServerUrl(url: string) {
	connectionState.update(state => ({
		...state,
		serverUrl: url
	}));
}
