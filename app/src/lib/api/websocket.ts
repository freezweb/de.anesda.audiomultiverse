import { 
	connectionState, 
	setConnecting, 
	setConnected, 
	setDisconnected, 
	setError 
} from '$lib/stores/connection';
import { 
	setFullState, 
	updateMeterData, 
	updateChannel,
	type MixerState,
	type MeterData
} from '$lib/stores/mixer';
import { get } from 'svelte/store';

// Message-Typen (passend zum Server-Protokoll)
interface ClientMessage {
	type: string;
	[key: string]: unknown;
}

interface ServerMessage {
	type: string;
	[key: string]: unknown;
}

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let pingInterval: ReturnType<typeof setInterval> | null = null;

const RECONNECT_DELAY = 3000;
const PING_INTERVAL = 30000;

export function connect(url?: string) {
	const state = get(connectionState);
	const serverUrl = url || state.serverUrl;
	
	if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
		return;
	}
	
	setConnecting(serverUrl);
	
	try {
		ws = new WebSocket(serverUrl);
		
		ws.onopen = handleOpen;
		ws.onclose = handleClose;
		ws.onerror = handleError;
		ws.onmessage = handleMessage;
	} catch (error) {
		setError(`Verbindungsfehler: ${error}`);
		scheduleReconnect();
	}
}

export function disconnect() {
	if (reconnectTimer) {
		clearTimeout(reconnectTimer);
		reconnectTimer = null;
	}
	
	if (pingInterval) {
		clearInterval(pingInterval);
		pingInterval = null;
	}
	
	if (ws) {
		ws.close();
		ws = null;
	}
	
	setDisconnected();
}

export function send(message: ClientMessage) {
	if (ws && ws.readyState === WebSocket.OPEN) {
		ws.send(JSON.stringify(message));
	} else {
		console.warn('WebSocket nicht verbunden, Nachricht verworfen:', message);
	}
}

// Event-Handler
function handleOpen() {
	setConnected();
	
	// Vollständigen State anfordern
	send({ type: 'GetFullState' });
	
	// Ping starten
	pingInterval = setInterval(() => {
		send({ type: 'Ping' });
	}, PING_INTERVAL);
}

function handleClose() {
	ws = null;
	
	if (pingInterval) {
		clearInterval(pingInterval);
		pingInterval = null;
	}
	
	setDisconnected();
	scheduleReconnect();
}

function handleError(event: Event) {
	console.error('WebSocket Fehler:', event);
	setError('Verbindungsfehler');
}

function handleMessage(event: MessageEvent) {
	try {
		const message: ServerMessage = JSON.parse(event.data);
		processMessage(message);
	} catch (error) {
		console.error('Fehler beim Parsen der Nachricht:', error);
	}
}

function processMessage(message: ServerMessage) {
	switch (message.type) {
		case 'FullState':
			setFullState(message.state as MixerState);
			break;
			
		case 'MeterUpdate':
			updateMeterData(message.meters as MeterData[]);
			break;
			
		case 'ChannelUpdate':
			updateChannel(
				message.channelId as number, 
				message.updates as Record<string, unknown>
			);
			break;
			
		case 'Pong':
			// Pong empfangen, Verbindung ist aktiv
			break;
			
		case 'Error':
			console.error('Server-Fehler:', message.message);
			break;
			
		default:
			console.log('Unbekannte Nachricht:', message);
	}
}

function scheduleReconnect() {
	const state = get(connectionState);
	
	if (state.reconnectAttempts < 10) {
		const delay = RECONNECT_DELAY * Math.min(state.reconnectAttempts + 1, 5);
		
		reconnectTimer = setTimeout(() => {
			connect();
		}, delay);
	}
}

// Client-Aktionen (zum Server senden)
export function sendChannelFader(channelId: number, value: number) {
	send({
		type: 'SetChannelFader',
		channelId,
		value
	});
}

export function sendChannelMute(channelId: number, muted: boolean) {
	send({
		type: 'SetChannelMute',
		channelId,
		muted
	});
}

export function sendChannelSolo(channelId: number, solo: boolean) {
	send({
		type: 'SetChannelSolo',
		channelId,
		solo
	});
}

export function sendChannelPan(channelId: number, pan: number) {
	send({
		type: 'SetChannelPan',
		channelId,
		pan
	});
}

export function sendRouting(input: number, output: number, enabled: boolean) {
	send({
		type: 'SetRouting',
		inputChannel: input,
		outputChannel: output,
		enabled
	});
}

export function sendMidiLearnStart(channelId: number, parameter: string) {
	send({
		type: 'StartMidiLearn',
		channelId,
		parameter
	});
}

export function sendMidiLearnCancel() {
	send({
		type: 'CancelMidiLearn'
	});
}

export function sendSceneSave(name: string) {
	send({
		type: 'SaveScene',
		name
	});
}

export function sendSceneRecall(name: string) {
	send({
		type: 'RecallScene',
		name
	});
}
